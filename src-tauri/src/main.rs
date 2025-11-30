#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use tauri::{Manager, RunEvent};
use tracing::{info, error};

// Import our new integrated modules
mod bridge;
mod backend_integration;

use backend_integration::{
    get_backend_config, get_backend_models,
    chat_completion, health_check, IntegratedAppState
};

#[tauri::command]
fn toggle_fullscreen(window: tauri::Window) {
    if let Ok(is_fullscreen) = window.is_fullscreen() {
        let _ = window.set_fullscreen(!is_fullscreen);
    }
}

#[tauri::command]
async fn initialize_integrated_backend(
    backend_state: tauri::State<'_, Arc<Mutex<Option<Arc<IntegratedAppState>>>>>,
) -> Result<String, String> {
    info!("🚀 Initializing integrated backend...");

    match backend_integration::initialize_backend().await {
        Ok(app_state) => {
            let app_state_arc = Arc::new(app_state);

            // Store the backend state
            {
                let mut backend_guard = backend_state.lock().unwrap();
                *backend_guard = Some(app_state_arc);
            }

            info!("✅ Integrated backend initialized successfully");
            Ok("Integrated backend initialized successfully".to_string())
        },
        Err(e) => {
            error!("❌ Failed to initialize integrated backend: {}", e);
            Err(format!("Failed to initialize integrated backend: {}", e))
        }
    }
}

/// HTTP request handler that forwards requests to the integrated backend
/// 兼容 Tauri HTTP Proxy 格式的请求
#[tauri::command]
async fn handle_http_request(
    backend_state: tauri::State<'_, Arc<Mutex<Option<Arc<IntegratedAppState>>>>>,
    request: bridge::LocalRequest,
) -> Result<bridge::LocalResponse, String> {
    // Check if backend is initialized
    let app_state_option = {
        let backend_guard = backend_state.lock().unwrap();
        backend_guard.as_ref().cloned()
    };

    match app_state_option {
        Some(app_state) => {
            // Create a request handler and process the request
            let handler = backend_integration::RequestHandler::new(app_state);
            match process_integrated_request(handler, request).await {
                Ok(response) => Ok(response),
                Err(e) => Ok(bridge::LocalResponse::internal_server_error(format!("Request processing failed: {}", e)))
            }
        },
        None => {
            // 如果后端未初始化，尝试直接处理一些基本请求
            handle_fallback_request(request).await
        }
    }
}

/// 处理集成后端的请求
async fn process_integrated_request(
    handler: backend_integration::RequestHandler,
    request: bridge::LocalRequest,
) -> Result<bridge::LocalResponse, String> {
    // 根据路径路由到相应的处理器
    match request.uri.as_str() {
        "/health" => {
            match handler.handle_health().await {
                Ok(response) => bridge::LocalResponse::json(response).map_err(|e| e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        },
        "/api/config" => {
            match handler.handle_config().await {
                Ok(response) => bridge::LocalResponse::json(response).map_err(|e| e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        },
        "/api/models" => {
            match handler.handle_models().await {
                Ok(response) => bridge::LocalResponse::json(response).map_err(|e| e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        },
        path if path.starts_with("/api/chat/completions") => {
            // 解析请求体
            let payload = if let Some(body) = &request.body {
                serde_json::from_str(body).map_err(|e| format!("JSON parse error: {}", e))?
            } else {
                serde_json::Value::Null
            };

            match handler.handle_chat_completion(payload, None).await {
                Ok(response) => bridge::LocalResponse::json(response).map_err(|e| e.to_string()),
                Err(e) => Err(e.to_string()),
            }
        },
        _ => {
            // 对于其他路径，返回 404
            Ok(bridge::LocalResponse {
                status_code: 404,
                body: b"Not Found".to_vec(),
                headers: std::collections::HashMap::new(),
            })
        }
    }
}

/// 处理桥接未就绪时的回退请求
async fn handle_fallback_request(request: bridge::LocalRequest) -> Result<bridge::LocalResponse, String> {
    use serde_json::json;

    match request.uri.as_str() {
        "/health" => {
            Ok(bridge::LocalResponse::json(json!({
                "status": true,
                "message": "Backend is initializing..."
            })).map_err(|e| e.to_string())?)
        },
        "/api/config" => {
            Ok(bridge::LocalResponse::json(json!({
                "status": true,
                "name": "Open CoreUI",
                "version": env!("CARGO_PKG_VERSION"),
                "features": {
                    "auth": false,
                    "enable_signup": false,
                    "enable_login_form": false,
                    "enable_api_key": false,
                    "enable_websocket": false,
                    "enable_version_update_check": false,
                }
            })).map_err(|e| e.to_string())?)
        },
        _ => {
            Err(format!("Backend not initialized and no fallback for: {}", request.uri))
        }
    }
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

fn main() {
    // Initialize logging first
    initialize_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // Initialize shared state for integrated backend only
            app.manage(Arc::new(Mutex::new(None::<Arc<IntegratedAppState>>)));

            info!("🔧 Tauri application setup completed with integrated backend");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Legacy commands for compatibility
            toggle_fullscreen,
            // Integrated backend commands
            initialize_integrated_backend,
            get_backend_config,
            get_backend_models,
            chat_completion,
            health_check,
            handle_http_request,
        ])
        .build(tauri::generate_context!())
        .expect("Error while running tauri application")
        .run(|app_handle, event| {
            match event {
                RunEvent::Ready => {
                    info!("🎉 Tauri application is ready");
                    // Auto-initialize the integrated backend
                    let app_handle_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = initialize_integrated_backend(app_handle_clone.state()).await {
                            error!("Failed to auto-initialize backend: {}", e);
                        }
                    });
                },
                RunEvent::ExitRequested { .. } => {
                    info!("👋 Tauri application is shutting down");
                },
                _ => {}
            }
        });
}
