#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::RunEvent;
use tracing::info;

// Import our modules
mod bridge;
mod backend_routes;
mod backend_integration;
mod file_manager;
mod pipeline_executor;
mod knowledge_search;
mod socket_bridge;
mod socket_commands;

use bridge::{LocalRequest, LocalResponse, AppState, process_local_request, handle_http_request};
use backend_routes::BackendRouter;
use backend_integration::initialize_backend;

#[tauri::command]
fn toggle_fullscreen(window: tauri::Window) {
    if let Ok(is_fullscreen) = window.is_fullscreen() {
        let _ = window.set_fullscreen(!is_fullscreen);
    }
}

/// Simple health check command
#[tauri::command]
async fn simple_health_check() -> Result<String, String> {
    Ok("Backend is running".to_string())
}

/// HTTP request handler using the tauri-actix-web pattern
#[tauri::command]
async fn local_app_request(
    app_state: tauri::State<'_, AppState>,
    request: LocalRequest,
) -> Result<LocalResponse, String> {
    // Process the request using the bridge module
    process_local_request(&app_state.inner(), request).await
}

/// Initialize logging for the integrated application
fn initialize_logging() {
    dotenvy::dotenv().ok();

    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(tracing::Level::INFO);

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global subscriber");
}

#[tokio::main]
async fn main() {
    // Initialize logging first
    initialize_logging();

    // Initialize the backend
    let backend_state = match initialize_backend().await {
        Ok(state) => {
            info!("✅ Backend initialized successfully");
            state
        },
        Err(e) => {
            panic!("Failed to initialize backend: {}", e);
        }
    };

    // Create the backend router
    let backend_router = BackendRouter::new(backend_state);

    // Initialize enhanced services
    match backend_router.initialize().await {
        Ok(()) => {
            info!("✅ Enhanced services initialized successfully");
        },
        Err(e) => {
            panic!("Failed to initialize enhanced services: {}", e);
        }
    }

    // Create the application state
    let app_state = AppState {
        backend_router,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_http::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // UI commands
            toggle_fullscreen,
            // Backend commands
            simple_health_check,
            local_app_request,
            handle_http_request,
            // Socket commands
            socket_commands::init_socket,
            socket_commands::socket_connect,
            socket_commands::socket_disconnect,
            socket_commands::join_room,
            socket_commands::leave_room,
            socket_commands::emit_event,
            socket_commands::get_connection_status,
            socket_commands::send_chat_message,
            socket_commands::send_typing_indicator,
            socket_commands::update_presence_status,
            socket_commands::join_document,
            socket_commands::send_document_update,
            socket_commands::get_socket_stats,
        ])
        .build(tauri::generate_context!())
        .expect("Error while running tauri application")
        .run(|_app_handle, event| {
            match event {
                RunEvent::Ready => {
                    info!("🎉 Tauri application is ready");
                },
                RunEvent::ExitRequested { .. } => {
                    info!("👋 Tauri application is shutting down");
                },
                _ => {}
            }
        });
}
