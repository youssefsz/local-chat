use anyhow::{Result, bail, ensure};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PROTOCOL: u16 = 1;
pub const MESSAGE_BYTES: usize = 65_536;
pub const FRAME_BYTES: usize = 1_048_576;
pub const BATCH_SIZE: usize = 8;
pub const PAGE_SIZE: usize = 80;
pub const GENERAL: &str = "general";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Payload {
    CreateRoom {
        room: String,
        name: String,
        description: String,
    },
    RenameRoom {
        room: String,
        name: String,
        description: String,
    },
    ArchiveRoom {
        room: String,
        archived: bool,
    },
    Message {
        room: String,
        content: String,
        mode: MessageMode,
    },
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageMode {
    Text,
    Code,
}
impl Payload {
    pub fn room(&self) -> &str {
        match self {
            Self::CreateRoom { room, .. }
            | Self::RenameRoom { room, .. }
            | Self::ArchiveRoom { room, .. }
            | Self::Message { room, .. } => room,
        }
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.room().is_empty() && self.room().len() <= 128,
            "Invalid room identifier"
        );
        match self {
            Self::CreateRoom {
                name, description, ..
            }
            | Self::RenameRoom {
                name, description, ..
            } => {
                ensure!(self.room() != GENERAL, "General is a system room");
                ensure!(
                    !name.trim().is_empty() && name.chars().count() <= 80,
                    "Room names must contain 1–80 characters"
                );
                ensure!(
                    description.chars().count() <= 240,
                    "Descriptions may contain up to 240 characters"
                );
            }
            Self::ArchiveRoom { room, .. } => {
                ensure!(room != GENERAL, "General cannot be archived")
            }
            Self::Message { content, .. } => ensure!(
                !content.trim().is_empty() && content.len() <= MESSAGE_BYTES,
                "Messages must contain 1–65536 UTF-8 bytes"
            ),
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub version: u16,
    pub namespace: String,
    pub id: String,
    pub author: String,
    pub profile: String,
    pub sequence: i64,
    pub logical: i64,
    pub timestamp: i64,
    pub payload: Payload,
    pub signature: String,
}
impl Event {
    pub fn bytes(&self) -> Result<Vec<u8>> {
        // Explicit tuple ordering is the wire canonicalization. No unordered maps.
        Ok(serde_json::to_vec(&(
            self.version,
            &self.namespace,
            &self.author,
            &self.profile,
            self.sequence,
            self.logical,
            self.timestamp,
            &self.payload,
        ))?)
    }
    pub fn sign(
        key: &SigningKey,
        namespace: String,
        profile: String,
        sequence: i64,
        logical: i64,
        payload: Payload,
    ) -> Result<Self> {
        let mut event = Self {
            version: PROTOCOL,
            namespace,
            id: String::new(),
            author: hex::encode(key.verifying_key().as_bytes()),
            profile,
            sequence,
            logical,
            timestamp: now(),
            payload,
            signature: String::new(),
        };
        event.payload.validate()?;
        let bytes = event.bytes()?;
        event.id = hex::encode(Sha256::digest(&bytes));
        event.signature = hex::encode(key.sign(&bytes).to_bytes());
        Ok(event)
    }
    pub fn verify(&self) -> Result<()> {
        ensure!(self.version == PROTOCOL, "Incompatible event version");
        ensure!(
            self.sequence > 0
                && self.sequence <= 9_007_199_254_740_991
                && self.logical > 0
                && self.logical <= 9_007_199_254_740_991,
            "Invalid event counter"
        );
        ensure!(
            uuid::Uuid::parse_str(&self.namespace).is_ok(),
            "Invalid namespace"
        );
        ensure!(
            !self.profile.trim().is_empty() && self.profile.chars().count() <= 80,
            "Invalid display name"
        );
        ensure!(
            (-8_640_000_000_000_000..=8_640_000_000_000_000).contains(&self.timestamp),
            "Invalid display timestamp"
        );
        self.payload.validate()?;
        if let Payload::CreateRoom { room, .. } = &self.payload {
            ensure!(
                room.starts_with(&format!("{}:", self.author))
                    && uuid::Uuid::parse_str(room.rsplit(':').next().unwrap_or("")).is_ok(),
                "Room identity must belong to its creator"
            );
        }
        if matches!(
            self.payload,
            Payload::RenameRoom { .. } | Payload::ArchiveRoom { .. }
        ) {
            ensure!(
                self.payload
                    .room()
                    .starts_with(&format!("{}:", self.author)),
                "Only the creator may manage this room"
            );
        }
        let bytes = self.bytes()?;
        ensure!(
            self.id == hex::encode(Sha256::digest(&bytes)),
            "Event identifier mismatch"
        );
        verify(&self.author, &bytes, &self.signature)
    }
}
pub fn verify(author: &str, bytes: &[u8], signature: &str) -> Result<()> {
    let public: [u8; 32] = hex::decode(author)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid public key"))?;
    let sig = Signature::from_slice(&hex::decode(signature)?)?;
    VerifyingKey::from_bytes(&public)?.verify_strict(bytes, &sig)?;
    Ok(())
}
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}
pub fn validate_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name.chars().count() > 80 {
        bail!("Display name must contain 1–80 characters")
    };
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub context: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub archived: bool,
    pub muted: bool,
    pub unread: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub room: String,
    pub author: String,
    pub profile: String,
    pub content: String,
    pub mode: MessageMode,
    pub logical: i64,
    pub timestamp: i64,
    pub delivery: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessagePage {
    pub items: Vec<Message>,
    pub has_older: bool,
    pub has_newer: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Draft {
    pub content: String,
    pub code: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    pub label: String,
    pub current: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub excerpt: String,
    pub message: Message,
    pub context: String,
    pub room_name: String,
    pub current: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub status: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub identity: String,
    pub profile: String,
    pub theme: String,
    pub notifications: bool,
    pub context: String,
    pub contexts: Vec<Context>,
    pub rooms: Vec<Room>,
    pub peers: Vec<Peer>,
    pub network_status: String,
    pub network_detail: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signatures_and_limits() {
        let key = SigningKey::from_bytes(&[7; 32]);
        let mut event = Event::sign(
            &key,
            uuid::Uuid::new_v4().to_string(),
            "A".into(),
            1,
            1,
            Payload::Message {
                room: GENERAL.into(),
                content: "  x\n\t".into(),
                mode: MessageMode::Code,
            },
        )
        .unwrap();
        event.verify().unwrap();
        event.profile = "B".into();
        assert!(event.verify().is_err());
        assert!(
            Payload::ArchiveRoom {
                room: GENERAL.into(),
                archived: true
            }
            .validate()
            .is_err()
        );
        assert!(
            Payload::Message {
                room: GENERAL.into(),
                content: "x".repeat(MESSAGE_BYTES + 1),
                mode: MessageMode::Text
            }
            .validate()
            .is_err()
        );
    }
}
