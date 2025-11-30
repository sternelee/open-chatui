//! # Tauri HTTP Bridge
//!
//! A simplified bridge for handling HTTP requests in the Tauri application.
//! This module provides request/response structures for communication between
//! the frontend and the integrated backend.

use std::collections::HashMap;
use std::fmt::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during request/response bridging
#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Could not parse method from LocalRequest: {0}")]
    RequestMethodParseError(String),

    #[error("Could not parse URL: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Represents an HTTP request that can be processed by the backend
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalRequest {
    pub uri: String,
    pub method: String,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

/// Represents an HTTP response returned from the backend
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalResponse {
    pub status_code: u16,
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
}

impl LocalResponse {
    /// Create an internal server error response
    pub fn internal_server_error(error: impl Display) -> Self {
        let error_message = format!("An error occurred: {}", error);
        Self {
            status_code: 500,
            body: error_message.into(),
            headers: Default::default(),
        }
    }

    /// Create a successful response with the given body
    pub fn ok<T: Into<Vec<u8>>>(body: T) -> Self {
        Self {
            status_code: 200,
            body: body.into(),
            headers: Default::default(),
        }
    }

    /// Create a JSON response
    pub fn json<T: Serialize>(data: T) -> Result<Self, BridgeError> {
        let body = serde_json::to_vec(&data)?;
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Ok(Self {
            status_code: 200,
            body,
            headers,
        })
    }
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
        let response = LocalResponse::ok("test body");
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
}