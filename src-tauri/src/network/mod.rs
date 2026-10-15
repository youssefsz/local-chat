#[cfg(target_os = "macos")]
mod discovery;
#[cfg(not(target_os = "macos"))]
mod discovery_portable;
mod tls;
use crate::{
    domain::*,
    platform::{Identity, NetworkEvidence},
    storage::Store,
};
use anyhow::{Context as _, Result, ensure};
#[cfg(not(target_os = "macos"))]
use discovery_portable as discovery;
use ed25519_dalek::Signer;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::{
    net::TcpSocket,
    sync::{Mutex, Semaphore, broadcast, mpsc, oneshot},
    time::{interval, timeout},
};
use tokio_util::{
    codec::{Framed, LengthDelimitedCodec},
    sync::CancellationToken,
};

fn valid_hex_key(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
type SequenceKey = (String, String, i64);
#[derive(Clone, Debug)]
pub enum Change {
    State,
    History,
    Event(Event),
    Received(Event, bool),
}
/// The view reloads durable state after a burst; missing a notification is not an exit.
pub async fn next_view_change(changes: &mut broadcast::Receiver<Change>) -> Option<Change> {
    match changes.recv().await {
        Ok(change) => Some(change),
        Err(broadcast::error::RecvError::Lagged(_)) => Some(Change::History),
        Err(broadcast::error::RecvError::Closed) => None,
    }
}
type ReconnectReply = oneshot::Sender<std::result::Result<(), String>>;
struct Presence {
    peer: Peer,
    session_id: uuid::Uuid,
}
struct RunningNetwork {
    cancel: CancellationToken,
    task: tokio::task::JoinHandle<()>,
}
impl RunningNetwork {
    async fn shutdown(self) {
        self.cancel.cancel();
        let _ = self.task.await;
    }
}
pub struct Engine {
    pub store: Mutex<Store>,
    pub identity: Identity,
    peers: Mutex<HashMap<String, Presence>>,
    pub status: Mutex<(String, String)>,
    pub changes: broadcast::Sender<Change>,
    pub stop: CancellationToken,
    context_generation: AtomicU64,
    reconnect_tx: mpsc::Sender<ReconnectReply>,
    reconnect_rx: Mutex<mpsc::Receiver<ReconnectReply>>,
    reconnect_gate: Semaphore,
}
impl Engine {
    pub fn new(store: Store, identity: Identity) -> Arc<Self> {
        let (changes, _) = broadcast::channel(256);
        let (reconnect_tx, reconnect_rx) = mpsc::channel(1);
        Arc::new(Self {
            store: Mutex::new(store),
            identity,
            peers: Mutex::new(HashMap::new()),
            status: Mutex::new(("discovering".into(), "Looking for a local network".into())),
            changes,
            stop: CancellationToken::new(),
            context_generation: AtomicU64::new(crate::platform::network_generation()),
            reconnect_tx,
            reconnect_rx: Mutex::new(reconnect_rx),
            reconnect_gate: Semaphore::new(1),
        })
    }
    pub fn changed(&self) {
        let _ = self.changes.send(Change::State);
    }
    pub async fn snapshot(&self) -> Result<Snapshot> {
        let db = self.store.lock().await;
        let mut status = self.status.lock().await.clone();
        let peers: Vec<Peer> = self
            .peers
            .lock()
            .await
            .values()
            .map(|presence| presence.peer.clone())
            .collect();
        if status.0 != "unavailable" && !peers.iter().any(|p| p.status == "connected") {
            if peers.iter().any(|p| p.status == "reconnecting") {
                status = (
                    "reconnecting".into(),
                    "Reconnecting to nearby computers".into(),
                );
            } else if status.0 == "connected" {
                status = (
                    "discovering".into(),
                    "Looking for nearby computers. Your messages remain saved here.".into(),
                );
            }
        }
        Ok(Snapshot {
            identity: self.identity.id(),
            profile: db.meta("profile", "You")?,
            theme: db.meta("theme", "system")?,
            notifications: db.meta("notifications", "true")? == "true",
            context: db.active.clone(),
            contexts: db.contexts()?,
            rooms: db.rooms(&db.active)?,
            peers,
            network_status: status.0,
            network_detail: status.1,
        })
    }
    pub async fn publish(&self, payload: Payload) -> Result<Event> {
        let event = self
            .store
            .lock()
            .await
            .create(&self.identity.signing, payload)?;
        let _ = self.changes.send(Change::Event(event.clone()));
        Ok(event)
    }
    /// Restart network discovery and sessions, without replacing identity or local history.
    pub async fn reconnect(&self) -> Result<()> {
        ensure!(!self.stop.is_cancelled(), "Local Chat is shutting down");
        let _permit = self
            .reconnect_gate
            .try_acquire()
            .context("Already reconnecting to nearby computers")?;
        let (reply, response) = oneshot::channel();
        self.reconnect_tx
            .try_send(reply)
            .context("A reconnect is already queued")?;
        timeout(Duration::from_secs(15), response)
            .await
            .context("Reconnecting took too long. Try again.")?
            .context("The network service stopped while reconnecting")?
            .map_err(anyhow::Error::msg)
    }
    pub async fn run(self: Arc<Self>) {
        let mut last: Option<NetworkEvidence> = None;
        let mut current: Option<RunningNetwork> = None;
        let mut requests = self.reconnect_rx.lock().await;
        let mut ticker = interval(Duration::from_millis(250));
        let mut last_generation = 0;
        let mut last_probe = std::time::Instant::now() - Duration::from_secs(30);
        // Instant may exclude macOS sleep. Wall time also catches suspension/wake.
        let mut previous_tick = std::time::SystemTime::now();
        loop {
            let request = tokio::select! {
                _ = self.stop.cancelled() => break,
                request = requests.recv() => request,
                _ = ticker.tick() => None,
            };
            if request.as_ref().is_some_and(oneshot::Sender::is_closed) {
                continue;
            }
            let manual = request.is_some();
            let resumed = previous_tick
                .elapsed()
                .map_or(true, |gap| gap > Duration::from_secs(8));
            previous_tick = std::time::SystemTime::now();
            let generation = crate::platform::network_generation();
            let signal_changed = last_generation != generation;
            if !manual
                && !resumed
                && !signal_changed
                && last_probe.elapsed() < Duration::from_secs(10)
            {
                continue;
            }
            last_generation = generation;
            last_probe = std::time::Instant::now();
            let had_network = current.is_some();
            if manual || signal_changed || resumed {
                if let Some(network) = current.take() {
                    network.shutdown().await;
                }
                self.peers.lock().await.clear();
                *self.status.lock().await = (
                    "reconnecting".into(),
                    "Checking the current network before reconnecting".into(),
                );
                self.changed();
            }
            let evidence = tokio::task::spawn_blocking(crate::platform::network_evidence)
                .await
                .ok()
                .and_then(Result::ok);
            let restart = manual
                || resumed
                || signal_changed
                || evidence != last
                || current.as_ref().is_none_or(|network| {
                    network.cancel.is_cancelled() || network.task.is_finished()
                });
            let mut outcome = Ok(());
            if restart {
                // Await old sessions before starting new ones: their cleanup owns peer state.
                if let Some(network) = current.take() {
                    network.shutdown().await;
                }
                self.peers.lock().await.clear();
                if let Some(e) = evidence.clone() {
                    let fingerprint = e.fingerprint();
                    let mut db = self.store.lock().await;
                    // A transport retry on an unchanged, continuously observed route is
                    // not a new network, even when its gateway has no hardware address.
                    // A failed activation leaves no running service. Observing its route
                    // again must retry activation, never reuse the previous context.
                    let continuous = had_network
                        && !resumed
                        && !signal_changed
                        && last.as_ref() == Some(&e)
                        && !db.active.is_empty();
                    let activated = if continuous {
                        Ok(db.active.clone())
                    } else {
                        db.activate(
                            fingerprint.as_deref(),
                            if fingerprint.is_some() {
                                "Local network"
                            } else {
                                "Unrecognized network"
                            },
                        )
                    };
                    drop(db);
                    outcome = activated.map(|_| ()).map_err(|e| e.to_string());
                    if outcome.is_ok()
                        && !self.stop.is_cancelled()
                        && crate::platform::network_generation() == generation
                    {
                        self.context_generation.store(generation, Ordering::Release);
                        let cancel = self.stop.child_token();
                        let service_cancel = cancel.clone();
                        let engine = self.clone();
                        let task = tokio::spawn(async move {
                            let _result = engine.clone().serve(e, service_cancel.clone()).await;
                            if !service_cancel.is_cancelled() {
                                engine.peers.lock().await.clear();
                                *engine.status.lock().await = ("unavailable".into(), "Local discovery is unavailable. Check local-network permissions and your firewall. Retrying automatically.".into());
                                service_cancel.cancel();
                                engine.changed();
                            }
                        });
                        current = Some(RunningNetwork { cancel, task });
                    } else if outcome.is_ok() {
                        outcome = Err(
                            "The network changed while reconnecting. Retrying automatically."
                                .into(),
                        );
                    }
                } else {
                    outcome =
                        Err("Connect to Wi-Fi or Ethernet. VPN interfaces are not shared.".into());
                }
                if let Err(message) = &outcome {
                    *self.status.lock().await = ("unavailable".into(), message.clone());
                }
                last = evidence;
                self.changed();
            }
            if let Some(reply) = request {
                let _ = reply.send(outcome);
            }
        }
        if let Some(network) = current {
            network.shutdown().await;
        }
        self.peers.lock().await.clear();
    }
    async fn serve(
        self: Arc<Self>,
        evidence: NetworkEvidence,
        cancel: CancellationToken,
    ) -> Result<()> {
        let socket = TcpSocket::new_v4()?;
        crate::platform::bind_physical_socket(&socket, &evidence)?;
        socket.bind(SocketAddr::new(IpAddr::V4(evidence.address), 0))?;
        let listener = socket.listen(32)?;
        let port = listener.local_addr()?.port();
        let id = self.identity.id();
        let mut discovery =
            discovery::Discovery::start(&evidence, &id, &self.identity.fingerprint(), port)?;
        let acceptor = tls::server(&self.identity)?;
        let context = self.store.lock().await.active.clone();
        *self.status.lock().await = (
            "discovering".into(),
            "Discovering nearby computers on this network".into(),
        );
        self.changed();
        let mut dials: HashMap<String, (SocketAddr, String, CancellationToken)> = HashMap::new();
        let mut sessions = tokio::task::JoinSet::new();
        loop {
            tokio::select! {
                _=cancel.cancelled()=>break,
                Some(_)=sessions.join_next(),if !sessions.is_empty()=>{},
                incoming=listener.accept()=>{let (stream,address)=incoming?;if !matches!(address.ip(),IpAddr::V4(a) if evidence.permits(a))||sessions.len()>=32{continue}
                    let acceptor=acceptor.clone();let engine=self.clone();let session_cancel=cancel.clone();let context=context.clone();
                    sessions.spawn(async move{let _=tokio::select!{_=session_cancel.cancelled()=>Ok(()),result=async{
                        let stream=timeout(Duration::from_secs(5),acceptor.accept(stream)).await??;
                        let binding=stream.get_ref().1.export_keying_material([0u8;32],b"localchat identity v1",None)?;
                        engine.session(stream,binding,None,context,session_cancel.clone()).await
                    }=>result};});
                },
                event=discovery.receiver.recv()=>{match event.context("Discovery stopped")?{
                    discovery::DiscoveryEvent::Resolved{peer,certificate:cert,address:ip,port}=>{
                        if peer<=id||!valid_hex_key(&peer)||!valid_hex_key(&cert)||!evidence.permits(ip)||dials.len()>=19 && !dials.contains_key(&peer){continue}
                        let endpoint=SocketAddr::new(IpAddr::V4(ip),port);
                        if let Some((old_endpoint,old_cert,old_cancel))=dials.get(&peer){if *old_endpoint==endpoint && *old_cert==cert {continue}old_cancel.cancel();self.peers.lock().await.remove(&peer);self.changed();}
                        let child=cancel.child_token();dials.insert(peer.clone(),(endpoint,cert.clone(),child.clone()));
                        let engine=self.clone();let context=context.clone();let local=evidence.address;let scope=evidence.clone();
                        sessions.spawn(async move{let mut attempt=0u32;loop{if child.is_cancelled(){break}
                            let result=tokio::select!{_=child.cancelled()=>break,result=async{
                                let socket=TcpSocket::new_v4()?;crate::platform::bind_physical_socket(&socket,&scope)?;socket.bind(SocketAddr::new(IpAddr::V4(local),0))?;
                                let tcp=timeout(Duration::from_secs(5),socket.connect(SocketAddr::new(IpAddr::V4(ip),port))).await??;
                                let connector=tls::client(cert.clone());let stream=timeout(Duration::from_secs(5),connector.connect("localchat.local".try_into()?,tcp)).await??;
                                let binding=stream.get_ref().1.export_keying_material([0u8;32],b"localchat identity v1",None)?;
                                engine.clone().session(stream,binding,Some(peer.clone()),context.clone(),child.clone()).await
                            }=>result};
                            #[cfg(feature="test-profile")] if let Err(err)=&result { eprintln!("Test dial ended: {err}"); }
                            if result.is_err() && !child.is_cancelled(){
                                let mut peers=engine.peers.lock().await;
                                if peers.get(&peer).is_none_or(|presence|presence.peer.status!="connected") {
                                    peers.insert(peer.clone(),Presence{peer:Peer{id:peer.clone(),name:"Nearby computer".into(),platform:String::new(),status:"reconnecting".into()},session_id:uuid::Uuid::new_v4()});
                                    drop(peers);engine.changed();
                                }
                            }
                            attempt=attempt.saturating_add(1);let jitter=rand::random::<u64>()%500;
                            tokio::select!{_=child.cancelled()=>break,_=tokio::time::sleep(Duration::from_millis((1u64<<attempt.min(5))*1000+jitter))=>{}}
                        }});
                    },
                    discovery::DiscoveryEvent::Removed(fullname)=>{if let Some(peer)=dials.keys().find(|peer|fullname.contains(&peer[..16])).cloned(){if let Some((_,_,child))=dials.remove(&peer){child.cancel();}self.peers.lock().await.remove(&peer);self.changed();}},
                    discovery::DiscoveryEvent::Failure(message)=>{self.store.lock().await.set_meta("last_discovery_error",&message)?;anyhow::bail!("Local discovery could not start. Check local-network permissions and your firewall. Local Chat retries automatically.");}
                }}
            }
        }
        cancel.cancel();
        sessions.abort_all();
        while sessions.join_next().await.is_some() {}
        drop(discovery);
        Ok(())
    }
    #[cfg(any(test, feature = "test-profile"))]
    pub async fn connect_test(
        self: Arc<Self>,
        stream: tokio::net::TcpStream,
        server: bool,
        expected: Option<String>,
        cert: String,
        cancel: CancellationToken,
    ) -> Result<()> {
        let context = self.store.lock().await.active.clone();
        if server {
            let stream = tls::server(&self.identity)?.accept(stream).await?;
            let binding = stream.get_ref().1.export_keying_material(
                [0u8; 32],
                b"localchat identity v1",
                None,
            )?;
            self.session(stream, binding, expected, context, cancel)
                .await
        } else {
            let stream = tls::client(cert)
                .connect("localchat.local".try_into()?, stream)
                .await?;
            let binding = stream.get_ref().1.export_keying_material(
                [0u8; 32],
                b"localchat identity v1",
                None,
            )?;
            self.session(stream, binding, expected, context, cancel)
                .await
        }
    }
    async fn session<T>(
        self: Arc<Self>,
        stream: T,
        binding: [u8; 32],
        expected: Option<String>,
        context: String,
        cancel: CancellationToken,
    ) -> Result<()>
    where
        T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let generation = self.context_generation.load(Ordering::Acquire);
        let codec = LengthDelimitedCodec::builder()
            .max_frame_length(FRAME_BYTES)
            .new_codec();
        let mut wire = Framed::new(stream, codec);
        let profile = self.store.lock().await.meta("profile", "You")?;
        let id = self.identity.id();
        let platform = std::env::consts::OS.to_string();
        let signature = hex::encode(
            self.identity
                .signing
                .sign(&serde_json::to_vec(&(
                    PROTOCOL, &id, &profile, &platform, binding,
                ))?)
                .to_bytes(),
        );
        eligible_send(
            &mut wire,
            generation,
            &Wire::Hello {
                version: PROTOCOL,
                id: id.clone(),
                profile,
                platform,
                signature,
            },
        )
        .await?;
        let hello = timeout(Duration::from_secs(5), receive(&mut wire)).await??;
        let Wire::Hello {
            version,
            id: peer,
            profile,
            platform,
            signature,
        } = hello
        else {
            anyhow::bail!("Expected authenticated hello")
        };
        ensure!(version == PROTOCOL && peer != id, "Incompatible peer");
        ensure!(
            expected.as_ref().is_none_or(|e| e == &peer),
            "Peer identity mismatch"
        );
        validate_name(&profile)?;
        ensure!(platform.len() <= 32, "Invalid platform");
        verify(
            &peer,
            &serde_json::to_vec(&(version, &peer, &profile, &platform, binding))?,
            &signature,
        )?;
        let session_id = uuid::Uuid::new_v4();
        {
            let mut peers = self.peers.lock().await;
            ensure!(
                peers
                    .get(&peer)
                    .is_none_or(|p| p.peer.status != "connected"),
                "Duplicate peer session"
            );
            peers.insert(
                peer.clone(),
                Presence {
                    session_id,
                    peer: Peer {
                        id: peer.clone(),
                        name: profile,
                        platform,
                        status: "connected".into(),
                    },
                },
            );
        }
        *self.status.lock().await = ("connected".into(), "Connected on your local network".into());
        self.changed();
        let result=async{
            let mut changes=self.changes.subscribe();let mut tick=interval(Duration::from_secs(10));let mut sent_digest=String::new();let mut requested_digest=String::new();let mut last_rx=std::time::Instant::now();
            loop{
                ensure!(!cancel.is_cancelled() && crate::platform::network_generation()==generation,"Network session expired");
                ensure!(self.store.lock().await.active==context,"Network context changed");
                tokio::select!{
                    _=cancel.cancelled()=>break,
                    _=tick.tick()=>{ensure!(last_rx.elapsed()<Duration::from_secs(35),"Peer liveness timeout");let digest=self.store.lock().await.digest()?;if digest!=sent_digest{eligible_send(&mut wire,generation,&Wire::Summary{digest:digest.clone()}).await?;sent_digest=digest;}else{eligible_send(&mut wire,generation,&Wire::Ping).await?;}},
                    change=changes.recv()=>{match change?{Change::Event(e)=>{eligible_send(&mut wire,generation,&Wire::Events{events:vec![e],live:true}).await?;},Change::Received(_,_)|Change::State|Change::History=>{}}},
                    packet=receive(&mut wire)=>{let packet=packet?;last_rx=std::time::Instant::now();
                        // Each operation holds the database lock through its context check.
                        let mut db=self.store.lock().await;ensure!(db.active==context&&!cancel.is_cancelled()&&crate::platform::network_generation()==generation,"Context no longer eligible");
                        match packet{
                            Wire::Summary{digest}=>{ensure!(digest.len()==64,"Invalid summary");if digest!=db.digest()? && digest!=requested_digest{requested_digest=digest;drop(db);eligible_send(&mut wire,generation,&Wire::InventoryRequest{offset:0}).await?;}},
                            Wire::InventoryRequest{offset}=>{let entries=db.inventory(offset)?;drop(db);eligible_send(&mut wire,generation,&Wire::Inventory{offset,entries}).await?;},
                            Wire::Inventory{offset,entries}=>{ensure!(entries.len()<=512,"Oversized inventory");let missing=db.missing(&entries)?;drop(db);for batch in missing.chunks(BATCH_SIZE){eligible_send(&mut wire,generation,&Wire::Need{entries:batch.to_vec()}).await?;}if entries.len()==512{eligible_send(&mut wire,generation,&Wire::InventoryRequest{offset:offset.checked_add(512).context("Inventory offset overflow")?}).await?;}},
                            Wire::Need{entries}=>{let events=db.export(&entries)?;drop(db);eligible_send(&mut wire,generation,&Wire::Events{events,live:false}).await?;},
                            Wire::Events{events,live}=>{ensure!(events.len()<=BATCH_SIZE,"Oversized event batch");let mut acks=Vec::new();let mut received=Vec::new();for event in events{if db.ingest(&event)?{received.push(event.clone());}acks.push(event.id);}drop(db);eligible_send(&mut wire,generation,&Wire::Ack{ids:acks}).await?;if live { for event in received{let _=self.changes.send(Change::Received(event,true));} } else if !received.is_empty(){let _=self.changes.send(Change::History);}},
                            Wire::Ack{ids}=>{ensure!(ids.len()<=BATCH_SIZE,"Oversized acknowledgement");for id in ids{db.acknowledge(&id,&peer)?;}drop(db);let _=self.changes.send(Change::History);},
                            Wire::Ping=>{drop(db);eligible_send(&mut wire,generation,&Wire::Pong).await?;},Wire::Pong=>{},Wire::Hello{..}=>anyhow::bail!("Repeated hello")
                        }
                    }
                }
            }Ok(())
        }.await;
        let mut peers = self.peers.lock().await;
        if peers
            .get(&peer)
            .is_some_and(|presence| presence.session_id == session_id)
        {
            peers.remove(&peer);
            drop(peers);
            self.changed();
        }
        result
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Wire {
    Hello {
        version: u16,
        id: String,
        profile: String,
        platform: String,
        signature: String,
    },
    Summary {
        digest: String,
    },
    InventoryRequest {
        offset: usize,
    },
    Inventory {
        offset: usize,
        entries: Vec<SequenceKey>,
    },
    Need {
        entries: Vec<SequenceKey>,
    },
    Events {
        events: Vec<Event>,
        live: bool,
    },
    Ack {
        ids: Vec<String>,
    },
    Ping,
    Pong,
}
async fn eligible_send<T>(
    wire: &mut Framed<T, LengthDelimitedCodec>,
    generation: u64,
    packet: &Wire,
) -> Result<()>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    ensure!(
        crate::platform::network_generation() == generation,
        "Network changed before export"
    );
    send(wire, packet).await
}
async fn send<T>(wire: &mut Framed<T, LengthDelimitedCodec>, packet: &Wire) -> Result<()>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let bytes = serde_json::to_vec(packet)?;
    if bytes.len() > FRAME_BYTES
        && let Wire::Events { events, live } = packet
        && events.len() > 1
    {
        for event in events {
            let frame = serde_json::to_vec(&Wire::Events {
                events: vec![event.clone()],
                live: *live,
            })?;
            ensure!(frame.len() <= FRAME_BYTES, "Event exceeds frame limit");
            timeout(Duration::from_secs(5), wire.send(frame.as_slice())).await??;
        }
        return Ok(());
    }
    ensure!(bytes.len() <= FRAME_BYTES, "Frame too large");
    timeout(Duration::from_secs(5), wire.send(bytes.as_slice())).await??;
    Ok(())
}
async fn receive<T>(wire: &mut Framed<T, LengthDelimitedCodec>) -> Result<Wire>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let bytes = wire.next().await.context("Peer disconnected")??;
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use tokio::net::{TcpListener, TcpStream};
    #[tokio::test]
    async fn view_subscription_survives_a_burst() {
        let (sender, mut receiver) = broadcast::channel(2);
        for _ in 0..8 {
            sender.send(Change::State).unwrap();
        }
        assert!(matches!(
            next_view_change(&mut receiver).await,
            Some(Change::History)
        ));
        assert!(matches!(
            next_view_change(&mut receiver).await,
            Some(Change::State)
        ));
        drop(sender);
        assert!(matches!(
            next_view_change(&mut receiver).await,
            Some(Change::State)
        ));
        assert!(next_view_change(&mut receiver).await.is_none());
    }

    #[tokio::test]
    async fn reconnect_requests_are_bounded_and_stop_with_the_app() {
        let (_directory, engine) = peer();
        let first = tokio::spawn({
            let engine = engine.clone();
            async move { engine.reconnect().await }
        });
        let reply = engine.reconnect_rx.lock().await.recv().await.unwrap();
        assert!(
            engine
                .reconnect()
                .await
                .unwrap_err()
                .to_string()
                .contains("Already reconnecting")
        );
        reply.send(Ok(())).unwrap();
        first.await.unwrap().unwrap();
        engine.stop.cancel();
        assert!(
            engine
                .reconnect()
                .await
                .unwrap_err()
                .to_string()
                .contains("shutting down")
        );
    }
    fn peer() -> (tempfile::TempDir, Arc<Engine>) {
        let dir = tempfile::tempdir().unwrap();
        let identity = Identity::load(dir.path(), true).unwrap();
        let mut db = Store::open(&dir.path().join("db.sqlite3")).unwrap();
        db.activate(Some("test-network"), "Test LAN").unwrap();
        db.set_meta("profile", "Test peer").unwrap();
        (dir, Engine::new(db, identity))
    }
    async fn connect(
        a: Arc<Engine>,
        b: Arc<Engine>,
        cancel: CancellationToken,
    ) -> Vec<tokio::task::JoinHandle<Result<()>>> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let cert = b.identity.fingerprint();
        let expected = b.identity.id();
        let child = cancel.clone();
        let server = tokio::spawn(async move {
            let (tcp, _) = listener.accept().await?;
            b.connect_test(tcp, true, None, String::new(), child).await
        });
        let client = tokio::spawn(async move {
            let tcp = TcpStream::connect(addr).await?;
            a.connect_test(tcp, false, Some(expected), cert, cancel)
                .await
        });
        vec![server, client]
    }
    async fn wait_count(peers: &[Arc<Engine>], count: usize) {
        timeout(Duration::from_secs(30), async {
            loop {
                let mut ready = true;
                for peer in peers {
                    let db = peer.store.lock().await;
                    if db.messages(&db.active, GENERAL, None, None).unwrap().len() != count {
                        ready = false;
                        break;
                    }
                }
                if ready {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(30)).await
            }
        })
        .await
        .expect("Peers failed to converge");
    }
    #[tokio::test]
    async fn old_session_cleanup_cannot_remove_a_reconnected_peer() {
        let (_a_dir, a) = peer();
        let (_b_dir, b) = peer();
        let old_cancel = CancellationToken::new();
        let old = connect(a.clone(), b.clone(), old_cancel.clone()).await;
        async fn established(a: &Engine, b: &Engine) {
            timeout(Duration::from_secs(5), async {
                while a.peers.lock().await.is_empty() || b.peers.lock().await.is_empty() {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
        }
        established(&a, &b).await;
        // Discovery replaced an endpoint before the old connection finished cleanup.
        a.peers.lock().await.clear();
        b.peers.lock().await.clear();
        let new_cancel = CancellationToken::new();
        let new = connect(a.clone(), b.clone(), new_cancel.clone()).await;
        established(&a, &b).await;
        old_cancel.cancel();
        for task in old {
            let _ = task.await;
        }
        assert_eq!(a.snapshot().await.unwrap().peers.len(), 1);
        assert_eq!(b.snapshot().await.unwrap().peers.len(), 1);
        new_cancel.cancel();
        for task in new {
            let _ = task.await;
        }
    }
    async fn mesh(count: usize) {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let nodes = (0..count).map(|_| peer()).collect::<Vec<_>>();
        let peers = nodes.iter().map(|(_, e)| e.clone()).collect::<Vec<_>>();
        let cancel = CancellationToken::new();
        let mut tasks = Vec::new();
        for i in 0..count {
            for j in i + 1..count {
                tasks.extend(connect(peers[i].clone(), peers[j].clone(), cancel.clone()).await);
            }
        }
        timeout(Duration::from_secs(20), async {
            loop {
                let mut all = true;
                for p in &peers {
                    if p.peers.lock().await.len() != count - 1 {
                        all = false;
                        break;
                    }
                }
                if all {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        for (i, p) in peers.iter().enumerate() {
            p.publish(Payload::Message {
                room: GENERAL.into(),
                content: format!("Concurrent message {i}"),
                mode: MessageMode::Text,
            })
            .await
            .unwrap();
        }
        wait_count(&peers, count).await;
        cancel.cancel();
        for task in tasks {
            let _ = task.await;
        }
        println!("{count} real TLS peers converged");
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn two_five_twenty_encrypted_peers() {
        for count in [2, 5, 20] {
            mesh(count).await;
        }
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn reconnect_relay_restart_and_context_change() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (_da, a) = peer();
        let (db_dir, b) = peer();
        let cancel = CancellationToken::new();
        let tasks = connect(a.clone(), b.clone(), cancel.clone()).await;
        a.publish(Payload::Message {
            room: GENERAL.into(),
            content: "recover from relay".into(),
            mode: MessageMode::Code,
        })
        .await
        .unwrap();
        wait_count(&[a.clone(), b.clone()], 1).await;
        cancel.cancel();
        for t in tasks {
            let _ = t.await;
        }
        drop(b);
        let identity = Identity::load(db_dir.path(), true).unwrap();
        let mut store = Store::open(&db_dir.path().join("db.sqlite3")).unwrap();
        store.activate(Some("test-network"), "Test LAN").unwrap();
        let b = Engine::new(store, identity);
        let (_dc, c) = peer();
        let cancel = CancellationToken::new();
        let tasks = connect(b.clone(), c.clone(), cancel.clone()).await;
        wait_count(std::slice::from_ref(&c), 1).await;
        cancel.cancel();
        for t in tasks {
            let _ = t.await;
        }
        c.store
            .lock()
            .await
            .activate(Some("different-network"), "Other LAN")
            .unwrap();
        assert!(c.store.lock().await.inventory(0).unwrap().is_empty());
        assert!(
            c.store
                .lock()
                .await
                .export(&a.store.lock().await.inventory(0).unwrap())
                .unwrap()
                .is_empty()
        );
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn interrupted_multi_page_sync_resumes_without_original_sender() {
        // This is a functional 600-event test, not a latency benchmark. Windows
        // debug signature checks and durable SQLite writes exceeded 20s in CI.
        let sync_deadline = Duration::from_secs(if cfg!(windows) { 60 } else { 20 });
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (_da, a) = peer();
        let (_db, b) = peer();
        for i in 0..600 {
            a.publish(Payload::Message {
                room: GENERAL.into(),
                content: format!("Historical event {i}"),
                mode: MessageMode::Text,
            })
            .await
            .unwrap();
        }
        let cancel = CancellationToken::new();
        let tasks = connect(a.clone(), b.clone(), cancel.clone()).await;
        timeout(Duration::from_secs(10), async {
            loop {
                if b.store.lock().await.inventory(0).unwrap().len() >= 8 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        cancel.cancel();
        for t in tasks {
            let _ = t.await;
        }
        let cancel = CancellationToken::new();
        let tasks = connect(a.clone(), b.clone(), cancel.clone()).await;
        timeout(sync_deadline, async {
            loop {
                if a.store.lock().await.digest().unwrap() == b.store.lock().await.digest().unwrap()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        cancel.cancel();
        for t in tasks {
            let _ = t.await;
        }
        drop(a);
        let (_dc, c) = peer();
        let cancel = CancellationToken::new();
        let tasks = connect(b.clone(), c.clone(), cancel.clone()).await;
        timeout(sync_deadline, async {
            loop {
                if b.store.lock().await.digest().unwrap() == c.store.lock().await.digest().unwrap()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(c.store.lock().await.inventory(0).unwrap().len(), 512);
        assert_eq!(c.store.lock().await.inventory(512).unwrap().len(), 88);
        cancel.cancel();
        for t in tasks {
            let _ = t.await;
        }
    }
    #[tokio::test]
    async fn escaped_maximum_messages_split_into_bounded_frames() {
        let (_dir, engine) = peer();
        let mut events = Vec::new();
        for _ in 0..BATCH_SIZE {
            events.push(
                engine
                    .publish(Payload::Message {
                        room: GENERAL.into(),
                        content: "\0".repeat(MESSAGE_BYTES),
                        mode: MessageMode::Code,
                    })
                    .await
                    .unwrap(),
            );
        }
        let (a, b) = tokio::io::duplex(FRAME_BYTES * 2);
        let mut sender = Framed::new(
            a,
            LengthDelimitedCodec::builder()
                .max_frame_length(FRAME_BYTES)
                .new_codec(),
        );
        let task = tokio::spawn(async move {
            send(
                &mut sender,
                &Wire::Events {
                    events,
                    live: false,
                },
            )
            .await
            .unwrap();
        });
        let mut receiver = Framed::new(
            b,
            LengthDelimitedCodec::builder()
                .max_frame_length(FRAME_BYTES)
                .new_codec(),
        );
        let mut count = 0;
        while count < BATCH_SIZE {
            let Wire::Events { events, .. } = receive(&mut receiver).await.unwrap() else {
                panic!("Expected event frame")
            };
            for event in events {
                assert_eq!(
                    match event.payload {
                        Payload::Message { content, .. } => content.len(),
                        _ => 0,
                    },
                    MESSAGE_BYTES
                );
                count += 1;
            }
        }
        task.await.unwrap();
    }
    #[tokio::test]
    async fn oversized_length_prefix_is_rejected_before_payload_allocation() {
        use tokio::io::AsyncWriteExt;
        let (mut a, b) = tokio::io::duplex(32);
        a.write_all(&((FRAME_BYTES + 1) as u32).to_be_bytes())
            .await
            .unwrap();
        let mut receiver = Framed::new(
            b,
            LengthDelimitedCodec::builder()
                .max_frame_length(FRAME_BYTES)
                .new_codec(),
        );
        assert!(receive(&mut receiver).await.is_err());
    }
    #[tokio::test]
    async fn wrong_tls_pin_is_rejected() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (_d, a) = peer();
        let (_e, b) = peer();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            b.connect_test(tcp, true, None, String::new(), CancellationToken::new())
                .await
        });
        let tcp = TcpStream::connect(addr).await.unwrap();
        assert!(
            a.connect_test(tcp, false, None, "00".repeat(32), CancellationToken::new())
                .await
                .is_err()
        );
        let _ = task.await;
    }
}
