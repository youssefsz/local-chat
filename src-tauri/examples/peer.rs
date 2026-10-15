//! Test-only isolated peer process. This binary is never bundled in the desktop app.
use local_chat::{
    domain::{Draft, MessageMode, Payload},
    network::Engine,
    platform::Identity,
    storage::Store,
};
use tokio::io::{AsyncBufReadExt, BufReader};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(args.next().expect("isolated data directory required"));
    let name = args.next().unwrap_or_else(|| "Test peer".into());
    std::fs::create_dir_all(&dir)?;
    let identity = Identity::load(&dir, true)?;
    let db = Store::open(&dir.join("local-chat.sqlite3"))?;
    db.set_meta("profile", &name)?;
    db.set_meta("identity", &identity.id())?;
    let engine = Engine::new(db, identity);
    let network = tokio::spawn(engine.clone().run());
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = lines.next_line().await? {
        let response = async {
            let value: serde_json::Value = serde_json::from_str(&line)?;
            let operation = value["op"].as_str().unwrap_or("");
            match operation {
                "status" => Ok(serde_json::to_value(engine.snapshot().await?)?),
                "reconnect" => {
                    engine.reconnect().await?;
                    Ok(serde_json::json!({"restarted":true}))
                }
                "draft" => {
                    let db = engine.store.lock().await;
                    let room = value["room"].as_str().unwrap_or("general");
                    if let Some(content) = value["content"].as_str() {
                        db.save_draft(
                            &db.active,
                            room,
                            &Draft {
                                content: content.into(),
                                code: true,
                            },
                        )?;
                    }
                    Ok(serde_json::to_value(db.draft(&db.active, room)?)?)
                }
                "send" => {
                    let event = engine
                        .publish(Payload::Message {
                            room: value["room"].as_str().unwrap_or("general").into(),
                            content: value["content"].as_str().unwrap_or("").into(),
                            mode: if value["code"].as_bool().unwrap_or(false) {
                                MessageMode::Code
                            } else {
                                MessageMode::Text
                            },
                        })
                        .await?;
                    Ok(serde_json::json!({"id":event.id}))
                }
                "create" => {
                    let room = format!("{}:{}", engine.identity.id(), uuid::Uuid::new_v4());
                    engine
                        .publish(Payload::CreateRoom {
                            room: room.clone(),
                            name: value["name"].as_str().unwrap_or("Integration room").into(),
                            description: String::new(),
                        })
                        .await?;
                    Ok(serde_json::json!({"room":room}))
                }
                "rename" | "archive" | "restore" => {
                    let room = value["room"].as_str().unwrap_or("general").to_string();
                    let payload = if operation == "rename" {
                        Payload::RenameRoom {
                            room,
                            name: value["name"].as_str().unwrap_or("Renamed room").into(),
                            description: String::new(),
                        }
                    } else {
                        Payload::ArchiveRoom {
                            room,
                            archived: operation == "archive",
                        }
                    };
                    engine.publish(payload).await?;
                    Ok(serde_json::json!({"managed":true}))
                }
                "messages" => {
                    let db = engine.store.lock().await;
                    Ok(serde_json::to_value(db.messages(
                        &db.active,
                        value["room"].as_str().unwrap_or("general"),
                        None,
                        None,
                    )?)?)
                }
                "quit" => {
                    engine.stop.cancel();
                    Ok(serde_json::json!({"stopping":true}))
                }
                _ => anyhow::bail!("Unknown test operation"),
            }
        }
        .await;
        match response {
            Ok::<serde_json::Value, anyhow::Error>(value) => {
                println!("{}", serde_json::json!({"ok":value}))
            }
            Err(err) => println!("{}", serde_json::json!({"error":err.to_string()})),
        };
        if engine.stop.is_cancelled() {
            break;
        }
    }
    engine.stop.cancel();
    network.await?;
    Ok(())
}
