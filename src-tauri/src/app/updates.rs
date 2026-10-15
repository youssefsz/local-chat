use super::{DesktopState, Reply, error};
use crate::updates::{Channel, download_url, due};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};
use tokio::sync::{Notify, Semaphore};

const MAX_PACKAGE: u64 = 256 * 1024 * 1024;
#[derive(Clone, Serialize)]
pub struct Release {
    version: String,
    notes: String,
}
impl From<&Update> for Release {
    fn from(u: &Update) -> Self {
        Self {
            version: u.version.clone(),
            notes: u
                .body
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(16_000)
                .collect(),
        }
    }
}
enum Phase {
    Idle,
    Checking,
    Current,
    Available(Update),
    Downloading {
        update: Update,
        downloaded: u64,
        total: Option<u64>,
    },
    Ready {
        update: Update,
        path: PathBuf,
    },
    Installing,
    Failed(String),
}
#[derive(Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum View {
    Idle,
    Checking,
    Current,
    Available {
        release: Release,
    },
    Downloading {
        release: Release,
        downloaded: u64,
        total: Option<u64>,
    },
    Ready {
        release: Release,
    },
    Installing,
    Failed {
        message: String,
    },
}
#[derive(Clone, Serialize)]
pub struct Snapshot {
    version: String,
    channel: Channel,
    automatic: bool,
    last_checked: u64,
    dismissed: String,
    in_app: bool,
    #[serde(flatten)]
    phase: View,
}
pub struct Service {
    phase: Mutex<Phase>,
    gate: Semaphore,
    data: PathBuf,
}
impl Service {
    pub fn new(data: PathBuf) -> Self {
        Self {
            phase: Mutex::new(Phase::Idle),
            gate: Semaphore::new(1),
            data,
        }
    }
    fn set(&self, phase: Phase) {
        *self.phase.lock().expect("update state") = phase;
    }
    async fn emit(&self, app: &tauri::AppHandle, engine: &DesktopState) {
        if let Ok(snapshot) = self.snapshot(app, engine).await {
            let _ = app.emit("local-chat:updates", snapshot);
        }
    }
    async fn snapshot(&self, app: &tauri::AppHandle, engine: &DesktopState) -> Result<Snapshot> {
        let db = engine.store.lock().await;
        let version = app.package_info().version.to_string();
        let channel = Channel::saved(&db.meta("updates_channel", "")?, &version);
        let phase = match &*self.phase.lock().expect("update state") {
            Phase::Idle => View::Idle,
            Phase::Checking => View::Checking,
            Phase::Current => View::Current,
            Phase::Available(u) => View::Available { release: u.into() },
            Phase::Downloading {
                update,
                downloaded,
                total,
            } => View::Downloading {
                release: update.into(),
                downloaded: *downloaded,
                total: *total,
            },
            Phase::Ready { update, .. } => View::Ready {
                release: update.into(),
            },
            Phase::Installing => View::Installing,
            Phase::Failed(message) => View::Failed {
                message: message.clone(),
            },
        };
        #[cfg(target_os = "linux")]
        let in_app = app.env().appimage.is_some();
        #[cfg(not(target_os = "linux"))]
        let in_app = true;
        Ok(Snapshot {
            version,
            channel,
            automatic: db.meta(
                "updates_automatic",
                if cfg!(feature = "test-profile") {
                    "false"
                } else {
                    "true"
                },
            )? == "true",
            last_checked: db
                .meta(&format!("updates_checked_{}", channel.name()), "0")?
                .parse()
                .unwrap_or(0),
            dismissed: if (crate::domain::now().max(0) as u64 / 1000)
                .saturating_sub(db.meta("updates_dismissed_at", "0")?.parse().unwrap_or(0))
                < 86_400
            {
                db.meta("updates_dismissed", "")?
            } else {
                String::new()
            },
            in_app,
            phase,
        })
    }
    fn updater(
        &self,
        app: &tauri::AppHandle,
        channel: Channel,
    ) -> Result<tauri_plugin_updater::Updater> {
        let endpoint = format!(
            "https://raw.githubusercontent.com/youssefsz/local-chat/updates/{}.json",
            channel.name()
        );
        let builder = app
            .updater_builder()
            .endpoints(vec![endpoint.parse()?])?
            .timeout(Duration::from_secs(15));
        #[cfg(feature = "test-profile")]
        let builder = if let Some(options) = test_options(&self.data)? {
            builder
                .endpoints(vec![options.endpoint.parse()?])?
                .pubkey(options.pubkey)
        } else {
            builder
        };
        Ok(builder.build()?)
    }
    fn validate_candidate(
        &self,
        app: &tauri::AppHandle,
        channel: Channel,
        u: &mut Update,
    ) -> Result<()> {
        u.timeout = Some(Duration::from_secs(300));
        ensure!(
            channel.accepts(&u.version),
            "The update does not belong to this channel"
        );
        ensure!(
            semver::Version::parse(&u.version)? > app.package_info().version,
            "The update is not newer"
        );
        let allowed = download_url(&u.version).is_some_and(|url| url == u.download_url.as_str());
        #[cfg(feature = "test-profile")]
        let allowed = allowed
            || test_options(&self.data)?.is_some_and(|_| {
                u.download_url.scheme() == "http" && u.download_url.host_str() == Some("127.0.0.1")
            });
        ensure!(
            allowed,
            "The update download is not an official Local Chat release"
        );
        ensure!(
            serde_json::to_vec(&u.raw_json)?.len() <= 65_536,
            "Oversized update metadata"
        );
        Ok(())
    }
    async fn restore(&self, app: &tauri::AppHandle, engine: &DesktopState) {
        let Ok(_permit) = self.gate.try_acquire() else {
            return;
        };
        if !matches!(*self.phase.lock().expect("update state"), Phase::Idle) {
            return;
        }
        let restored = async {
            let settings = self.snapshot(app, engine).await?;
            let directory = self.data.join("updates");
            let metadata = std::fs::read(directory.join("pending.json"))?;
            ensure!(metadata.len() <= 65_536, "Oversized cached metadata");
            let mut update = self
                .updater(app, settings.channel)?
                .restore_update(serde_json::from_slice(&metadata)?)?
                .ok_or_else(|| anyhow::anyhow!("Cached update is no longer newer"))?;
            self.validate_candidate(app, settings.channel, &mut update)?;
            let path = directory.join("package.bin");
            let verifier = update.clone();
            let file = path.clone();
            tauri::async_runtime::spawn_blocking(move || -> Result<()> {
                ensure!(
                    std::fs::metadata(&file)?.len() <= MAX_PACKAGE,
                    "Oversized cached update"
                );
                verifier.verify(&std::fs::read(file)?)?;
                Ok(())
            })
            .await??;
            Ok::<_, anyhow::Error>((update, path))
        }
        .await;
        if let Ok((update, path)) = restored {
            self.set(Phase::Ready { update, path });
            self.emit(app, engine).await;
        }
    }
    async fn check(
        &self,
        app: &tauri::AppHandle,
        engine: &DesktopState,
        manual: bool,
    ) -> Result<()> {
        // A manual check can arrive before the delayed background task.
        self.restore(app, engine).await;
        let Ok(_permit) = self.gate.try_acquire() else {
            return Ok(());
        };
        if matches!(
            *self.phase.lock().expect("update state"),
            Phase::Ready { .. }
        ) {
            self.emit(app, engine).await;
            return Ok(());
        }
        let before = self.snapshot(app, engine).await?;
        let now = crate::domain::now().max(0) as u64 / 1000;
        {
            let db = engine.store.lock().await;
            let attempted = db
                .meta(&format!("updates_attempt_{}", before.channel.name()), "0")?
                .parse()
                .unwrap_or(0);
            if !manual && (!before.automatic || !due(now, before.last_checked, attempted)) {
                return Ok(());
            }
            db.set_meta(
                &format!("updates_attempt_{}", before.channel.name()),
                &now.to_string(),
            )?;
        }
        self.set(Phase::Checking);
        self.emit(app, engine).await;
        let result = async {
            let mut update = self.updater(app, before.channel)?.check().await?;
            if let Some(u) = &mut update {
                self.validate_candidate(app, before.channel, u)?;
            }
            engine.store.lock().await.set_meta(
                &format!("updates_checked_{}", before.channel.name()),
                &now.to_string(),
            )?;
            Ok::<_, anyhow::Error>(update)
        }
        .await;
        match result {
            Ok(Some(u)) => self.set(Phase::Available(u)),
            Ok(None) => self.set(Phase::Current),
            Err(_) => self.set(Phase::Failed(
                "Couldn’t check for updates. Check your internet connection and try again.".into(),
            )),
        }
        self.emit(app, engine).await;
        Ok(())
    }
    async fn download(&self, app: &tauri::AppHandle, engine: &DesktopState) -> Result<()> {
        let _permit = self
            .gate
            .try_acquire()
            .map_err(|_| anyhow::anyhow!("An update operation is already running"))?;
        let update = match &*self.phase.lock().expect("update state") {
            Phase::Available(u) => u.clone(),
            _ => anyhow::bail!("Check for an update first"),
        };
        ensure!(
            self.snapshot(app, engine).await?.in_app,
            "Use the Debian download to update this installation"
        );
        self.set(Phase::Downloading {
            update: update.clone(),
            downloaded: 0,
            total: None,
        });
        self.emit(app, engine).await;
        let limit = Arc::new(Notify::new());
        let too_large = limit.clone();
        let mut downloaded = 0u64;
        let mut last_event = Instant::now() - Duration::from_secs(1);
        let result = tokio::select! {
            biased;
            _=limit.notified()=>Err(anyhow::anyhow!("Update package exceeds the size limit")),
            result=update.download(|chunk,total| {
                downloaded=downloaded.saturating_add(chunk as u64);
                if downloaded>MAX_PACKAGE || total.is_some_and(|n|n>MAX_PACKAGE) {too_large.notify_one();return;}
                self.set(Phase::Downloading {update:update.clone(),downloaded,total});
                if last_event.elapsed()>=Duration::from_millis(150) {
                    last_event=Instant::now();
                    let _=app.emit("local-chat:update-progress",(downloaded,total));
                }
            },||{})=>result.map_err(anyhow::Error::from),
        };
        match result { Ok(bytes) if bytes.len() as u64 <= MAX_PACKAGE => {
                let directory=self.data.join("updates");let metadata=serde_json::to_vec(&update.raw_json)?;
                let cached=tauri::async_runtime::spawn_blocking(move|| -> Result<PathBuf> {
                    std::fs::create_dir_all(&directory)?;
                    let path=directory.join("package.bin");let temporary=directory.join("package.part");
                    std::fs::write(&temporary,bytes)?;
                    if path.exists(){std::fs::remove_file(&path)?;}
                    std::fs::rename(temporary,&path)?;
                    std::fs::write(directory.join("pending.json"),metadata)?;Ok(path)
                }).await;
                match cached {Ok(Ok(path))=>self.set(Phase::Ready{update,path}),_=>self.set(Phase::Failed("Couldn’t save the downloaded update. Check free disk space and try again.".into()))}
            }, _=>self.set(Phase::Failed("The update couldn’t be downloaded or verified. Your installed app is unchanged. Check for updates to try again.".into())) }
        self.emit(app, engine).await;
        Ok(())
    }
}
#[tauri::command]
pub async fn update_status(
    app: tauri::AppHandle,
    engine: State<'_, DesktopState>,
    service: State<'_, Service>,
) -> Reply<Snapshot> {
    service.snapshot(&app, &engine).await.map_err(error)
}
#[tauri::command]
pub async fn check_updates(
    app: tauri::AppHandle,
    engine: State<'_, DesktopState>,
    service: State<'_, Service>,
) -> Reply<()> {
    service.check(&app, &engine, true).await.map_err(error)
}
#[tauri::command]
pub async fn download_update(
    app: tauri::AppHandle,
    engine: State<'_, DesktopState>,
    service: State<'_, Service>,
) -> Reply<()> {
    service.download(&app, &engine).await.map_err(error)
}
#[tauri::command]
pub async fn update_preferences(
    app: tauri::AppHandle,
    engine: State<'_, DesktopState>,
    service: State<'_, Service>,
    automatic: bool,
    channel: Channel,
) -> Reply<()> {
    let _permit = service.gate.try_acquire().map_err(error)?;
    let ready = matches!(
        *service.phase.lock().expect("update state"),
        Phase::Ready { .. }
    );
    let old = service
        .snapshot(&app, &engine)
        .await
        .map_err(error)?
        .channel;
    if ready && old != channel {
        return Err("Install the downloaded update before changing channels".into());
    }
    {
        let db = engine.store.lock().await;
        db.set_meta(
            "updates_automatic",
            if automatic { "true" } else { "false" },
        )
        .map_err(error)?;
        db.set_meta("updates_channel", channel.name())
            .map_err(error)?;
    }
    if old != channel {
        service.set(Phase::Idle);
    }
    service.emit(&app, &engine).await;
    Ok(())
}
#[tauri::command]
pub async fn dismiss_update(
    app: tauri::AppHandle,
    engine: State<'_, DesktopState>,
    service: State<'_, Service>,
) -> Reply<()> {
    let version = match &*service.phase.lock().expect("update state") {
        Phase::Available(u) | Phase::Ready { update: u, .. } => Some(u.version.clone()),
        _ => None,
    };
    if let Some(v) = version {
        engine
            .store
            .lock()
            .await
            .set_meta("updates_dismissed", &v)
            .map_err(error)?;
        engine
            .store
            .lock()
            .await
            .set_meta(
                "updates_dismissed_at",
                &(crate::domain::now().max(0) as u64 / 1000).to_string(),
            )
            .map_err(error)?;
    }
    service.emit(&app, &engine).await;
    Ok(())
}
#[tauri::command]
pub async fn install_update(
    app: tauri::AppHandle,
    engine: State<'_, DesktopState>,
    service: State<'_, Service>,
) -> Reply<()> {
    let _permit = service.gate.try_acquire().map_err(error)?;
    let phase = std::mem::replace(
        &mut *service.phase.lock().expect("update state"),
        Phase::Installing,
    );
    let Phase::Ready { update, path } = phase else {
        service.set(phase);
        return Err("Download and verify an update first".into());
    };
    service.emit(&app, &engine).await;
    // Installation performs no database changes. The frontend awaits its draft save first.
    let installed = tauri::async_runtime::spawn_blocking(move || -> Result<()> {
        ensure!(
            std::fs::metadata(&path)?.len() <= MAX_PACKAGE,
            "Oversized cached update"
        );
        update.install(std::fs::read(path)?)?;
        Ok(())
    })
    .await;
    if !matches!(installed, Ok(Ok(()))) {
        service.set(Phase::Failed("Couldn’t install the update. Check that you have permission to update this app, then try again.".into()));
        service.emit(&app, &engine).await;
        return Ok(());
    }
    app.restart();
}
#[tauri::command]
pub async fn open_update_download(app: tauri::AppHandle, service: State<'_, Service>) -> Reply<()> {
    use tauri_plugin_opener::OpenerExt;
    let version = match &*service.phase.lock().expect("update state") {
        Phase::Available(u) => u.version.clone(),
        _ => return Err("Check for an update first".into()),
    };
    app.opener()
        .open_url(
            format!("https://github.com/youssefsz/local-chat/releases/tag/v{version}"),
            None::<&str>,
        )
        .map_err(error)
}
pub fn start_background(app: tauri::AppHandle, engine: DesktopState) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        app.state::<Service>().restore(&app, &engine).await;
        loop {
            if engine.stop.is_cancelled() {
                break;
            }
            let _ = app.state::<Service>().check(&app, &engine, false).await;
            tokio::select! {_=engine.stop.cancelled()=>break,_=tokio::time::sleep(Duration::from_secs(3600))=>{}}
        }
    });
}
#[cfg(feature = "test-profile")]
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TestOptions {
    endpoint: String,
    pubkey: String,
}
#[cfg(feature = "test-profile")]
fn test_options(data: &std::path::Path) -> Result<Option<TestOptions>> {
    let path = data.join("update-test.json");
    if !path.exists() {
        return Ok(None);
    }
    let options: TestOptions = serde_json::from_slice(&std::fs::read(path)?)?;
    let url: tauri::Url = options.endpoint.parse()?;
    ensure!(
        url.scheme() == "http" && url.host_str() == Some("127.0.0.1") && url.port().is_some(),
        "Only a loopback update fixture is allowed"
    );
    Ok(Some(options))
}
