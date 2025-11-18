// src/tools/search.rs
use crate::rag::RagClient;
use crate::mcp::types::{SearchRequest as McpSearchRequest, SearchResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchRequest {
    pub query: String,
    pub context: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query: String,
    pub took_ms: u64,
}

pub struct SearchTool {
    pub rag_client: RagClient,
}

impl SearchTool {
    pub fn new(rag_client: RagClient) -> Self {
        Self { rag_client }
    }

    pub async fn execute(&self, request: SearchRequest) -> Result<SearchResponse, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        let mcp_request = McpSearchRequest {
            query: request.query.clone(),
            context: request.context,
            limit: request.limit,
        };

        let results = self.rag_client.search(&mcp_request).await?;

        let took_ms = start_time.elapsed().as_millis() as u64;

        Ok(SearchResponse {
            results,
            query: request.query,
            took_ms,
        })
    }

    pub async fn index_repo_files(
        &self,
        repo_id: &str,
        files: Vec<(String, String)>, // (file_path, content)
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.rag_client.index_repo_files(repo_id, files).await?;
        Ok(())
    }
}