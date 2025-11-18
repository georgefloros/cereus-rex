// src/mcp/handlers.rs
use super::protocol::{McpRequest, McpResponse};
use super::types::*;
use std::collections::HashMap;
use async_trait::async_trait;
use crate::{rag::client::RagClient, agents::sync::AgentCoordinator};
use tokio::sync::RwLock;
use std::sync::Arc;

// Define a more general state type for handlers
pub struct HandlerState {
    pub rag_client: Arc<RwLock<RagClient>>,
    pub agent_coordinator: Arc<AgentCoordinator>,
}

#[async_trait]
pub trait RequestHandler {
    async fn handle(&self, state: &HandlerState, request: McpRequest) -> McpResponse;
}

pub struct SearchHandler;
pub struct IndexHandler;
pub struct ContextHandler;
pub struct ToolHandler;

#[async_trait]
impl RequestHandler for SearchHandler {
    async fn handle(&self, state: &HandlerState, request: McpRequest) -> McpResponse {
        // Extract query from params
        let search_req = if let Some(params) = request.params {
            match serde_json::from_value::<SearchRequest>(params) {
                Ok(req) => req,
                Err(_) => {
                    return McpResponse::error(
                        -1,
                        "Invalid search request parameters".to_string(),
                        request.id,
                    );
                }
            }
        } else {
            return McpResponse::error(-1, "Missing parameters".to_string(), request.id);
        };

        // Perform search using RAG client
        let rag_client = state.rag_client.read().await;
        // In a real implementation, we would call the RAG system here
        // For now, return mock results
        drop(rag_client);

        let results = vec![
            SearchResult {
                id: "1".to_string(),
                content: format!("Search result for query: {}", search_req.query),
                source: "mock.md".to_string(),
                score: 0.95,
                metadata: Some(HashMap::new()),
            },
            SearchResult {
                id: "2".to_string(),
                content: "Another relevant result".to_string(),
                source: "project-structure.md".to_string(),
                score: 0.85,
                metadata: Some(HashMap::new()),
            },
        ];

        let result_json = serde_json::to_value(results).unwrap_or_default();
        McpResponse::success(result_json, request.id)
    }
}

#[async_trait]
impl RequestHandler for IndexHandler {
    async fn handle(&self, state: &HandlerState, request: McpRequest) -> McpResponse {
        // Extract index request from params
        let index_req = if let Some(params) = request.params {
            match serde_json::from_value::<IndexRequest>(params) {
                Ok(req) => req,
                Err(_) => {
                    return McpResponse::error(
                        -1,
                        "Invalid index request parameters".to_string(),
                        request.id,
                    );
                }
            }
        } else {
            return McpResponse::error(-1, "Missing parameters".to_string(), request.id);
        };

        // In a real implementation, we would update the repository index via the RAG client
        // For now, return a success message
        McpResponse::success(serde_json::json!({"status": "indexed", "repo_id": index_req.repo_id}), request.id)
    }
}

#[async_trait]
impl RequestHandler for ContextHandler {
    async fn handle(&self, state: &HandlerState, request: McpRequest) -> McpResponse {
        // Extract context key from params
        let params = if let Some(params) = request.params {
            params
        } else {
            return McpResponse::error(-1, "Missing parameters".to_string(), request.id);
        };

        let key = if let Some(key_val) = params.get("key").and_then(|v| v.as_str()) {
            key_val.to_string()
        } else {
            return McpResponse::error(
                -1,
                "Missing key parameter".to_string(),
                request.id,
            );
        };

        // In a real implementation, we would retrieve context from the store
        // For now, return mock result
        let result = serde_json::json!({
            "key": key,
            "value": format!("Context value for {}", key)
        });

        McpResponse::success(result, request.id)
    }
}

#[async_trait]
impl RequestHandler for ToolHandler {
    async fn handle(&self, state: &HandlerState, request: McpRequest) -> McpResponse {
        // Return list of available tools
        let tools = vec![
            Tool {
                id: "search".to_string(),
                name: "Documentation Search".to_string(),
                description: "Search documentation and codebase".to_string(),
                input_schema: None,
            },
            Tool {
                id: "index_repo".to_string(),
                name: "Repository Indexer".to_string(),
                description: "Index repository files for quick access".to_string(),
                input_schema: None,
            },
            Tool {
                id: "get_context".to_string(),
                name: "Context Manager".to_string(),
                description: "Access and manage context information".to_string(),
                input_schema: None,
            },
        ];

        let result = serde_json::to_value(tools).unwrap_or_default();
        McpResponse::success(result, request.id)
    }
}