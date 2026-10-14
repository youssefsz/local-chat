use crate::domain::*;
use anyhow::{Result, ensure};
use ed25519_dalek::SigningKey;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

pub struct Store {
    db: Connection,
    digest_cache: std::cell::RefCell<Option<(String, String)>>,
    pub active: String,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let db = Connection::open(path)?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.execute_batch(include_str!("../../migrations/001.sql"))?;
        Ok(Self {
            db,
            digest_cache: std::cell::RefCell::new(None),
            active: String::new(),
        })
    }
    pub fn meta(&self, key: &str, default: &str) -> Result<String> {
        Ok(self
            .db
            .query_row("SELECT value FROM meta WHERE key=?1", [key], |r| r.get(0))
            .optional()?
            .unwrap_or_else(|| default.into()))
    }
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.db.execute(
            "INSERT INTO meta VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
    pub fn activate(&mut self, evidence: Option<&str>, label: &str) -> Result<String> {
        let existing = if let Some(evidence) = evidence {
            self.db
                .query_row(
                    "SELECT id FROM contexts WHERE evidence=?1",
                    [evidence],
                    |r| r.get::<_, String>(0),
                )
                .optional()?
        } else {
            None
        };
        let id = existing.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        self.db.execute(
            "INSERT OR IGNORE INTO contexts VALUES(?1,?2,?3,?4,?5)",
            params![id, evidence, uuid::Uuid::new_v4().to_string(), label, now()],
        )?;
        self.db.execute("INSERT OR IGNORE INTO rooms VALUES(?1,'general','General','A little of everything, for everyone.','system',0)",[&id])?;
        self.digest_cache.replace(None);
        self.active = id.clone();
        Ok(id)
    }
    pub fn contexts(&self) -> Result<Vec<Context>> {
        Ok(self
            .db
            .prepare("SELECT id,label FROM contexts ORDER BY created DESC")?
            .query_map([], |r| {
                let id: String = r.get(0)?;
                Ok(Context {
                    id: id.clone(),
                    label: r.get(1)?,
                    current: id == self.active,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }
    pub fn rooms(&self, context: &str) -> Result<Vec<Room>> {
        Ok(self.db.prepare("SELECT r.id,r.name,r.description,r.owner,r.archived,COALESCE(l.muted,0),(SELECT COUNT(*) FROM messages m WHERE m.context=r.context AND m.room=r.id AND m.logical>COALESCE(l.read_logical,0) AND m.author<>?2) FROM rooms r LEFT JOIN room_local l ON l.context=r.context AND l.room=r.id WHERE r.context=?1 ORDER BY r.id<>'general',r.archived,r.name,r.id")?.query_map(params![context,self.meta("identity","")?],|r|Ok(Room{id:r.get(0)?,context:context.into(),name:r.get(1)?,description:r.get(2)?,owner:r.get(3)?,archived:r.get(4)?,muted:r.get(5)?,unread:r.get(6)?}))?.collect::<rusqlite::Result<_>>()?)
    }
    pub fn create(&mut self, key: &SigningKey, payload: Payload) -> Result<Event> {
        payload.validate()?;
        let room = payload.room();
        if !matches!(payload, Payload::CreateRoom { .. }) {
            let (owner, archived): (String, bool) = self.db.query_row(
                "SELECT owner,archived FROM rooms WHERE context=?1 AND id=?2",
                params![self.active, room],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            match &payload {
                Payload::Message { .. } => ensure!(!archived, "This room is archived"),
                _ => ensure!(
                    owner == hex::encode(key.verifying_key().as_bytes()),
                    "Only this room's creator can manage it"
                ),
            }
        }
        let namespace: String = self.db.query_row(
            "SELECT namespace FROM contexts WHERE id=?1",
            [&self.active],
            |r| r.get(0),
        )?;
        let sequence = self.meta("sequence", "0")?.parse::<i64>()? + 1;
        let logical =
            self.db
                .query_row("SELECT COALESCE(MAX(logical),0)+1 FROM events", [], |r| {
                    r.get::<_, i64>(0)
                })?;
        let event = Event::sign(
            key,
            namespace,
            self.meta("profile", "You")?,
            sequence,
            logical,
            payload,
        )?;
        let tx = self.db.transaction()?;
        tx.execute("INSERT INTO meta VALUES('sequence',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[sequence.to_string()])?;
        Self::insert(&tx, &self.active, &event, true)?;
        Self::materialize(&tx, &self.active, event.payload.room())?;
        if matches!(event.payload, Payload::Message { .. }) {
            tx.execute(
                "DELETE FROM drafts WHERE context=?1 AND room=?2",
                params![self.active, event.payload.room()],
            )?;
        }
        tx.commit()?;
        self.digest_cache.replace(None);
        Ok(event)
    }
    fn insert(db: &Connection, context: &str, event: &Event, local: bool) -> Result<bool> {
        event.verify()?;
        if let Some((id,stored_context))=db.query_row("SELECT id,context FROM events WHERE id=?1 OR (namespace=?2 AND author=?3 AND sequence=?4)",params![event.id,event.namespace,event.author,event.sequence],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).optional()? {
            ensure!(id==event.id,"Author sequence equivocation");
            // Duplicate receipts never move an event into a new export context.
            let _ = stored_context;
            return Ok(false)
        }
        let kind = match event.payload {
            Payload::CreateRoom { .. } => "create",
            Payload::RenameRoom { .. } => "rename",
            Payload::ArchiveRoom { .. } => "archive",
            Payload::Message { .. } => "message",
        };
        db.execute(
            "INSERT INTO events VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                event.id,
                context,
                event.namespace,
                event.author,
                event.sequence,
                event.logical,
                event.payload.room(),
                kind,
                serde_json::to_string(event)?
            ],
        )?;
        if let Payload::Message {
            room,
            content,
            mode,
        } = &event.payload
        {
            db.execute(
                "INSERT INTO messages VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    event.id,
                    context,
                    room,
                    event.author,
                    event.profile,
                    content,
                    if *mode == MessageMode::Code {
                        "code"
                    } else {
                        "text"
                    },
                    event.logical,
                    event.timestamp,
                    if local { "waiting" } else { "received" }
                ],
            )?;
        }
        Ok(true)
    }
    pub fn ingest(&mut self, event: &Event) -> Result<bool> {
        let tx = self.db.transaction()?;
        let inserted = Self::insert(&tx, &self.active, event, false)?;
        if inserted {
            Self::materialize(&tx, &self.active, event.payload.room())?;
        }
        tx.commit()?;
        if inserted {
            self.digest_cache.replace(None);
        }
        Ok(inserted)
    }
    fn materialize(db: &Connection, context: &str, room: &str) -> Result<()> {
        if room == GENERAL {
            return Ok(());
        }
        let rows=db.prepare("SELECT json FROM events WHERE context=?1 AND room=?2 AND kind<>'message' ORDER BY logical,author,id")?.query_map(params![context,room],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
        let mut current: Option<Room> = None;
        for json in rows {
            let e: Event = serde_json::from_str(&json)?;
            match e.payload {
                Payload::CreateRoom {
                    name, description, ..
                } if current.is_none() => {
                    current = Some(Room {
                        id: room.into(),
                        context: context.to_owned(),
                        name,
                        description,
                        owner: e.author,
                        archived: false,
                        muted: false,
                        unread: 0,
                    });
                }
                Payload::RenameRoom {
                    name, description, ..
                } => {
                    if let Some(r) = &mut current
                        && r.owner == e.author
                    {
                        r.name = name;
                        r.description = description
                    }
                }
                Payload::ArchiveRoom { archived, .. } => {
                    if let Some(r) = &mut current
                        && r.owner == e.author
                    {
                        r.archived = archived
                    }
                }
                _ => {}
            }
        }
        if let Some(r) = current {
            db.execute("INSERT INTO rooms VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(context,id) DO UPDATE SET name=excluded.name,description=excluded.description,owner=excluded.owner,archived=excluded.archived",params![r.context,r.id,r.name,r.description,r.owner,r.archived])?;
        }
        Ok(())
    }
    pub fn acknowledge(&self, id: &str, peer: &str) -> Result<()> {
        self.db.execute(
            "INSERT OR IGNORE INTO receipts SELECT id,?2 FROM events WHERE id=?1 AND context=?3",
            params![id, peer, self.active],
        )?;
        self.db.execute("UPDATE messages SET delivery='shared' WHERE id=?1 AND context=?2 AND delivery='waiting'",params![id,self.active])?;
        Ok(())
    }
    pub fn digest(&self) -> Result<String> {
        if let Some((context, digest)) = &*self.digest_cache.borrow()
            && context == &self.active
        {
            return Ok(digest.clone());
        }

        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        let mut q = self
            .db
            .prepare("SELECT id FROM events WHERE context=?1 ORDER BY id")?;
        for id in q.query_map([&self.active], |r| r.get::<_, String>(0))? {
            hash.update(id?.as_bytes());
        }
        let digest = hex::encode(hash.finalize());
        self.digest_cache
            .replace(Some((self.active.clone(), digest.clone())));
        Ok(digest)
    }
    pub fn inventory(&self, offset: usize) -> Result<Vec<(String, String, i64)>> {
        Ok(self.db.prepare("SELECT namespace,author,sequence FROM events WHERE context=?1 ORDER BY namespace,author,sequence LIMIT 512 OFFSET ?2")?.query_map(params![self.active,i64::try_from(offset)?],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect::<rusqlite::Result<_>>()?)
    }
    pub fn missing(&self, entries: &[(String, String, i64)]) -> Result<Vec<(String, String, i64)>> {
        let mut missing = Vec::new();
        let mut q = self
            .db
            .prepare("SELECT 1 FROM events WHERE namespace=?1 AND author=?2 AND sequence=?3")?;
        for (ns, a, s) in entries {
            if q.query_row(params![ns, a, s], |_| Ok(()))
                .optional()?
                .is_none()
            {
                missing.push((ns.clone(), a.clone(), *s));
            }
        }
        Ok(missing)
    }
    pub fn export(&self, entries: &[(String, String, i64)]) -> Result<Vec<Event>> {
        ensure!(entries.len() <= BATCH_SIZE, "Batch too large");
        let mut result = Vec::new();
        let mut q=self.db.prepare("SELECT json FROM events WHERE context=?1 AND namespace=?2 AND author=?3 AND sequence=?4")?;
        for (ns, a, s) in entries {
            if let Some(json) = q
                .query_row(params![self.active, ns, a, s], |r| r.get::<_, String>(0))
                .optional()?
            {
                result.push(serde_json::from_str(&json)?)
            }
        }
        Ok(result)
    }
    fn message_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Message> {
        Ok(Message {
            id: r.get(0)?,
            room: r.get(1)?,
            author: r.get(2)?,
            profile: r.get(3)?,
            content: r.get(4)?,
            mode: if r.get::<_, String>(5)? == "code" {
                MessageMode::Code
            } else {
                MessageMode::Text
            },
            logical: r.get(6)?,
            timestamp: r.get(7)?,
            delivery: r.get(8)?,
        })
    }
    pub fn messages(
        &self,
        context: &str,
        room: &str,
        before: Option<String>,
        around: Option<String>,
    ) -> Result<Vec<Message>> {
        if let Some(id) = around {
            let pivot: i64 = self.db.query_row(
                "SELECT logical FROM messages WHERE context=?1 AND room=?2 AND id=?3",
                params![context, room, id],
                |r| r.get(0),
            )?;
            let sql = "WITH earlier AS (SELECT id,room,author,profile,content,mode,logical,timestamp,delivery FROM messages WHERE context=?1 AND room=?2 AND (logical,id)<=(?3,?4) ORDER BY logical DESC,id DESC LIMIT 40), later AS (SELECT id,room,author,profile,content,mode,logical,timestamp,delivery FROM messages WHERE context=?1 AND room=?2 AND (logical,id)>(?3,?4) ORDER BY logical,id LIMIT 40) SELECT * FROM earlier UNION ALL SELECT * FROM later ORDER BY logical,id";
            return Ok(self
                .db
                .prepare(sql)?
                .query_map(params![context, room, pivot, id], Self::message_row)?
                .collect::<rusqlite::Result<_>>()?);
        }
        let pivot = before
            .as_ref()
            .map(|id| {
                self.db.query_row(
                    "SELECT logical FROM messages WHERE context=?1 AND room=?2 AND id=?3",
                    params![context, room, id],
                    |r| r.get::<_, i64>(0),
                )
            })
            .transpose()?;
        let mut result=self.db.prepare("SELECT id,room,author,profile,content,mode,logical,timestamp,delivery FROM messages WHERE context=?1 AND room=?2 AND (?3 IS NULL OR (logical,id)<(?3,?4)) ORDER BY logical DESC,id DESC LIMIT ?5")?.query_map(params![context,room,pivot,before.unwrap_or_default(),PAGE_SIZE as i64],Self::message_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
        result.reverse();
        Ok(result)
    }
    pub fn page(
        &self,
        context: &str,
        room: &str,
        before: Option<String>,
        around: Option<String>,
    ) -> Result<MessagePage> {
        let items = self.messages(context, room, before, around)?;
        let has_older = if let Some(first) = items.first() {
            self.db.query_row("SELECT EXISTS(SELECT 1 FROM messages WHERE context=?1 AND room=?2 AND (logical,id)<(?3,?4))",params![context,room,first.logical,first.id],|r|r.get(0))?
        } else {
            false
        };
        let has_newer = if let Some(last) = items.last() {
            self.db.query_row("SELECT EXISTS(SELECT 1 FROM messages WHERE context=?1 AND room=?2 AND (logical,id)>(?3,?4))",params![context,room,last.logical,last.id],|r|r.get(0))?
        } else {
            false
        };
        Ok(MessagePage {
            items,
            has_older,
            has_newer,
        })
    }
    pub fn search(&self, query: &str, room: Option<&str>) -> Result<Vec<SearchResult>> {
        self.search_context(query, room, &self.active)
    }
    pub fn search_context(
        &self,
        query: &str,
        room: Option<&str>,
        context: &str,
    ) -> Result<Vec<SearchResult>> {
        let terms = query
            .split_whitespace()
            .take(12)
            .map(|s| format!("\"{}\"", s.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" AND ");
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        Ok(self.db.prepare("SELECT m.id,m.room,m.author,m.profile,m.content,m.mode,m.logical,m.timestamp,m.delivery,m.context,r.name,snippet(message_fts,0,'','','…',32) FROM message_fts f JOIN messages m ON m.rowid=f.rowid JOIN rooms r ON r.context=m.context AND r.id=m.room WHERE message_fts MATCH ?1 AND (?2 IS NULL OR (m.room=?2 AND m.context=?3)) ORDER BY rank LIMIT 50")?.query_map(params![terms,room,context],|r|{let context:String=r.get(9)?;Ok(SearchResult{excerpt:r.get(11)?,message:Self::message_row(r)?,current:context==self.active,context,room_name:r.get(10)?})})?.collect::<rusqlite::Result<_>>()?)
    }
    pub fn draft(&self, context: &str, room: &str) -> Result<Draft> {
        Ok(self
            .db
            .query_row(
                "SELECT content,code FROM drafts WHERE context=?1 AND room=?2",
                params![context, room],
                |r| {
                    Ok(Draft {
                        content: r.get(0)?,
                        code: r.get(1)?,
                    })
                },
            )
            .optional()?
            .unwrap_or_default())
    }
    pub fn save_draft(&self, context: &str, room: &str, draft: &Draft) -> Result<()> {
        ensure!(
            context == self.active,
            "Previous network history is read-only"
        );
        ensure!(draft.content.len() <= MESSAGE_BYTES, "Draft is too large");
        self.db.execute("INSERT INTO drafts VALUES(?1,?2,?3,?4) ON CONFLICT(context,room) DO UPDATE SET content=excluded.content,code=excluded.code",params![context,room,draft.content,draft.code])?;
        Ok(())
    }
    pub fn room_local(
        &self,
        context: &str,
        room: &str,
        muted: Option<bool>,
        read: Option<i64>,
        scroll: Option<f64>,
    ) -> Result<f64> {
        self.db.execute(
            "INSERT OR IGNORE INTO room_local(context,room) VALUES(?1,?2)",
            params![context, room],
        )?;
        self.db.execute("UPDATE room_local SET muted=COALESCE(?3,muted),read_logical=MAX(read_logical,COALESCE(?4,read_logical)),scroll=COALESCE(?5,scroll) WHERE context=?1 AND room=?2",params![context,room,muted,read,scroll])?;
        Ok(self.db.query_row(
            "SELECT scroll FROM room_local WHERE context=?1 AND room=?2",
            params![context, room],
            |r| r.get(0),
        )?)
    }
    pub fn view(&self, context: &str, room: &str) -> Result<serde_json::Value> {
        Ok(serde_json::from_str(
            &self.meta(&format!("view:{context}:{room}"), "null")?,
        )?)
    }
    pub fn save_view(&self, context: &str, room: &str, anchor: &str, offset: f64) -> Result<()> {
        ensure!(offset.is_finite(), "Invalid scroll offset");
        let exists = self
            .db
            .query_row(
                "SELECT 1 FROM messages WHERE context=?1 AND room=?2 AND id=?3",
                params![context, room, anchor],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        ensure!(exists, "Scroll anchor no longer exists");
        self.set_meta(
            &format!("view:{context}:{room}"),
            &serde_json::to_string(&serde_json::json!({"anchor":anchor,"offset":offset}))?,
        )
    }
    pub fn notification_allowed(&self, room: &str) -> Result<bool> {
        Ok(self.meta("notifications", "true")? == "true"
            && !self
                .db
                .query_row(
                    "SELECT muted FROM room_local WHERE context=?1 AND room=?2",
                    params![self.active, room],
                    |r| r.get::<_, bool>(0),
                )
                .optional()?
                .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (tempfile::TempDir, Store, SigningKey) {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Store::open(&dir.path().join("db.sqlite3")).unwrap();
        db.activate(Some("network-a"), "A").unwrap();
        db.set_meta("profile", "Alice").unwrap();
        let key = SigningKey::from_bytes(&[4; 32]);
        (dir, db, key)
    }
    fn message(text: &str) -> Payload {
        Payload::Message {
            room: GENERAL.into(),
            content: text.into(),
            mode: MessageMode::Code,
        }
    }
    #[test]
    fn persistence_drafts_and_search() {
        let (dir, mut db, key) = setup();
        let context = db.active.clone();
        let e = db.create(&key, message("  exact\n\tcode\n")).unwrap();
        db.save_draft(
            &context,
            GENERAL,
            &Draft {
                content: "draft A".into(),
                code: true,
            },
        )
        .unwrap();
        drop(db);
        let mut db = Store::open(&dir.path().join("db.sqlite3")).unwrap();
        db.activate(Some("network-a"), "A").unwrap();
        assert_eq!(
            db.messages(&context, GENERAL, None, None).unwrap()[0].content,
            "  exact\n\tcode\n"
        );
        assert!(db.draft(&context, GENERAL).unwrap().code);
        assert_eq!(db.search("exact", None).unwrap().len(), 1);
        assert!(db.search("\" OR *", None).is_ok());
        assert_eq!(db.inventory(0).unwrap()[0].2, e.sequence);
    }
    #[test]
    fn context_isolation_including_pending() {
        let (_, mut db, key) = setup();
        let old = db.active.clone();
        let e = db.create(&key, message("old secret")).unwrap();
        let keys = db.inventory(0).unwrap();
        db.activate(Some("network-b"), "B").unwrap();
        assert!(db.export(&keys).unwrap().is_empty());
        assert!(!db.ingest(&e).unwrap());
        assert!(db.draft(&db.active, GENERAL).unwrap().content.is_empty());
        assert_eq!(db.messages(&old, GENERAL, None, None).unwrap().len(), 1);
        assert!(!db.search("secret", None).unwrap()[0].current);
        db.activate(Some("network-a"), "A").unwrap();
        assert_eq!(db.export(&keys).unwrap().len(), 1);
    }
    #[test]
    fn ownership_out_of_order_gaps_and_wrong_clocks() {
        let (_, mut db, key) = setup();
        let author = hex::encode(key.verifying_key().as_bytes());
        let room = format!("{author}:{}", uuid::Uuid::new_v4());
        let ns = uuid::Uuid::new_v4().to_string();
        let create = Event::sign(
            &key,
            ns.clone(),
            "A".into(),
            1,
            1,
            Payload::CreateRoom {
                room: room.clone(),
                name: "Same name".into(),
                description: "".into(),
            },
        )
        .unwrap();
        let rename = Event::sign(
            &key,
            ns.clone(),
            "A".into(),
            3,
            3,
            Payload::RenameRoom {
                room: room.clone(),
                name: "Renamed".into(),
                description: "".into(),
            },
        )
        .unwrap();
        db.ingest(&rename).unwrap();
        db.ingest(&create).unwrap();
        assert_eq!(
            db.rooms(&db.active)
                .unwrap()
                .iter()
                .find(|r| r.id == room)
                .unwrap()
                .name,
            "Renamed"
        );
        assert!(!db.ingest(&create).unwrap());
        let other = SigningKey::from_bytes(&[9; 32]);
        assert!(
            db.create(
                &other,
                Payload::ArchiveRoom {
                    room: room.clone(),
                    archived: true
                }
            )
            .is_err()
        );
        let hostile = Event::sign(
            &other,
            ns.clone(),
            "Pretender".into(),
            1,
            9,
            Payload::RenameRoom {
                room: room.clone(),
                name: "Stolen".into(),
                description: "".into(),
            },
        )
        .unwrap();
        assert!(db.ingest(&hostile).is_err());
        assert_eq!(
            db.rooms(&db.active)
                .unwrap()
                .iter()
                .find(|r| r.id == room)
                .unwrap()
                .name,
            "Renamed"
        );
        let mut delayed = Event::sign(
            &key,
            ns.clone(),
            "A".into(),
            2,
            2,
            Payload::Message {
                room: room.clone(),
                content: "Earlier logical value".into(),
                mode: MessageMode::Text,
            },
        )
        .unwrap();
        delayed.timestamp = 9999999999999;
        let bytes = delayed.bytes().unwrap();
        use ed25519_dalek::Signer;
        use sha2::{Digest, Sha256};
        delayed.id = hex::encode(Sha256::digest(&bytes));
        delayed.signature = hex::encode(key.sign(&bytes).to_bytes());
        let expected = vec![
            (ns.clone(), author.clone(), 1),
            (ns.clone(), author.clone(), 2),
            (ns, author, 3),
        ];
        assert_eq!(db.missing(&expected).unwrap().len(), 1);
        db.ingest(&delayed).unwrap();
        assert!(db.missing(&expected).unwrap().is_empty());
        assert_eq!(
            db.messages(&db.active, &room, None, None).unwrap()[0].logical,
            2
        );
    }
    #[test]
    fn duplicate_room_names_and_archive_preserves_delayed_messages() {
        let (_, mut db, key) = setup();
        let author = hex::encode(key.verifying_key().as_bytes());
        let mut ids = Vec::new();
        for _ in 0..2 {
            let room = format!("{author}:{}", uuid::Uuid::new_v4());
            db.create(
                &key,
                Payload::CreateRoom {
                    room: room.clone(),
                    name: "Same".into(),
                    description: "".into(),
                },
            )
            .unwrap();
            ids.push(room);
        }
        assert_eq!(
            db.rooms(&db.active)
                .unwrap()
                .iter()
                .filter(|r| r.name == "Same")
                .count(),
            2
        );
        db.create(
            &key,
            Payload::ArchiveRoom {
                room: ids[0].clone(),
                archived: true,
            },
        )
        .unwrap();
        assert!(
            db.create(
                &key,
                Payload::Message {
                    room: ids[0].clone(),
                    content: "cannot compose".into(),
                    mode: MessageMode::Text
                }
            )
            .is_err()
        );
        let delayed = Event::sign(
            &key,
            uuid::Uuid::new_v4().to_string(),
            "A".into(),
            1,
            2,
            Payload::Message {
                room: ids[0].clone(),
                content: "delayed historical message".into(),
                mode: MessageMode::Text,
            },
        )
        .unwrap();
        db.ingest(&delayed).unwrap();
        assert_eq!(
            db.messages(&db.active, &ids[0], None, None).unwrap().len(),
            1
        );
        db.create(
            &key,
            Payload::ArchiveRoom {
                room: ids[0].clone(),
                archived: false,
            },
        )
        .unwrap();
        assert!(
            !db.rooms(&db.active)
                .unwrap()
                .iter()
                .find(|r| r.id == ids[0])
                .unwrap()
                .archived
        );
    }
}

#[cfg(feature = "test-profile")]
impl Store {
    pub fn seed_search_fixture(&mut self, count: usize) -> Result<()> {
        let tx = self.db.transaction()?;
        for i in 0..count {
            let id = format!("performance-fixture-{i}");
            let body = format!(
                "Synthetic performance fixture {i}. {}",
                if i % 100 == 0 {
                    "connection clipboard searchable needle"
                } else {
                    "ordinary local message content"
                }
            );
            tx.execute("INSERT INTO events VALUES(?1,?2,'fixture','fixture',?3,?3,'general','message','{}')",params![id,self.active,i as i64+1])?;
            tx.execute("INSERT INTO messages VALUES(?1,?2,'general','fixture','Synthetic fixture',?3,'text',?4,?4,'received')",params![id,self.active,body,i as i64+1])?;
        }
        tx.commit()?;
        Ok(())
    }
}
