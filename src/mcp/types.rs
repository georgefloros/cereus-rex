// src/mcp/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool definition for MCP
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_schema: Option<serde_json::Value>,
}

/// Search request structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub context: Option<String>,
    pub limit: Option<u32>,
}

/// Search result structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub source: String,
    pub score: f32,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Repository indexing request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexRequest {
    pub repo_id: String,
    pub files: Vec<IndexedFile>,
}

/// Represents an indexed file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexedFile {
    pub path: String,
    pub content: String,
    pub language: Option<String>,
    pub hash: String,
}

/// Context information for agents
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentContext {
    pub id: String,
    pub name: String,
    pub data: HashMap<String, serde_json::Value>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Agent session information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentSession {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
    pub last_seen: u64,
    pub context: AgentContext,
}