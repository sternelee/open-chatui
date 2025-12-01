//! Tauri Commands for Socket.IO Integration
//!
//! Provides Tauri command handlers for Socket.IO functionality,
//! allowing the frontend to interact with real-time features.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use crate::socket_bridge::{EventBridge, SocketMessage, SocketEventType, EventListenerConfig, MockSocketProcessor};

/// Application state for socket functionality
pub struct SocketAppState {
    pub bridge: Arc<EventBridge>,
    pub processor: Arc<MockSocketProcessor>,
}

/// Socket event payload from frontend
#[derive(Debug, Deserialize)]
pub struct SocketEventPayload {
    pub event_type: String,
    pub event_name: String,
    pub data: JsonValue,
    pub room: Option<String>,
    pub user_id: Option<String>,
}

/// Socket connection request
#[derive(Debug, Deserialize)]
pub struct ConnectRequest {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub auth_token: Option<String>,
}

/// Socket connection response
#[derive(Debug, Serialize)]
pub struct ConnectResponse {
    pub session_id: String,
    pub status: String,
    pub timestamp: i64,
}

/// Room join request
#[derive(Debug, Deserialize)]
pub struct JoinRoomRequest {
    pub session_id: String,
    pub room: String,
}

/// Emit event request
#[derive(Debug, Deserialize)]
pub struct EmitEventRequest {
    pub session_id: Option<String>,
    pub room: Option<String>,
    pub event_type: String,
    pub event_name: String,
    pub data: JsonValue,
}

/// Initialize socket functionality
#[tauri::command]
pub async fn init_socket(
    app: AppHandle,
) -> Result<ConnectResponse, String> {
    // Create event bridge configuration
    let config = EventListenerConfig {
        auto_reconnect: true,
        reconnect_interval_ms: 3000,
        max_retry_count: 5,
        heartbeat_interval_ms: 30000,
    };

    // Create event bridge
    let (bridge, mut tauri_rx, _bridge_tx) = EventBridge::new(config);
    let bridge = Arc::new(bridge);

    // Create mock processor
    let processor = Arc::new(MockSocketProcessor::new(Arc::clone(&bridge)));
    processor.start().await.map_err(|e| e.to_string())?;

    // Generate session ID
    let session_id = uuid::Uuid::new_v4().to_string();
    processor.create_session(&session_id, None).await;

    // Store in app state
    let socket_state = SocketAppState {
        bridge: Arc::clone(&bridge),
        processor: Arc::clone(&processor),
    };
    app.manage(socket_state);

    // Start listening for events
    let _bridge_clone = Arc::clone(&bridge);
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(message) = tauri_rx.recv().await {
            // Emit Tauri event to frontend
            let event_name = format!("socket:{}", message.event_name);
            let _ = app_clone.emit(&event_name, &message);
        }
    });

    // Set connection status
    bridge.set_connection_status(crate::socket_bridge::ConnectionStatus::Connected).await;

    tracing::info!("Socket initialized with session: {}", session_id);

    Ok(ConnectResponse {
        session_id,
        status: "connected".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// Connect to Socket.IO with authentication
#[tauri::command]
pub async fn socket_connect(
    app: AppHandle,
    request: ConnectRequest,
) -> Result<ConnectResponse, String> {
    let socket_state = app.state::<SocketAppState>();
    let session_id = request.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Create session
    socket_state.processor.create_session(&session_id, request.user_id.clone()).await;

    // Register listener for this session
    let _listener = socket_state.bridge.register_listener(&session_id).await;

    // Send connection event
    let connect_message = SocketMessage::new(
        SocketEventType::UserJoin,
        "user-join",
        serde_json::json!({
            "session_id": session_id,
            "user_id": request.user_id,
            "auth_token": request.auth_token,
            "timestamp": chrono::Utc::now().timestamp()
        }),
        &session_id,
    );

    let _ = socket_state.bridge.tauri_sender().send(connect_message);

    Ok(ConnectResponse {
        session_id,
        status: "connected".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// Disconnect from Socket.IO
#[tauri::command]
pub async fn socket_disconnect(
    app: AppHandle,
    session_id: String,
) -> Result<(), String> {
    let socket_state = app.state::<SocketAppState>();

    // Unregister listener
    socket_state.bridge.unregister_listener(&session_id).await;

    // Send disconnect event
    let disconnect_message = SocketMessage::new(
        SocketEventType::UserLeave,
        "user-leave",
        serde_json::json!({
            "session_id": session_id,
            "timestamp": chrono::Utc::now().timestamp()
        }),
        &session_id,
    );

    let _ = socket_state.bridge.tauri_sender().send(disconnect_message);

    tracing::info!("Disconnected session: {}", session_id);

    Ok(())
}

/// Join a room
#[tauri::command]
pub async fn join_room(
    app: AppHandle,
    request: JoinRoomRequest,
) -> Result<(), String> {
    let socket_state = app.state::<SocketAppState>();

    // Join room in processor
    socket_state.processor.join_room(&request.session_id, &request.room).await;

    // Send join event
    let join_message = SocketMessage::new(
        SocketEventType::ChannelJoin,
        "join-room",
        serde_json::json!({
            "session_id": request.session_id,
            "room": request.room,
            "timestamp": chrono::Utc::now().timestamp()
        }),
        &request.session_id,
    ).with_room(request.room.clone());

    let _ = socket_state.bridge.tauri_sender().send(join_message);

    tracing::info!("Session {} joined room: {}", request.session_id, request.room);

    Ok(())
}

/// Leave a room
#[tauri::command]
pub async fn leave_room(
    app: AppHandle,
    session_id: String,
    room: String,
) -> Result<(), String> {
    let socket_state = app.state::<SocketAppState>();

    // Send leave event
    let leave_message = SocketMessage::new(
        SocketEventType::ChannelLeave,
        "leave-room",
        serde_json::json!({
            "session_id": session_id,
            "room": room,
            "timestamp": chrono::Utc::now().timestamp()
        }),
        &session_id,
    ).with_room(room.clone());

    let _ = socket_state.bridge.tauri_sender().send(leave_message);

    tracing::info!("Session {} left room: {}", session_id, room);

    Ok(())
}

/// Emit an event
#[tauri::command]
pub async fn emit_event(
    app: AppHandle,
    request: EmitEventRequest,
) -> Result<(), String> {
    let socket_state = app.state::<SocketAppState>();

    // Parse event type
    let event_type = match request.event_type.as_str() {
        "user_join" => SocketEventType::UserJoin,
        "user_leave" => SocketEventType::UserLeave,
        "chat_event" => SocketEventType::ChatEvent,
        "channel_event" => SocketEventType::ChannelEvent,
        "channel_join" => SocketEventType::ChannelJoin,
        "channel_leave" => SocketEventType::ChannelLeave,
        "ydoc_join" => SocketEventType::YdocJoin,
        "ydoc_leave" => SocketEventType::YdocLeave,
        "ydoc_update" => SocketEventType::YdocUpdate,
        "ydoc_state_request" => SocketEventType::YdocStateRequest,
        "presence_status" => SocketEventType::PresenceStatus,
        "typing_start" => SocketEventType::TypingStart,
        "typing_stop" => SocketEventType::TypingStop,
        "usage" => SocketEventType::Usage,
        _ => return Err(format!("Unknown event type: {}", request.event_type)),
    };

    let session_id = request.session_id.unwrap_or_else(|| "anonymous".to_string());
    let mut message = SocketMessage::new(event_type, &request.event_name, request.data, &session_id);

    if let Some(room) = request.room {
        message = message.with_room(room);
    }

    let _ = socket_state.bridge.tauri_sender().send(message);

    tracing::debug!("Emitted event: {}", request.event_name);

    Ok(())
}

/// Get connection status
#[tauri::command]
pub async fn get_connection_status(
    app: AppHandle,
) -> Result<JsonValue, String> {
    let socket_state = app.state::<SocketAppState>();

    let status = socket_state.bridge.get_connection_status().await;

    Ok(serde_json::json!({
        "status": status,
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// Send a chat message
#[tauri::command]
pub async fn send_chat_message(
    app: AppHandle,
    session_id: String,
    chat_id: String,
    message: String,
) -> Result<(), String> {
    let socket_state = app.state::<SocketAppState>();

    let chat_message = SocketMessage::new(
        SocketEventType::ChatEvent,
        "chat-message",
        serde_json::json!({
            "chat_id": chat_id,
            "message": message,
            "timestamp": chrono::Utc::now().timestamp(),
            "session_id": session_id
        }),
        &session_id,
    ).with_room(format!("chat:{}", chat_id));

    let _ = socket_state.bridge.tauri_sender().send(chat_message);

    tracing::info!("Chat message sent to chat: {}", chat_id);

    Ok(())
}

/// Send typing indicator
#[tauri::command]
pub async fn send_typing_indicator(
    app: AppHandle,
    session_id: String,
    room: String,
    is_typing: bool,
) -> Result<(), String> {
    let socket_state = app.state::<SocketAppState>();

    let event_type = if is_typing {
        SocketEventType::TypingStart
    } else {
        SocketEventType::TypingStop
    };

    let typing_message = SocketMessage::new(
        event_type,
        if is_typing { "typing-start" } else { "typing-stop" },
        serde_json::json!({
            "session_id": session_id,
            "room": room,
            "timestamp": chrono::Utc::now().timestamp()
        }),
        &session_id,
    ).with_room(room.clone());

    let _ = socket_state.bridge.tauri_sender().send(typing_message);

    tracing::debug!("Typing indicator sent to room: {} (typing: {})", room, is_typing);

    Ok(())
}

/// Update user presence status
#[tauri::command]
pub async fn update_presence_status(
    app: AppHandle,
    session_id: String,
    status: String,
) -> Result<(), String> {
    let socket_state = app.state::<SocketAppState>();

    let presence_message = SocketMessage::new(
        SocketEventType::PresenceStatus,
        "presence-status",
        serde_json::json!({
            "session_id": session_id,
            "status": status,
            "timestamp": chrono::Utc::now().timestamp()
        }),
        &session_id,
    );

    let _ = socket_state.bridge.tauri_sender().send(presence_message);

    tracing::info!("Presence status updated for session {}: {}", session_id, status);

    Ok(())
}

/// Join a collaborative document
#[tauri::command]
pub async fn join_document(
    app: AppHandle,
    session_id: String,
    document_id: String,
) -> Result<(), String> {
    let socket_state = app.state::<SocketAppState>();

    // Join document room
    let room = format!("doc:{}", document_id);
    socket_state.processor.join_room(&session_id, &room).await;

    let join_message = SocketMessage::new(
        SocketEventType::YdocJoin,
        "ydoc-join",
        serde_json::json!({
            "document_id": document_id,
            "session_id": session_id,
            "timestamp": chrono::Utc::now().timestamp()
        }),
        &session_id,
    ).with_room(room);

    let _ = socket_state.bridge.tauri_sender().send(join_message);

    tracing::info!("Session {} joined document: {}", session_id, document_id);

    Ok(())
}

/// Send document update
#[tauri::command]
pub async fn send_document_update(
    app: AppHandle,
    session_id: String,
    document_id: String,
    update: JsonValue,
) -> Result<(), String> {
    let socket_state = app.state::<SocketAppState>();

    let room = format!("doc:{}", document_id);

    let update_message = SocketMessage::new(
        SocketEventType::YdocUpdate,
        "ydoc-update",
        serde_json::json!({
            "document_id": document_id,
            "update": update,
            "session_id": session_id,
            "timestamp": chrono::Utc::now().timestamp()
        }),
        &session_id,
    ).with_room(room);

    let _ = socket_state.bridge.tauri_sender().send(update_message);

    tracing::debug!("Document update sent for: {}", document_id);

    Ok(())
}

/// Get socket statistics
#[tauri::command]
pub async fn get_socket_stats(
    app: AppHandle,
) -> Result<JsonValue, String> {
    let socket_state = app.state::<SocketAppState>();

    // In a real implementation, this would return actual statistics
    let stats = serde_json::json!({
        "active_sessions": 1, // Mock value
        "total_connections": 1,
        "rooms_count": 0,
        "messages_sent": 0,
        "uptime": chrono::Utc::now().timestamp(),
        "status": socket_state.bridge.get_connection_status().await
    });

    Ok(stats)
}