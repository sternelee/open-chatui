//! Backend Bridge Module for Tauri Integration
//!
//! This module implements the tauri-actix-web pattern for processing HTTP requests
//! within the Tauri application using an Actix-web server.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::backend_routes::BackendRouter;

/// Represents an HTTP request that can be processed by an Actix-web server
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalRequest {
    pub uri: String,
    pub method: String,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

/// Represents an HTTP response returned from an Actix-web server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalResponse {
    pub status_code: u16,
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
}

/// Application state containing the backend router
#[derive(Clone)]
pub struct AppState {
    // Backend router for handling all API requests
    pub backend_router: BackendRouter,
}

impl LocalRequest {
    /// Process the request using the integrated backend router
    pub async fn process_with_backend(self, backend_router: &BackendRouter) -> LocalResponse {
        // Parse query parameters from the URI
        let query_params = if let Some(query_start) = self.uri.find('?') {
            let query_part = &self.uri[query_start + 1..];
            let mut params = HashMap::new();
            for pair in query_part.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    params.insert(key.to_string(), value.to_string());
                }
            }
            params
        } else {
            HashMap::new()
        };

        // Extract path without query parameters
        let path = if let Some(query_start) = self.uri.find('?') {
            &self.uri[..query_start]
        } else {
            &self.uri
        };

        // Route the request using the backend router
        match backend_router.route_request(&self.method, path,
            self.body.and_then(|b| serde_json::from_str(&b).ok()),
            query_params).await {
            Ok(response) => response,
            Err(e) => LocalResponse {
                status_code: 500,
                body: format!("Internal Server Error: {}", e).into_bytes(),
                headers: HashMap::new(),
            }
        }
    }
}

impl LocalResponse {

    /// Create an internal server error response
    pub fn internal_server_error(error: String) -> Self {
        LocalResponse {
            status_code: 500,
            body: format!("Internal Server Error: {}", error).into_bytes(),
            headers: HashMap::new(),
        }
    }

    /// Create a successful response with JSON body
    pub fn json<T: Serialize>(data: T) -> Result<Self, serde_json::Error> {
        let json_bytes = serde_json::to_vec(&data)?;
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Ok(LocalResponse {
            status_code: 200,
            body: json_bytes,
            headers,
        })
    }

    /// Create a successful response with text body
    pub fn text(text: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());

        LocalResponse {
            status_code: 200,
            body: text.into_bytes(),
            headers,
        }
    }

    /// Create a successful response with HTML body
    pub fn html(html: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html; charset=utf-8".to_string());

        LocalResponse {
            status_code: 200,
            body: html.into_bytes(),
            headers,
        }
    }
}

/// Process local app requests (can be used from main.rs)
pub async fn process_local_request(
    state: &AppState,
    local_request: LocalRequest,
) -> Result<LocalResponse, String> {
    // Process the request using the integrated backend router
    let response = local_request.process_with_backend(&state.backend_router).await;
    Ok(response)
}

/// HTTP request handler for frontend bridge calls
#[tauri::command]
pub async fn handle_http_request(
    app_state: tauri::State<'_, AppState>,
    localRequest: LocalRequest,  // Parameter name matches frontend
) -> Result<LocalResponse, String> {
    process_local_request(&app_state.inner(), localRequest).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_request_creation() {
        let request = LocalRequest {
            uri: "/health".to_string(),
            method: "GET".to_string(),
            body: None,
            headers: HashMap::new(),
        };

        assert_eq!(request.uri, "/health");
        assert_eq!(request.method, "GET");
        assert!(request.body.is_none());
    }

    #[tokio::test]
    async fn test_local_response_creation() {
        let response = LocalResponse::text("test body".to_string());
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"test body");
    }

    #[tokio::test]
    async fn test_json_response() {
        let data = serde_json::json!({"status": "ok"});
        let response = LocalResponse::json(data).unwrap();

        assert_eq!(response.status_code, 200);
        assert!(response.headers.contains_key("content-type"));
        assert_eq!(response.headers["content-type"], "application/json");

        let parsed: serde_json::Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(parsed["status"], "ok");
    }

    #[tokio::test]
    async fn test_html_response() {
        let html = "<h1>Hello World</h1>".to_string();
        let response = LocalResponse::html(html.clone());

        assert_eq!(response.status_code, 200);
        assert!(response.headers.contains_key("content-type"));
        assert_eq!(response.headers["content-type"], "text/html; charset=utf-8");
        assert_eq!(response.body, html.as_bytes());
    }

    #[tokio::test]
    async fn test_internal_server_error() {
        let error = "Something went wrong".to_string();
        let response = LocalResponse::internal_server_error(error.clone());

        assert_eq!(response.status_code, 500);
        assert_eq!(response.body, format!("Internal Server Error: {}", error).as_bytes());
    }

    #[tokio::test]
    async fn test_local_request_processing() {
        let request = LocalRequest {
            uri: "/api/health".to_string(),
            method: "GET".to_string(),
            body: None,
            headers: HashMap::new(),
        };

        let response = request.process_with_backend().await;
        assert_eq!(response.status_code, 200);

        let body_str = String::from_utf8(response.body).unwrap();
        assert!(body_str.contains("GET"));
        assert!(body_str.contains("/api/health"));
    }
}