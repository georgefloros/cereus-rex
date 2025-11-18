// src/mcp/protocol.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Request structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpRequest {
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<String>,
}

/// MCP Response structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
    pub id: Option<String>,
}

/// MCP Error structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

/// MCP Capability structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpCapability {
    pub name: String,
    pub version: String,
}

/// MCP Configuration structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpConfig {
    pub capabilities: Vec<McpCapability>,
    pub settings: HashMap<String, serde_json::Value>,
}

impl McpRequest {
    pub fn new(method: String, params: Option<serde_json::Value>, id: Option<String>) -> Self {
        Self { method, params, id }
    }
}

impl McpResponse {
    pub fn success(result: serde_json::Value, id: Option<String>) -> Self {
        Self {
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(code: i32, message: String, id: Option<String>) -> Self {
        Self {
            result: None,
            error: Some(McpError { code, message }),
            id,
        }
    }
}