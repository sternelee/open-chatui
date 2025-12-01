//! Backend Bridge Module for Tauri Integration
//!
//! This module implements the tauri-actix-web pattern for processing HTTP requests
//! within the Tauri application using an Actix-web server.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tauri::State;
use actix_web::{
    http::{Method, Uri},
    HttpRequest, HttpResponse,
    http::header::{HeaderMap, HeaderValue},
};

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

/// Application state containing the backend configuration
#[derive(Clone)]
pub struct AppState {
    // This will hold our integrated backend state
    pub backend_available: bool,
}

impl LocalRequest {
    /// Process the request using the integrated backend
    pub async fn process_with_backend(self) -> LocalResponse {
        // This will be implemented to use our integrated backend
        // For now, return a simple response based on the path
        let (status_code, body) = match self.uri.as_str() {
            "/health" | "/api/health" => (
                200,
                serde_json::json!({
                    "status": true,
                    "message": "Backend is running"
                }).to_string()
            ),
            "/api/config" => (
                200,
                serde_json::json!({
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
                }).to_string()
            ),
            "/api/models" => (
                200,
                serde_json::json!({
                    "data": []
                }).to_string()
            ),
            _ => (
                404,
                serde_json::json!({
                    "error": "Not Found"
                }).to_string()
            ),
        };

        LocalResponse {
            status_code,
            body: body.into_bytes(),
            headers: HashMap::new(),
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
    _state: &AppState,
    local_request: LocalRequest,
) -> Result<LocalResponse, String> {
    // Process the request using the integrated backend
    let response = local_request.process_with_backend().await;
    Ok(response)
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