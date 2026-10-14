use crate::{
    domain::*,
    network::{Change, Engine},
    platform::Identity,
    storage::Store,
};
use std::sync::Arc;
use tauri::{Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
type DesktopState = Arc<Engine>;
struct LaunchClock(std::time::Instant, std::sync::atomic::AtomicBool);
type Reply<T> = Result<T, String>;
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
#[tauri::command]
async fn snapshot(state: State<'_, DesktopState>) -> Reply<Snapshot> {
    state.snapshot().await.map_err(error)
}
#[tauri::command]
async fn rooms(state: State<'_, DesktopState>, context: String) -> Reply<Vec<Room>> {
    state.store.lock().await.rooms(&context).map_err(error)
}
#[tauri::command]
async fn messages(
    state: State<'_, DesktopState>,
    context: String,
    room: String,
    before: Option<String>,
    around: Option<String>,
) -> Reply<MessagePage> {
    state
        .store
        .lock()
        .await
        .page(&context, &room, before, around)
        .map_err(error)
}
#[tauri::command]
async fn send_message(
    state: State<'_, DesktopState>,
    context: String,
    room: String,
    content: String,
    code: bool,
) -> Reply<String> {
    let event = {
        let mut db = state.store.lock().await;
        if db.active != context {
            return Err("This network is no longer current. Your draft has been kept.".into());
        }
        db.create(
            &state.identity.signing,
            Payload::Message {
                room: room.clone(),
                content,
                mode: if code {
                    MessageMode::Code
                } else {
                    MessageMode::Text
                },
            },
        )
        .map_err(error)?
    };
    let id = event.id.clone();
    let _ = state.changes.send(Change::Event(event));
    Ok(id)
}
#[tauri::command]
async fn manage_room(
    state: State<'_, DesktopState>,
    context: String,
    action: String,
    room: Option<String>,
    name: String,
    description: String,
) -> Reply<String> {
    let payload = match action.as_str() {
        "create" => Payload::CreateRoom {
            room: format!("{}:{}", state.identity.id(), uuid::Uuid::new_v4()),
            name,
            description,
        },
        "rename" => Payload::RenameRoom {
            room: room.ok_or("Missing room")?,
            name,
            description,
        },
        "archive" | "restore" => Payload::ArchiveRoom {
            room: room.ok_or("Missing room")?,
            archived: action == "archive",
        },
        _ => return Err("Unknown room action".into()),
    };
    let event = {
        let mut db = state.store.lock().await;
        if db.active != context {
            return Err("Previous network history is read-only".into());
        }
        db.create(&state.identity.signing, payload).map_err(error)?
    };
    let room = event.payload.room().to_string();
    let _ = state.changes.send(Change::Event(event));
    Ok(room)
}
#[tauri::command]
async fn draft(state: State<'_, DesktopState>, context: String, room: String) -> Reply<Draft> {
    state
        .store
        .lock()
        .await
        .draft(&context, &room)
        .map_err(error)
}
#[tauri::command]
async fn save_draft(
    state: State<'_, DesktopState>,
    context: String,
    room: String,
    content: String,
    code: bool,
) -> Reply<()> {
    state
        .store
        .lock()
        .await
        .save_draft(&context, &room, &Draft { content, code })
        .map_err(error)
}
#[tauri::command]
async fn room_local(
    state: State<'_, DesktopState>,
    context: String,
    room: String,
    muted: Option<bool>,
    read: Option<i64>,
    scroll: Option<f64>,
) -> Reply<f64> {
    state
        .store
        .lock()
        .await
        .room_local(&context, &room, muted, read, scroll)
        .map_err(error)
}
#[tauri::command]
async fn view_state(
    state: State<'_, DesktopState>,
    context: String,
    room: String,
) -> Reply<serde_json::Value> {
    state
        .store
        .lock()
        .await
        .view(&context, &room)
        .map_err(error)
}
#[tauri::command]
async fn save_view(
    state: State<'_, DesktopState>,
    context: String,
    room: String,
    anchor: String,
    offset: f64,
) -> Reply<()> {
    state
        .store
        .lock()
        .await
        .save_view(&context, &room, &anchor, offset)
        .map_err(error)
}
#[tauri::command]
async fn search(
    state: State<'_, DesktopState>,
    query: String,
    room: Option<String>,
    context: String,
) -> Reply<Vec<SearchResult>> {
    state
        .store
        .lock()
        .await
        .search_context(&query, room.as_deref(), &context)
        .map_err(error)
}
#[tauri::command]
async fn preferences(
    state: State<'_, DesktopState>,
    profile: String,
    theme: String,
    notifications: bool,
) -> Reply<()> {
    validate_name(&profile).map_err(error)?;
    if !["system", "light", "dark"].contains(&theme.as_str()) {
        return Err("Invalid appearance".into());
    }
    let db = state.store.lock().await;
    db.set_meta("profile", profile.trim()).map_err(error)?;
    db.set_meta("theme", &theme).map_err(error)?;
    db.set_meta(
        "notifications",
        if notifications { "true" } else { "false" },
    )
    .map_err(error)?;
    drop(db);
    state.changed();
    Ok(())
}
#[tauri::command]
async fn record_room_switch(state: State<'_, DesktopState>, milliseconds: f64) -> Reply<()> {
    if !milliseconds.is_finite() || !(0.0..=60000.0).contains(&milliseconds) {
        return Err("Invalid timing".into());
    }
    state
        .store
        .lock()
        .await
        .set_meta("last_room_switch_ms", &milliseconds.to_string())
        .map_err(error)
}
#[tauri::command]
fn ready(
    window: tauri::WebviewWindow,
    state: State<'_, DesktopState>,
    clock: State<'_, LaunchClock>,
) -> Reply<()> {
    let milliseconds = clock.0.elapsed().as_secs_f64() * 1000.;
    // Local measurement requested by the product brief; no message data or telemetry leaves the device.
    if let Ok(db) = state.store.try_lock() {
        let _ = db.set_meta("last_launch_ms", &milliseconds.to_string());
    }
    window.show().map_err(error)?;
    window.set_focus().map_err(error)?;
    if !clock.1.swap(true, std::sync::atomic::Ordering::AcqRel) {
        tauri::async_runtime::spawn(state.inner().clone().run());
    }
    Ok(())
}
fn open(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn run() {
    let launched = std::time::Instant::now();
    let _ = rustls::crypto::ring::default_provider().install_default();
    let builder = tauri::Builder::default();
    #[cfg(not(feature = "test-profile"))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _, _| open(app)));
    let app = builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            snapshot,
            rooms,
            messages,
            send_message,
            manage_room,
            draft,
            save_draft,
            room_local,
            view_state,
            save_view,
            search,
            preferences,
            record_room_switch,
            ready
        ])
        .setup(move |app| {
            let setup_enter_ms = launched.elapsed().as_secs_f64() * 1000.;
            let test = cfg!(feature = "test-profile");
            let data = {
                #[cfg(feature = "test-profile")]
                {
                    std::env::var_os("LOCAL_CHAT_TEST_DATA")
                        .map(std::path::PathBuf::from)
                        .unwrap_or(app.path().app_data_dir()?.join("test-profile"))
                }
                #[cfg(not(feature = "test-profile"))]
                {
                    app.path().app_data_dir()?
                }
            };
            std::fs::create_dir_all(&data)?;
            #[cfg(feature = "test-profile")]
            if let Ok(json) = std::fs::read_to_string(data.join("window.json")) {
                #[derive(serde::Deserialize)]
                #[serde(deny_unknown_fields)]
                struct TestWindow {
                    width: u32,
                    height: u32,
                }
                let size: TestWindow = serde_json::from_str(&json)?;
                if !(800..=1600).contains(&size.width) || !(600..=1100).contains(&size.height) {
                    return Err(anyhow::anyhow!("Invalid dedicated test window size").into());
                }
                if let Some(window) = app.get_webview_window("main") {
                    window.set_size(tauri::LogicalSize::new(size.width, size.height))?;
                }
            }

            let identity = Identity::load(&data, test)?;
            let identity_ready_ms = launched.elapsed().as_secs_f64() * 1000.;
            let store = Store::open(&data.join("local-chat.sqlite3"))?;
            let default_name = std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "You".into());
            if store.meta("profile", "")?.is_empty() {
                store.set_meta("profile", &default_name)?;
            }
            store.set_meta("identity", &identity.id())?;
            let theme = store.meta("theme", "system")?;
            if let Some(w) = app.get_webview_window("main") {
                w.set_theme(match theme.as_str() {
                    "light" => Some(tauri::Theme::Light),
                    "dark" => Some(tauri::Theme::Dark),
                    _ => None,
                })?;
            }
            store.set_meta("startup_setup_enter_ms", &setup_enter_ms.to_string())?;
            store.set_meta("startup_identity_ready_ms", &identity_ready_ms.to_string())?;
            let engine = Engine::new(store, identity);
            app.manage(engine.clone());
            app.manage(LaunchClock(
                launched,
                std::sync::atomic::AtomicBool::new(false),
            ));
            let handle = app.handle().clone();
            let events = engine.clone();
            tauri::async_runtime::spawn(async move {
                let mut changes = events.changes.subscribe();
                while let Ok(change) = changes.recv().await {
                    if let Change::Received(event, true) = &change {
                        let background = handle.get_webview_window("main").is_none_or(|w| {
                            !w.is_visible().unwrap_or(false) || !w.is_focused().unwrap_or(false)
                        });
                        if background
                            && let Payload::Message { room, .. } = &event.payload
                            && events
                                .store
                                .lock()
                                .await
                                .notification_allowed(room)
                                .unwrap_or(false)
                        {
                            let _ = handle
                                .notification()
                                .builder()
                                .title(format!("{} in Local Chat", event.profile))
                                .body("A new message arrived on your local network.")
                                .show();
                        }
                    }
                    let kind = if matches!(change, Change::State) {
                        "state"
                    } else {
                        "messages"
                    };
                    let _ = handle.emit("local-chat:changed", serde_json::json!({"kind":kind}));
                }
            });
            let menu = tauri::menu::Menu::default(app.handle())?;
            app.set_menu(menu)?;
            let show =
                tauri::menu::MenuItem::with_id(app, "open", "Open Local Chat", true, None::<&str>)?;
            let quit =
                tauri::menu::MenuItem::with_id(app, "quit", "Quit Local Chat", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&show, &quit])?;
            tauri::tray::TrayIconBuilder::new()
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/tray.png"
                ))?)
                .icon_as_template(true)
                .tooltip("Local Chat")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => open(app),
                    "quit" => {
                        app.state::<DesktopState>().stop.cancel();
                        app.exit(0)
                    }
                    _ => {}
                })
                .build(app)?;
            if let Ok(db) = engine.store.try_lock() {
                let _ = db.set_meta(
                    "startup_setup_end_ms",
                    &launched
                        .elapsed()
                        .as_secs_f64()
                        .mul_add(1000., 0.)
                        .to_string(),
                );
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("Local Chat could not initialize. Check local credential and data access.");
    app.run(|app, event| match event {
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen { .. } => open(app),
        tauri::RunEvent::Exit => {
            app.state::<DesktopState>().stop.cancel();
        }
        _ => {}
    });
}
