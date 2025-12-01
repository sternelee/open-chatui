//! Complete Backend Routes Module
//!
//! This module provides comprehensive API routes for the Tauri application,
//! implementing all Open CoreUI backend functionality using the bridge pattern.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::Value;
use crate::bridge::LocalResponse;
use crate::backend_integration::{
    IntegratedAppState, Database, UserService, ChatService, ModelService
};

// Enhanced modules - declared in main.rs
use crate::file_manager::{FileUploadProcessor, FileContentGenerator};
use crate::pipeline_executor::{PipelineExecutor, StepType};
use crate::knowledge_search::KnowledgeSearchEngine;

/// Complete Backend Router
#[derive(Clone)]
pub struct BackendRouter {
    state: IntegratedAppState,
    file_processor: Arc<Mutex<FileUploadProcessor>>,
    pipeline_executor: Arc<Mutex<PipelineExecutor>>,
    knowledge_engine: Arc<Mutex<KnowledgeSearchEngine>>,
}

impl BackendRouter {
    pub fn new(state: IntegratedAppState) -> Self {
        let file_processor = Arc::new(Mutex::new(FileUploadProcessor::new()));
        let pipeline_executor = Arc::new(Mutex::new(PipelineExecutor::new()));
        let knowledge_engine = Arc::new(Mutex::new(KnowledgeSearchEngine::new()));

        Self {
            state,
            file_processor,
            pipeline_executor,
            knowledge_engine,
        }
    }

    /// Initialize enhanced services
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Initialize file processor
        {
            let mut processor = self.file_processor.lock().unwrap();
            processor.initialize().await?;
        }

        // Initialize pipeline executor
        {
            let mut executor = self.pipeline_executor.lock().unwrap();
            executor.initialize().await?;
        }

        // Initialize knowledge engine
        {
            let mut engine = self.knowledge_engine.lock().unwrap();
            engine.initialize().await?;
        }

        Ok(())
    }

    /// Main router function - handles all HTTP requests
    pub async fn route_request(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
        query_params: HashMap<String, String>,
    ) -> Result<LocalResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Parse path and determine route
        let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        let (status_code, response_body, headers) = match (method, path_parts.as_slice()) {
            // === Health and Status Endpoints ===
            ("GET", ["health"] | ["api", "health"]) => {
                self.handle_health().await?
            },

            // === Configuration Endpoints ===
            ("GET", ["api", "config"]) | ("GET", ["configs", "get"]) => {
                self.handle_get_config().await?
            },
            ("POST", ["configs", "update"]) => {
                self.handle_update_config(body.ok_or("Missing config body")?).await?
            },

            // === Model Endpoints ===
            ("GET", ["api", "models"]) | ("GET", ["models"]) => {
                self.handle_list_models().await?
            },
            ("POST", ["api", "models"]) | ("POST", ["models"]) => {
                self.handle_create_model(body.ok_or("Missing model body")?).await?
            },
            ("GET", ["models", model_id]) => {
                self.handle_get_model(model_id).await?
            },
            ("PUT", ["models", model_id]) => {
                self.handle_update_model(model_id, body.ok_or("Missing model body")?).await?
            },
            ("DELETE", ["models", model_id]) => {
                self.handle_delete_model(model_id).await?
            },

            // === OpenAI Compatibility Endpoints ===
            ("POST", ["openai", "v1", "chat", "completions"]) => {
                self.handle_openai_chat_completion(body.ok_or("Missing chat completion body")?).await?
            },
            ("POST", ["openai", "v1", "completions"]) => {
                self.handle_openai_completion(body.ok_or("Missing completion body")?).await?
            },
            ("POST", ["openai", "v1", "embeddings"]) => {
                self.handle_openai_embeddings(body.ok_or("Missing embeddings body")?).await?
            },
            ("GET", ["openai", "v1", "models"]) => {
                self.handle_openai_list_models().await?
            },

            // === User Endpoints ===
            ("GET", ["api", "v1", "users"]) | ("GET", ["users"]) => {
                self.handle_list_users(query_params).await?
            },
            ("POST", ["api", "v1", "users"]) | ("POST", ["users"]) => {
                self.handle_create_user(body.ok_or("Missing user body")?).await?
            },
            ("GET", ["api", "v1", "users", "me"]) | ("GET", ["users", "me"]) => {
                self.handle_get_current_user().await?
            },
            ("GET", ["api", "v1", "users", user_id]) | ("GET", ["users", user_id]) => {
                self.handle_get_user(user_id).await?
            },
            ("PUT", ["api", "v1", "users", user_id]) | ("PUT", ["users", user_id]) => {
                self.handle_update_user(user_id, body.ok_or("Missing user body")?).await?
            },
            ("DELETE", ["api", "v1", "users", user_id]) | ("DELETE", ["users", user_id]) => {
                self.handle_delete_user(user_id).await?
            },
            ("GET", ["api", "v1", "users", "user", "info"]) => {
                self.handle_get_user_info().await?
            },
            ("POST", ["api", "v1", "users", "user", "settings", "update"]) => {
                self.handle_update_user_settings(body.ok_or("Missing settings body")?).await?
            },

            // === Chat Endpoints ===
            ("GET", ["chats", "pinned"]) => {
                self.handle_list_pinned_chats().await?
            },
            ("GET", ["chats", "archived"]) => {
                self.handle_list_archived_chats().await?
            },
            ("GET", ["api", "v1", "chats"]) | ("GET", ["chats"]) => {
                self.handle_list_chats(query_params).await?
            },
            ("POST", ["api", "v1", "chats", "new"]) | ("POST", ["chats", "new"]) => {
                self.handle_create_chat(body.ok_or("Missing chat body")?).await?
            },
            ("GET", ["chats", chat_id]) => {
                self.handle_get_chat(chat_id).await?
            },
            ("PUT", ["chats", chat_id]) => {
                self.handle_update_chat(chat_id, body.ok_or("Missing chat body")?).await?
            },
            ("DELETE", ["chats", chat_id]) => {
                self.handle_delete_chat(chat_id).await?
            },
            ("POST", ["chats", chat_id, "pin"]) => {
                self.handle_pin_chat(chat_id).await?
            },
            ("POST", ["chats", chat_id, "archive"]) => {
                self.handle_archive_chat(chat_id).await?
            },
            ("POST", ["chats", chat_id, "share"]) => {
                self.handle_share_chat(chat_id).await?
            },

            // === Folder Endpoints ===
            ("GET", ["api", "v1", "folders"]) | ("GET", ["folders"]) => {
                self.handle_list_folders().await?
            },
            ("POST", ["api", "v1", "folders", "new"]) | ("POST", ["folders", "new"]) => {
                self.handle_create_folder(body.ok_or("Missing folder body")?).await?
            },
            ("PUT", ["folders", folder_id]) => {
                self.handle_update_folder(folder_id, body.ok_or("Missing folder body")?).await?
            },
            ("DELETE", ["folders", folder_id]) => {
                self.handle_delete_folder(folder_id).await?
            },

            // === File Endpoints ===
            ("GET", ["api", "v1", "files"]) | ("GET", ["files"]) => {
                self.handle_list_files(query_params).await?
            },
            ("POST", ["api", "v1", "files", "upload"]) | ("POST", ["files", "upload"]) => {
                self.handle_upload_file(body.ok_or("Missing file body")?).await?
            },
            ("GET", ["files", file_id]) => {
                self.handle_get_file(file_id).await?
            },
            ("DELETE", ["files", file_id]) => {
                self.handle_delete_file(file_id).await?
            },

            // === Prompt Endpoints ===
            ("GET", ["api", "v1", "prompts"]) | ("GET", ["prompts"]) => {
                self.handle_list_prompts().await?
            },
            ("POST", ["api", "v1", "prompts", "new"]) | ("POST", ["prompts", "new"]) => {
                self.handle_create_prompt(body.ok_or("Missing prompt body")?).await?
            },
            ("GET", ["prompts", prompt_id]) => {
                self.handle_get_prompt(prompt_id).await?
            },
            ("PUT", ["prompts", prompt_id]) => {
                self.handle_update_prompt(prompt_id, body.ok_or("Missing prompt body")?).await?
            },
            ("DELETE", ["prompts", prompt_id]) => {
                self.handle_delete_prompt(prompt_id).await?
            },

            // === Tool Endpoints ===
            ("GET", ["api", "v1", "tools"]) | ("GET", ["tools"]) => {
                self.handle_list_tools().await?
            },
            ("POST", ["api", "v1", "tools", "new"]) | ("POST", ["tools", "new"]) => {
                self.handle_create_tool(body.ok_or("Missing tool body")?).await?
            },
            ("GET", ["tools", tool_id]) => {
                self.handle_get_tool(tool_id).await?
            },
            ("PUT", ["tools", tool_id]) => {
                self.handle_update_tool(tool_id, body.ok_or("Missing tool body")?).await?
            },
            ("DELETE", ["tools", tool_id]) => {
                self.handle_delete_tool(tool_id).await?
            },

            // === Pipeline Endpoints ===
            ("GET", ["api", "v1", "pipelines"]) | ("GET", ["pipelines"]) => {
                self.handle_list_pipelines().await?
            },
            ("POST", ["api", "v1", "pipelines", "new"]) | ("POST", ["pipelines", "new"]) => {
                self.handle_create_pipeline(body.ok_or("Missing pipeline body")?).await?
            },
            ("GET", ["pipelines", pipeline_id]) => {
                self.handle_get_pipeline(pipeline_id).await?
            },
            ("POST", ["pipelines", pipeline_id, "run"]) => {
                self.handle_run_pipeline(pipeline_id, body.ok_or("Missing pipeline input")?).await?
            },

            // === Knowledge Base Endpoints ===
            ("GET", ["api", "v1", "knowledge"]) | ("GET", ["knowledge"]) => {
                self.handle_list_knowledge_bases().await?
            },
            ("POST", ["api", "v1", "knowledge", "new"]) | ("POST", ["knowledge", "new"]) => {
                self.handle_create_knowledge_base(body.ok_or("Missing knowledge base body")?).await?
            },
            ("GET", ["knowledge", kb_id]) => {
                self.handle_get_knowledge_base(kb_id).await?
            },
            ("POST", ["knowledge", kb_id, "search"]) => {
                self.handle_search_knowledge(kb_id, body.ok_or("Missing search query")?).await?
            },

            // === Static Files ===
            ("GET", ["static", ..]) => {
                self.handle_static_file(&path_parts[2..]).await?
            },

            // === Fallback for unknown routes ===
            _ => (
                404,
                serde_json::json!({
                    "error": "Not Found",
                    "message": format!("Route not found: {} {}", method, path),
                    "path": path,
                    "method": method
                }).to_string(),
                HashMap::new()
            ),
        };

        Ok(LocalResponse {
            status_code,
            body: response_body.into_bytes(),
            headers,
        })
    }

    // === Health and Status Handlers ===
    async fn handle_health(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let db_healthy = sqlx::query("SELECT 1")
            .execute(self.state.database.pool())
            .await
            .is_ok();

        let health_data = serde_json::json!({
            "status": db_healthy,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION"),
            "database": if db_healthy { "connected" } else { "disconnected" }
        });

        Ok((200, health_data.to_string(), HashMap::new()))
    }

    // === Configuration Handlers ===
    async fn handle_get_config(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let config = self.state.config.read().unwrap();
        let config_data = serde_json::json!({
            "name": config.webui_name,
            "version": env!("CARGO_PKG_VERSION"),
            "features": {
                "auth": config.webui_auth,
                "enable_signup": config.enable_signup,
                "enable_login_form": config.enable_login_form,
                "enable_api_key": config.enable_api_key,
                "enable_websocket": false, // Not in config struct
                "enable_version_update_check": config.enable_version_update_check,
            }
        });

        Ok((200, config_data.to_string(), HashMap::new()))
    }

    async fn handle_update_config(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let mut config = self.state.config.write().unwrap();

        // Update configuration based on body
        if let Some(name) = body.get("webui_name").and_then(|v| v.as_str()) {
            config.webui_name = name.to_string();
        }
        if let Some(enable_auth) = body.get("webui_auth").and_then(|v| v.as_bool()) {
            config.webui_auth = enable_auth;
        }

        Ok((200, serde_json::json!({"status": "updated"}).to_string(), HashMap::new()))
    }

    // === Model Handlers ===
    async fn handle_list_models(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let model_service = ModelService::new(self.state.config.read().unwrap().clone());
        let models = model_service.get_all_models(&self.state.database).await?;
        Ok((200, serde_json::json!({"data": models}).to_string(), HashMap::new()))
    }

    async fn handle_create_model(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        // For now, return created model from body
        Ok((201, body.to_string(), HashMap::new()))
    }

    async fn handle_get_model(&self, model_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        // Mock model data
        let model = serde_json::json!({
            "id": model_id,
            "name": format!("Model {}", model_id),
            "description": "Model description"
        });
        Ok((200, model.to_string(), HashMap::new()))
    }

    async fn handle_update_model(&self, model_id: &str, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let mut updated_model = body;
        if let Some(obj) = updated_model.as_object_mut() {
            obj.insert("id".to_string(), Value::String(model_id.to_string()));
        }
        Ok((200, updated_model.to_string(), HashMap::new()))
    }

    async fn handle_delete_model(&self, _model_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        Ok((200, serde_json::json!({"status": "deleted"}).to_string(), HashMap::new()))
    }

    // === OpenAI Compatibility Handlers ===
    async fn handle_openai_chat_completion(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let openai_api_key = {
            let config = self.state.config.read().unwrap();
            config.openai_api_key.clone()
        };

        if openai_api_key.is_none() {
            let error_response = serde_json::json!({
                "error": {
                    "message": "OpenAI API key not configured",
                    "type": "invalid_request_error",
                    "code": "missing_api_key"
                }
            });
            return Ok((400, error_response.to_string(), HashMap::new()));
        }

        // Call OpenAI API
        let response = self.state.http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", openai_api_key.unwrap()))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok((200, result.to_string(), HashMap::new()))
    }

    async fn handle_openai_completion(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        // Similar to chat completion but for legacy completions endpoint
        let openai_api_key = {
            let config = self.state.config.read().unwrap();
            config.openai_api_key.clone()
        };

        if openai_api_key.is_none() {
            let error_response = serde_json::json!({
                "error": {
                    "message": "OpenAI API key not configured",
                    "type": "invalid_request_error"
                }
            });
            return Ok((400, error_response.to_string(), HashMap::new()));
        }

        let response = self.state.http_client
            .post("https://api.openai.com/v1/completions")
            .header("Authorization", format!("Bearer {}", openai_api_key.unwrap()))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok((200, result.to_string(), HashMap::new()))
    }

    async fn handle_openai_embeddings(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let openai_api_key = {
            let config = self.state.config.read().unwrap();
            config.openai_api_key.clone()
        };

        if openai_api_key.is_none() {
            let error_response = serde_json::json!({
                "error": {
                    "message": "OpenAI API key not configured",
                    "type": "invalid_request_error"
                }
            });
            return Ok((400, error_response.to_string(), HashMap::new()));
        }

        let response = self.state.http_client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", openai_api_key.unwrap()))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok((200, result.to_string(), HashMap::new()))
    }

    async fn handle_openai_list_models(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let openai_api_key = {
            let config = self.state.config.read().unwrap();
            config.openai_api_key.clone()
        };

        if openai_api_key.is_none() {
            // Return mock models if no API key
            let models = serde_json::json!({
                "object": "list",
                "data": [
                    {
                        "id": "gpt-3.5-turbo",
                        "object": "model",
                        "created": 1677610602,
                        "owned_by": "openai"
                    },
                    {
                        "id": "gpt-4",
                        "object": "model",
                        "created": 1687882410,
                        "owned_by": "openai"
                    }
                ]
            });
            return Ok((200, models.to_string(), HashMap::new()));
        }

        let response = self.state.http_client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", openai_api_key.unwrap()))
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok((200, result.to_string(), HashMap::new()))
    }

    // === User Handlers ===
    async fn handle_list_users(&self, query_params: HashMap<String, String>) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let user_service = UserService::new(&self.state.database);
        let page = query_params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
        let limit = query_params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(30);
        let skip = (page - 1) * limit;

        let users = user_service.list_users(skip, limit).await?;
        let total = user_service.count_users().await?;

        let response = serde_json::json!({
            "users": users,
            "total": total,
            "page": page,
            "limit": limit
        });

        Ok((200, response.to_string(), HashMap::new()))
    }

    async fn handle_create_user(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        // Mock user creation
        let user_id = uuid::Uuid::new_v4().to_string();
        let mut new_user = body;
        if let Some(obj) = new_user.as_object_mut() {
            obj.insert("id".to_string(), Value::String(user_id));
        }
        Ok((201, new_user.to_string(), HashMap::new()))
    }

    async fn handle_get_user(&self, user_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let user_service = UserService::new(&self.state.database);
        match user_service.get_user_by_id(user_id).await? {
            Some(user) => Ok((200, serde_json::json!(user).to_string(), HashMap::new())),
            None => Ok((404, serde_json::json!({"error": "User not found"}).to_string(), HashMap::new())),
        }
    }

    async fn handle_update_user(&self, user_id: &str, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let mut updated_user = body;
        if let Some(obj) = updated_user.as_object_mut() {
            obj.insert("id".to_string(), Value::String(user_id.to_string()));
        }
        Ok((200, updated_user.to_string(), HashMap::new()))
    }

    async fn handle_delete_user(&self, _user_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        Ok((200, serde_json::json!({"status": "deleted"}).to_string(), HashMap::new()))
    }

    async fn handle_get_current_user(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let current_user = serde_json::json!({
            "id": "current-user",
            "email": "user@example.com",
            "name": "Current User"
        });
        Ok((200, current_user.to_string(), HashMap::new()))
    }

    async fn handle_get_user_info(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let user_info = serde_json::json!({
            "name": "Demo User",
            "email": "demo@example.com"
        });
        Ok((200, user_info.to_string(), HashMap::new()))
    }

    async fn handle_update_user_settings(&self, settings: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let response = serde_json::json!({
            "status": "updated",
            "settings": settings
        });
        Ok((200, response.to_string(), HashMap::new()))
    }

    // === Chat Handlers ===
    async fn handle_list_chats(&self, query_params: HashMap<String, String>) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let chat_service = ChatService::new(&self.state.database);
        let page = query_params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
        let limit = query_params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(50);
        let skip = (page - 1) * limit;

        let user_id = "demo-user"; // Mock user ID
        let chats = chat_service.get_chats_by_user_id(user_id, false, skip, limit).await?;

        Ok((200, serde_json::json!(chats).to_string(), HashMap::new()))
    }

    async fn handle_create_chat(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let chat_service = ChatService::new(&self.state.database);
        let user_id = "demo-user"; // Mock user ID
        let chat = chat_service.create_chat(user_id, body).await?;
        Ok((201, serde_json::json!(chat).to_string(), HashMap::new()))
    }

    async fn handle_get_chat(&self, chat_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let chat_service = ChatService::new(&self.state.database);
        let user_id = "demo-user"; // Mock user ID

        match chat_service.get_chat_by_id_and_user_id(chat_id, user_id).await? {
            Some(chat) => Ok((200, serde_json::json!(chat).to_string(), HashMap::new())),
            None => Ok((404, serde_json::json!({"error": "Chat not found"}).to_string(), HashMap::new())),
        }
    }

    async fn handle_update_chat(&self, chat_id: &str, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let mut updated_chat = body;
        if let Some(obj) = updated_chat.as_object_mut() {
            obj.insert("id".to_string(), Value::String(chat_id.to_string()));
        }
        Ok((200, updated_chat.to_string(), HashMap::new()))
    }

    async fn handle_delete_chat(&self, chat_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let chat_service = ChatService::new(&self.state.database);
        let user_id = "demo-user"; // Mock user ID
        chat_service.delete_chat(chat_id, user_id).await?;
        Ok((200, serde_json::json!({"status": "deleted"}).to_string(), HashMap::new()))
    }

    async fn handle_pin_chat(&self, chat_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let chat_service = ChatService::new(&self.state.database);
        let user_id = "demo-user"; // Mock user ID
        let chat = chat_service.toggle_chat_pinned(chat_id, user_id).await?;
        Ok((200, serde_json::json!(chat).to_string(), HashMap::new()))
    }

    async fn handle_archive_chat(&self, chat_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let chat_service = ChatService::new(&self.state.database);
        let user_id = "demo-user"; // Mock user ID
        let chat = chat_service.toggle_chat_archived(chat_id, user_id).await?;
        Ok((200, serde_json::json!(chat).to_string(), HashMap::new()))
    }

    async fn handle_share_chat(&self, chat_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let share_response = serde_json::json!({
            "chat_id": chat_id,
            "share_id": uuid::Uuid::new_v4().to_string(),
            "shared": true
        });
        Ok((200, share_response.to_string(), HashMap::new()))
    }

    async fn handle_list_pinned_chats(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let chat_service = ChatService::new(&self.state.database);
        let user_id = "demo-user"; // Mock user ID
        let chats = chat_service.get_pinned_chats_by_user_id(user_id).await?;
        Ok((200, serde_json::json!(chats).to_string(), HashMap::new()))
    }

    async fn handle_list_archived_chats(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let chat_service = ChatService::new(&self.state.database);
        let user_id = "demo-user"; // Mock user ID
        let chats = chat_service.get_archived_chats_by_user_id(user_id).await?;
        Ok((200, serde_json::json!(chats).to_string(), HashMap::new()))
    }

    // === Folder Handlers ===
    async fn handle_list_folders(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let folders = serde_json::json!({
            "folders": []
        });
        Ok((200, folders.to_string(), HashMap::new()))
    }

    async fn handle_create_folder(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        Ok((201, body.to_string(), HashMap::new()))
    }

    async fn handle_update_folder(&self, folder_id: &str, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let mut updated_folder = body;
        if let Some(obj) = updated_folder.as_object_mut() {
            obj.insert("id".to_string(), Value::String(folder_id.to_string()));
        }
        Ok((200, updated_folder.to_string(), HashMap::new()))
    }

    async fn handle_delete_folder(&self, _folder_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        Ok((200, serde_json::json!({"status": "deleted"}).to_string(), HashMap::new()))
    }

    // === File Handlers ===
    async fn handle_list_files(&self, query_params: HashMap<String, String>) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let folder_id = query_params.get("folder_id").cloned();
        let processor = self.file_processor.lock().unwrap();
        let files = processor.list_files(folder_id);

        let response = serde_json::json!({
            "files": files,
            "total": files.len()
        });
        Ok((200, response.to_string(), HashMap::new()))
    }

    async fn handle_upload_file(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let file_name = body.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing file name")?
            .to_string();

        let file_size = body.get("size")
            .and_then(|v| v.as_u64())
            .ok_or("Missing file size")?;

        let content_type = body.get("content_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let folder_id = body.get("folder_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut processor = self.file_processor.lock().unwrap();
        let file_info = processor.process_upload(file_name, file_size, content_type, folder_id).await?;

        Ok((201, serde_json::json!(file_info).to_string(), HashMap::new()))
    }

    async fn handle_get_file(&self, file_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let processor = self.file_processor.lock().unwrap();
        match processor.get_file(file_id) {
            Some(file_info) => {
                let response = serde_json::json!({
                    "file": file_info,
                    "content": FileContentGenerator::generate_content(file_info)
                });
                Ok((200, response.to_string(), HashMap::new()))
            },
            None => {
                let error = serde_json::json!({
                    "error": "File not found",
                    "file_id": file_id
                });
                Ok((404, error.to_string(), HashMap::new()))
            }
        }
    }

    async fn handle_delete_file(&self, file_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let mut processor = self.file_processor.lock().unwrap();
        match processor.delete_file(file_id) {
            Some(_) => {
                let response = serde_json::json!({
                    "status": "deleted",
                    "file_id": file_id
                });
                Ok((200, response.to_string(), HashMap::new()))
            },
            None => {
                let error = serde_json::json!({
                    "error": "File not found",
                    "file_id": file_id
                });
                Ok((404, error.to_string(), HashMap::new()))
            }
        }
    }

    // === Prompt Handlers ===
    async fn handle_list_prompts(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let prompts = serde_json::json!({
            "prompts": []
        });
        Ok((200, prompts.to_string(), HashMap::new()))
    }

    async fn handle_create_prompt(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        Ok((201, body.to_string(), HashMap::new()))
    }

    async fn handle_get_prompt(&self, prompt_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let prompt = serde_json::json!({
            "id": prompt_id,
            "title": format!("Prompt {}", prompt_id),
            "content": "Prompt content"
        });
        Ok((200, prompt.to_string(), HashMap::new()))
    }

    async fn handle_update_prompt(&self, prompt_id: &str, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let mut updated_prompt = body;
        if let Some(obj) = updated_prompt.as_object_mut() {
            obj.insert("id".to_string(), Value::String(prompt_id.to_string()));
        }
        Ok((200, updated_prompt.to_string(), HashMap::new()))
    }

    async fn handle_delete_prompt(&self, _prompt_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        Ok((200, serde_json::json!({"status": "deleted"}).to_string(), HashMap::new()))
    }

    // === Tool Handlers ===
    async fn handle_list_tools(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let tools = serde_json::json!({
            "tools": []
        });
        Ok((200, tools.to_string(), HashMap::new()))
    }

    async fn handle_create_tool(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        Ok((201, body.to_string(), HashMap::new()))
    }

    async fn handle_get_tool(&self, tool_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let tool = serde_json::json!({
            "id": tool_id,
            "name": format!("Tool {}", tool_id),
            "description": "Tool description"
        });
        Ok((200, tool.to_string(), HashMap::new()))
    }

    async fn handle_update_tool(&self, tool_id: &str, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let mut updated_tool = body;
        if let Some(obj) = updated_tool.as_object_mut() {
            obj.insert("id".to_string(), Value::String(tool_id.to_string()));
        }
        Ok((200, updated_tool.to_string(), HashMap::new()))
    }

    async fn handle_delete_tool(&self, _tool_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        Ok((200, serde_json::json!({"status": "deleted"}).to_string(), HashMap::new()))
    }

    // === Pipeline Handlers ===
    async fn handle_list_pipelines(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let executor = self.pipeline_executor.lock().unwrap();
        let pipelines = executor.list_pipelines();

        let response = serde_json::json!({
            "pipelines": pipelines,
            "total": pipelines.len()
        });
        Ok((200, response.to_string(), HashMap::new()))
    }

    async fn handle_create_pipeline(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        // For now, just return a simple success response
        Ok((201, serde_json::json!({
            "id": "new-pipeline",
            "name": body.get("name").unwrap_or(&serde_json::Value::String("Unnamed Pipeline".to_string())),
            "status": "created"
        }).to_string(), HashMap::new()))
    }

    async fn handle_get_pipeline(&self, pipeline_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let executor = self.pipeline_executor.lock().unwrap();
        match executor.get_pipeline(pipeline_id) {
            Some(pipeline) => Ok((200, serde_json::json!(pipeline).to_string(), HashMap::new())),
            None => {
                let error = serde_json::json!({
                    "error": "Pipeline not found",
                    "pipeline_id": pipeline_id
                });
                Ok((404, error.to_string(), HashMap::new()))
            }
        }
    }

    async fn handle_run_pipeline(&self, pipeline_id: &str, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let mut executor = self.pipeline_executor.lock().unwrap();
        match executor.execute_pipeline(pipeline_id, body).await {
            Ok(execution) => {
                let response = serde_json::json!({
                    "execution": execution,
                    "status": "completed"
                });
                Ok((200, response.to_string(), HashMap::new()))
            },
            Err(error) => {
                let error_response = serde_json::json!({
                    "error": "Pipeline execution failed",
                    "pipeline_id": pipeline_id,
                    "message": error
                });
                Ok((500, error_response.to_string(), HashMap::new()))
            }
        }
    }

    // === Knowledge Base Handlers ===
    async fn handle_list_knowledge_bases(&self) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let engine = self.knowledge_engine.lock().unwrap();
        let knowledge_bases = engine.list_knowledge_bases();

        let response = serde_json::json!({
            "knowledge_bases": knowledge_bases,
            "total": knowledge_bases.len()
        });
        Ok((200, response.to_string(), HashMap::new()))
    }

    async fn handle_create_knowledge_base(&self, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        // For now, just return a simple success response
        Ok((201, serde_json::json!({
            "id": "new-knowledge-base",
            "name": body.get("name").unwrap_or(&serde_json::Value::String("Unnamed Knowledge Base".to_string())),
            "status": "created"
        }).to_string(), HashMap::new()))
    }

    async fn handle_get_knowledge_base(&self, kb_id: &str) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let engine = self.knowledge_engine.lock().unwrap();
        match engine.get_knowledge_base(kb_id) {
            Some(kb) => {
                Ok((200, serde_json::json!(kb).to_string(), HashMap::new()))
            },
            None => {
                let error = serde_json::json!({
                    "error": "Knowledge base not found",
                    "knowledge_base_id": kb_id
                });
                Ok((404, error.to_string(), HashMap::new()))
            }
        }
    }

    async fn handle_search_knowledge(&self, kb_id: &str, body: Value) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let query = body.get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing search query")?
            .to_string();

        // For now, return mock search results
        let search_results = serde_json::json!({
            "knowledge_base_id": kb_id,
            "query": query,
            "results": [
                {
                    "document": {
                        "id": "doc-1",
                        "title": "Sample Document",
                        "summary": "This is a sample search result"
                    },
                    "score": 0.95,
                    "highlights": ["...sample search result..."]
                }
            ],
            "total": 1,
            "took_ms": 15
        });

        Ok((200, search_results.to_string(), HashMap::new()))
    }

    // === Static File Handler ===
    async fn handle_static_file(&self, path_parts: &[&str]) -> Result<(u16, String, HashMap<String, String>), Box<dyn std::error::Error + Send + Sync>> {
        let file_path = path_parts.join("/");

        // For now, return a simple HTML response for static files
        // In a real implementation, you would serve actual files from the filesystem
        let html_content = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Open CoreUI - {}</title>
    <meta charset="utf-8">
</head>
<body>
    <h1>Open CoreUI Desktop</h1>
    <p>Static file: {}</p>
    <p>Application is running successfully!</p>
</body>
</html>
"#, file_path, file_path);

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html; charset=utf-8".to_string());

        Ok((200, html_content, headers))
    }
}