//! Explicit synthetic UI fixture tool. Never bundled with the production application.
use ed25519_dalek::SigningKey;
use local_chat::{
    domain::{Event, MessageMode, Payload},
    platform::{Identity, network_evidence},
    storage::Store,
};
fn main() -> anyhow::Result<()> {
    let path = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Fresh dedicated UI test directory required"),
    );
    std::fs::create_dir_all(&path)?;
    anyhow::ensure!(
        !path.join("local-chat.sqlite3").exists(),
        "Do not overwrite an existing fixture database"
    );
    let identity = Identity::load(&path, true)?;
    let mut db = Store::open(&path.join("local-chat.sqlite3"))?;
    let evidence = network_evidence()?;
    let fingerprint = evidence.fingerprint();
    db.activate(fingerprint.as_deref(), "UI verification fixture")?;
    db.set_meta("identity", &identity.id())?;
    db.set_meta("profile", "UI verification")?;
    db.set_meta("theme", "light")?;
    db.set_meta("notifications", "false")?;
    for (name, description) in [
        ("Development", "Useful snippets and small fixes."),
        ("Design", "A place for the details."),
        ("Useful links", "References worth keeping."),
    ] {
        db.create(
            &identity.signing,
            Payload::CreateRoom {
                room: format!("{}:{}", identity.id(), uuid::Uuid::new_v4()),
                name: name.into(),
                description: description.into(),
            },
        )?;
    }
    db.create(&identity.signing,Payload::Message{room:"general".into(),content:"This is a dedicated UI verification profile. These messages are synthetic fixtures.".into(),mode:MessageMode::Text})?;
    let key = SigningKey::from_bytes(&[29; 32]);
    let namespace = uuid::Uuid::new_v4().to_string();
    db.ingest(&Event::sign(
        &key,
        namespace.clone(),
        "Fixture author".into(),
        1,
        10,
        Payload::Message {
            room: "general".into(),
            content: "The composer stays available while messages and code scroll above it.".into(),
            mode: MessageMode::Text,
        },
    )?)?;
    db.ingest(&Event::sign(
        &key,
        namespace,
        "Fixture author".into(),
        2,
        11,
        Payload::Message {
            room: "general".into(),
            content: "A saved reference: https://doc.rust-lang.org/book/".into(),
            mode: MessageMode::Text,
        },
    )?)?;
    db.create(
        &identity.signing,
        Payload::Message {
            room: "general".into(),
            content: "  const room = 'Local Chat';\n\tconsole.log(room);\n".into(),
            mode: MessageMode::Code,
        },
    )?;
    println!("Created signed synthetic UI fixture in {}", path.display());
    Ok(())
}
