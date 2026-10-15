use super::{DesktopState, LaunchClock, Reply, error, watch_changes};
use crate::{network::Engine, platform::Identity, storage::Store};
use std::{path::PathBuf, sync::Mutex};
use tauri::{Emitter, Manager, State};

#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    System,
    Light,
    Dark,
}
impl Appearance {
    fn saved(value: &str) -> Self {
        match value {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }
}
#[derive(Clone, serde::Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Phase {
    Loading,
    Ready,
    Failed { message: String },
}
#[derive(Clone, serde::Serialize)]
pub struct Snapshot {
    // None means the saved appearance has not been read yet. Keep the window hidden.
    theme: Option<Appearance>,
    #[serde(flatten)]
    phase: Phase,
}
struct Progress {
    snapshot: Snapshot,
    attempts: u32,
}
pub struct Startup {
    data: PathBuf,
    progress: Mutex<Progress>,
}
impl Startup {
    pub fn new(data: PathBuf) -> Self {
        Self {
            data,
            progress: Mutex::new(Progress {
                snapshot: Snapshot {
                    theme: None,
                    phase: Phase::Loading,
                },
                attempts: 0,
            }),
        }
    }
    pub fn begin(&self, app: tauri::AppHandle) {
        let attempt = {
            let mut progress = self.progress.lock().expect("startup state");
            if matches!(progress.snapshot.phase, Phase::Ready)
                || (matches!(progress.snapshot.phase, Phase::Loading) && progress.attempts > 0)
            {
                return;
            }
            progress.attempts += 1;
            progress.snapshot.phase = Phase::Loading;
            progress.attempts
        };
        self.emit(&app);
        let data = self.data.clone();
        tauri::async_runtime::spawn(async move {
            let handle = app.clone();
            let initialized =
                tauri::async_runtime::spawn_blocking(move || -> anyhow::Result<DesktopState> {
                    std::fs::create_dir_all(&data)?;
                    let store = Store::open(&data.join("local-chat.sqlite3"))?;
                    let theme = Appearance::saved(&store.meta("theme", "system")?);
                    handle
                        .state::<Startup>()
                        .progress
                        .lock()
                        .expect("startup state")
                        .snapshot
                        .theme = Some(theme);
                    handle.state::<Startup>().emit(&handle);
                    #[cfg(feature = "test-profile")]
                    test_startup(&data, attempt)?;
                    #[cfg(not(feature = "test-profile"))]
                    let _ = attempt;
                    // Keychain access can wait on the OS. Never block the native event loop.
                    let identity = Identity::load(&data, cfg!(feature = "test-profile"))?;
                    let default_name = std::env::var("USER")
                        .or_else(|_| std::env::var("USERNAME"))
                        .unwrap_or_else(|_| "You".into());
                    if store.meta("profile", "")?.is_empty() {
                        store.set_meta("profile", &default_name)?;
                    }
                    store.set_meta("identity", &identity.id())?;
                    store.set_meta(
                        "startup_identity_ready_ms",
                        &(handle
                            .state::<LaunchClock>()
                            .started
                            .elapsed()
                            .as_secs_f64()
                            * 1000.)
                            .to_string(),
                    )?;
                    Ok(Engine::new(store, identity))
                })
                .await;
            let phase = match initialized {
                Ok(Ok(engine)) => {
                    app.manage(engine.clone());
                    watch_changes(app.clone(), engine);
                    Phase::Ready
                }
                Ok(Err(e)) => Phase::Failed { message: error(e) },
                Err(_) => Phase::Failed {
                    message: "Startup was interrupted. Please try again.".into(),
                },
            };
            let startup = app.state::<Startup>();
            {
                let mut progress = startup.progress.lock().expect("startup state");
                progress.snapshot.theme.get_or_insert(Appearance::System);
                progress.snapshot.phase = phase;
            }
            startup.emit(&app);
        });
    }
    fn emit(&self, app: &tauri::AppHandle) {
        let snapshot = self
            .progress
            .lock()
            .expect("startup state")
            .snapshot
            .clone();
        let _ = app.emit("local-chat:startup", snapshot);
    }
}
#[tauri::command]
pub fn startup_status(state: State<'_, Startup>) -> Snapshot {
    state
        .progress
        .lock()
        .expect("startup state")
        .snapshot
        .clone()
}
#[tauri::command]
pub fn retry_startup(app: tauri::AppHandle, state: State<'_, Startup>) -> Reply<()> {
    state.begin(app);
    Ok(())
}

#[cfg(feature = "test-profile")]
fn test_startup(data: &std::path::Path, attempt: u32) -> anyhow::Result<()> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Options {
        delay_ms: u64,
        fail_once: bool,
    }
    let path = data.join("startup.json");
    if !path.exists() {
        return Ok(());
    }
    let options: Options = serde_json::from_slice(&std::fs::read(path)?)?;
    anyhow::ensure!(
        options.delay_ms <= 60_000,
        "Invalid dedicated test startup delay"
    );
    std::thread::sleep(std::time::Duration::from_millis(options.delay_ms));
    anyhow::ensure!(
        !(options.fail_once && attempt == 1),
        "Dedicated test startup failure"
    );
    Ok(())
}
