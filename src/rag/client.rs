// src/rag/client.rs
use crate::mcp::types::{SearchRequest, SearchResult};
use qdrant_client::{
    Qdrant,
    qdrant::{CreateCollection, VectorParams, HnswConfigDiff},
};
use serde_json::Value;
use std::collections::HashMap;

/// RAG (Retrieval-Augmented Generation) client for connecting to vector databases
pub struct RagClient {
    pub qdrant_client: Option<Qdrant>,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub collections: Vec<String>,
}

impl RagClient {
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        Self {
            qdrant_client: None, // Will be initialized later
            endpoint,
            api_key,
            collections: vec![
                "documentation".to_string(),
                "code".to_string(),
                "config".to_string(),
            ],
        }
    }

    /// Initialize the Qdrant client connection
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = Qdrant::from_url(&self.endpoint);
        if let Some(api_key) = &self.api_key {
            client = client.api_key(api_key.clone());
        }
        let client = client.build()?;

        // Set the client
        self.qdrant_client = Some(client);

        // Create collections if they don't exist
        self.create_collections().await?;

        Ok(())
    }

    /// Create necessary collections in Qdrant
    async fn create_collections(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.qdrant_client {
            for collection_name in &self.collections {
                // Check if collection exists
                match client.collection_exists(collection_name).await {
                    Ok(exists) => {
                        if !exists {
                            // Create collection
                            let collection_config = CreateCollection {
                                collection_name: collection_name.clone(),
                                vectors_config: Some(
                                    VectorParams {
                                        size: 384, // Use 384 for sentence transformers
                                        distance: 0, // Cosine distance (0 = Cosine)
                                        ..Default::default()
                                    }.into()
                                ),
                                hnsw_config: Some(HnswConfigDiff {
                                    m: Some(16),
                                    ef_construct: Some(100),
                                    ..Default::default()
                                }),
                                quantization_config: None, // Simplified for now
                                ..Default::default()
                            };

                            client.create_collection(collection_config).await?;
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to check collection existence: {}", e).into());
                    }
                }
            }
        }

        Ok(())
    }

    /// Perform a semantic search in the RAG system
    pub async fn search(&self, request: &SearchRequest) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        if let Some(client) = &self.qdrant_client {
            let collection_name = request.context.as_ref().unwrap_or(&"documentation".to_string());
            
            // In a real implementation, we would convert the query to an embedding
            // and perform the semantic search in Qdrant
            // For now, return mock results
            let results = vec![
                SearchResult {
                    id: "1".to_string(),
                    content: format!("Search result for query: {}", request.query),
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

            Ok(results)
        } else {
            Err("Qdrant client not initialized".into())
        }
    }

    /// Index a document in the RAG system
    pub async fn index_document(
        &self,
        collection_name: &str,
        id: &str,
        content: &str,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.qdrant_client {
            // In a real implementation, we would:
            // 1. Embed the content using a model like sentence transformers
            // 2. Store the embedding in Qdrant with the content as payload
            // For now, we just return Ok to simulate the operation
            
            println!("Would index document {} in collection {}", id, collection_name);
            Ok(())
        } else {
            Err("Qdrant client not initialized".into())
        }
    }

    /// Index repository files for quick access
    pub async fn index_repo_files(
        &self,
        repo_id: &str,
        files: Vec<(String, String)>, // (file_path, content)
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (file_path, content) in files {
            let doc_id = format!("{}-{}", repo_id, file_path.replace("/", "_"));
            let mut metadata = HashMap::new();
            metadata.insert("repo_id".to_string(), serde_json::Value::String(repo_id.to_string()));
            metadata.insert("file_path".to_string(), serde_json::Value::String(file_path));
            
            self.index_document("code", &doc_id, &content, Some(metadata)).await?;
        }
        
        Ok(())
    }

    /// Get a document by ID
    pub async fn get_document(&self, collection_name: &str, id: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some(client) = &self.qdrant_client {
            // In a real implementation, we would retrieve the document from Qdrant
            // For now, return a mock result
            Ok(Some(format!("Content of document {}", id)))
        } else {
            Err("Qdrant client not initialized".into())
        }
    }
}

impl Default for RagClient {
    fn default() -> Self {
        Self::new("http://localhost:6333".to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rag_client_creation() {
        let rag_client = RagClient::default();
        assert_eq!(rag_client.endpoint, "http://localhost:6333");
        assert_eq!(rag_client.collections.len(), 3);
    }

    #[tokio::test]
    #[ignore] // Ignore until Qdrant is available
    async fn test_rag_client_initialization() {
        let mut rag_client = RagClient::default();
        let result = rag_client.initialize().await;
        // This will fail without a running Qdrant instance
        // The test is just to make sure the code compiles
    }
}