// src/mcp_server.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::AsyncBufReadExt;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::rag::client::RagClient;
use crate::tools::search::SearchTool;
use crate::tools::filesystem::FilesystemTool;
use crate::tools::code_context::ThreadsafeCodeContextTool;

/// MCP Server for Cerebus-Rex that follows the Model Context Protocol specification
pub struct CerebusRexMcpServer {
    rag_client: Arc<RwLock<RagClient>>,
    search_tool: Arc<SearchTool>,
    filesystem_tool: Arc<FilesystemTool>,
    context_tool: Arc<ThreadsafeCodeContextTool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    #[serde(rename = "tools/list")]
    ListTools {},
    #[serde(rename = "tools/call")]
    CallTool {
        name: String,
        arguments: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "resources/list")]
    ListResources {},
    #[serde(rename = "resources/read")]
    ReadResource {
        uri: String,
    },
    #[serde(rename = "prompts/list")]
    ListPrompts {},
    #[serde(rename = "prompts/get")]
    GetPrompt {
        name: String,
        arguments: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "server/info")]
    GetServerInfo {},
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum McpResponse {
    Success {
        result: serde_json::Value,
        id: Option<String>,
    },
    Error {
        error: McpError,
        id: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

impl CerebusRexMcpServer {
    pub fn new(
        rag_client: RagClient,
        search_tool: SearchTool,
        filesystem_tool: FilesystemTool,
        context_tool: ThreadsafeCodeContextTool,
    ) -> Self {
        Self {
            rag_client: Arc::new(RwLock::new(rag_client)),
            search_tool: Arc::new(search_tool),
            filesystem_tool: Arc::new(filesystem_tool),
            context_tool: Arc::new(context_tool),
        }
    }

    pub async fn serve_websocket(
        self: Arc<Self>,
        websocket: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mut sender, mut receiver) = websocket.split();
        
        while let Some(msg) = receiver.next().await {
            let msg = msg?;
            if msg.is_text() || msg.is_binary() {
                let text = msg.to_text().unwrap().to_string();
                
                // Parse the incoming request
                let response = self.handle_request(&text).await;
                
                // Send the response
                let response_text = serde_json::to_string(&response)?;
                sender.send(Message::Text(response_text.into())).await?;
            }
        }
        
        Ok(())
    }

    async fn handle_request(&self, request_str: &str) -> McpResponse {
        match serde_json::from_str::<McpRequest>(request_str) {
            Ok(request) => {
                let id = extract_id(request_str); // Extract ID from original JSON
                
                match request {
                    McpRequest::ListTools {} => {
                        let tools = self.list_tools().await;
                        McpResponse::Success {
                            result: serde_json::json!(tools),
                            id,
                        }
                    },
                    McpRequest::CallTool { name, arguments } => {
                        let result = self.call_tool(&name, arguments.unwrap_or_default()).await;
                        match result {
                            Ok(content) => McpResponse::Success {
                                result: serde_json::json!({ "content": content }),
                                id,
                            },
                            Err(e) => McpResponse::Error {
                                error: McpError { code: -32000, message: e },
                                id,
                            },
                        }
                    },
                    McpRequest::ListResources {} => {
                        let resources = self.list_resources().await;
                        McpResponse::Success {
                            result: serde_json::json!(resources),
                            id,
                        }
                    },
                    McpRequest::ReadResource { uri } => {
                        let content = self.read_resource(&uri).await;
                        match content {
                            Ok(content) => McpResponse::Success {
                                result: serde_json::json!({ "content": content }),
                                id,
                            },
                            Err(e) => McpResponse::Error {
                                error: McpError { code: -32000, message: e },
                                id,
                            },
                        }
                    },
                    McpRequest::ListPrompts {} => {
                        let prompts = self.list_prompts().await;
                        McpResponse::Success {
                            result: serde_json::json!(prompts),
                            id,
                        }
                    },
                    McpRequest::GetPrompt { name, arguments } => {
                        let prompt = self.get_prompt(&name, arguments.unwrap_or_default()).await;
                        match prompt {
                            Ok(prompt) => McpResponse::Success {
                                result: serde_json::json!(prompt),
                                id,
                            },
                            Err(e) => McpResponse::Error {
                                error: McpError { code: -32000, message: e },
                                id,
                            },
                        }
                    },
                    McpRequest::GetServerInfo {} => {
                        let info = self.get_server_info().await;
                        McpResponse::Success {
                            result: serde_json::json!(info),
                            id,
                        }
                    },
                }
            }
            Err(e) => {
                McpResponse::Error {
                    error: McpError { 
                        code: -32700, // Parse error
                        message: format!("Invalid JSON: {}", e) 
                    },
                    id: extract_id(request_str),
                }
            }
        }
    }

    async fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "search".to_string(),
                description: Some("Search documentation and code".to_string()),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "context": {
                            "type": "string",
                            "description": "Search context (documentation, code, etc.)",
                            "enum": ["documentation", "code", "config"]
                        }
                    },
                    "required": ["query"]
                })),
            },
            Tool {
                name: "filesystem_read".to_string(),
                description: Some("Read a file from the filesystem".to_string()),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path to read"
                        }
                    },
                    "required": ["path"]
                })),
            },
            Tool {
                name: "index_repo".to_string(),
                description: Some("Index repository files for quick access".to_string()),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repo_id": {
                            "type": "string",
                            "description": "Repository identifier"
                        },
                        "files": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "List of file paths to index"
                        }
                    },
                    "required": ["repo_id", "files"]
                })),
            },
            Tool {
                name: "initialize_context".to_string(),
                description: Some("Initialize AI agent context by indexing repository and documentation".to_string()),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repo_path": {
                            "type": "string",
                            "description": "Path to repository to index"
                        },
                        "include_docs": {
                            "type": "boolean",
                            "description": "Whether to index documentation files",
                            "default": true
                        },
                        "include_code": {
                            "type": "boolean",
                            "description": "Whether to index code files",
                            "default": true
                        }
                    },
                    "required": ["repo_path"]
                })),
            },
        ]
    }

    async fn call_tool(&self, name: &str, arguments: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        match name {
            "search" => {
                let query = arguments
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing query parameter".to_string())?
                    .to_string();

                let context = arguments
                    .get("context")
                    .and_then(|v| v.as_str())
                    .unwrap_or("documentation") // default context
                    .to_string();

                // Create a search request for our search tool
                let search_request = crate::tools::search::SearchRequest {
                    query,
                    context: Some(context),
                    limit: Some(5), // reasonable default
                };

                match self.search_tool.execute(search_request).await {
                    Ok(results) => {
                        let mut output = String::new();
                        for result in results.results {
                            output.push_str(&format!("- Source: {}\n", result.source));
                            output.push_str(&format!("  Content: {}\n", result.content));
                            output.push_str(&format!("  Score: {}\n\n", result.score));
                        }
                        Ok(serde_json::json!(output))
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            "filesystem_read" => {
                let path = arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing path parameter".to_string())?
                    .to_string();

                let result = self.read_filesystem(&path).await?;
                Ok(serde_json::json!(result))
            }
            "index_repo" => {
                let repo_id = arguments
                    .get("repo_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing repo_id parameter".to_string())?
                    .to_string();

                let files_array = arguments
                    .get("files")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| "Missing files parameter as array".to_string())?;

                let files: Vec<String> = files_array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                // Convert files to the format expected by the RAG client
                let files_with_content: Vec<(String, String)> = files.iter()
                    .map(|file_path| (file_path.clone(), "content placeholder".to_string()))
                    .collect();

                match self.search_tool.index_repo_files(&repo_id, files_with_content).await {
                    Ok(_) => Ok(serde_json::json!({"status": "indexed", "repo_id": repo_id})),
                    Err(e) => Err(e.to_string()),
                }
            }
            "initialize_context" => {
                let repo_path = arguments
                    .get("repo_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing repo_path parameter".to_string())?
                    .to_string();

                let include_docs = arguments
                    .get("include_docs")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let include_code = arguments
                    .get("include_code")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                // Perform repository indexing
                let mut files_to_index = Vec::new();

                if include_docs {
                    // Add common documentation file patterns
                    files_to_index.extend(vec![
                        format!("{}/README.md", repo_path),
                        format!("{}/CHANGELOG.md", repo_path),
                        format!("{}/CONTRIBUTING.md", repo_path),
                        format!("{}/LICENSE", repo_path),
                    ]);
                }

                if include_code {
                    // Add common code file patterns (this is a simplified example)
                    // In a real implementation, this would scan the directory for code files
                    files_to_index.extend(vec![
                        format!("{}/src/main.rs", repo_path),
                        format!("{}/src/lib.rs", repo_path),
                    ]);
                }

                // Process the gathered files
                let repo_id = format!("repo_{}", uuid::Uuid::new_v4());
                let files_with_content: Vec<(String, String)> = files_to_index
                    .iter()
                    .filter(|file_path| std::path::Path::new(file_path).exists())
                    .map(|file_path| {
                        let content = match std::fs::read_to_string(file_path) {
                            Ok(content) => content,
                            Err(_) => "Could not read file content".to_string(),
                        };
                        (file_path.clone(), content)
                    })
                    .collect();

                let indexed_files_count = files_with_content.len();

                // Index the repository files
                match self.search_tool.index_repo_files(&repo_id, files_with_content).await {
                    Ok(_) => {
                        // Also initialize other context elements here
                        Ok(serde_json::json!({
                            "status": "context_initialized",
                            "repo_id": repo_id,
                            "indexed_files_count": indexed_files_count,
                            "included_docs": include_docs,
                            "included_code": include_code,
                            "message": "Context initialized successfully. Repository indexed for semantic search."
                        }))
                    },
                    Err(e) => Err(e.to_string()),
                }
            }
            _ => Err(format!("Unknown tool: {}", name)),
        }
    }

    async fn list_resources(&self) -> Vec<ResourceTemplate> {
        vec![
            ResourceTemplate {
                uri_template: "cerebus-rex://docs".to_string(),
                name: "Documentation".to_string(),
                description: Some("Documentation resources from RAG system".to_string()),
            },
            ResourceTemplate {
                uri_template: "cerebus-rex://code".to_string(),
                name: "Code".to_string(),
                description: Some("Code resources from repository".to_string()),
            },
        ]
    }

    async fn read_resource(&self, uri: &str) -> Result<String, String> {
        match uri {
            uri if uri.starts_with("cerebus-rex://docs") => {
                Ok("Documentation content from RAG system".to_string())
            },
            uri if uri.starts_with("cerebus-rex://code") => {
                Ok("Code content from repository".to_string())
            },
            _ => Err(format!("Resource not found: {}", uri)),
        }
    }

    async fn list_prompts(&self) -> Vec<PromptTemplate> {
        vec![
            PromptTemplate {
                name: "code_search".to_string(),
                description: Some("Search for code snippets".to_string()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        }
                    },
                    "required": ["query"]
                })),
            }
        ]
    }

    async fn get_prompt(&self, name: &str, _arguments: HashMap<String, serde_json::Value>) -> Result<Prompt, String> {
        match name {
            "code_search" => {
                Ok(Prompt {
                    messages: vec![MessageContent {
                        role: "user".to_string(),
                        content: "Search Results:\nCode search results here".to_string(),
                    }],
                })
            },
            _ => Err(format!("Prompt not found: {}", name)),
        }
    }

    async fn get_server_info(&self) -> ServerInfo {
        ServerInfo {
            name: "Cerebus-Rex MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: Capabilities {
                list_resources: true,
                read_resource: true,
                list_prompts: true,
                get_prompt: true,
                list_tools: true,
                call_tool: true,
            },
        }
    }

    async fn read_filesystem(&self, path: &str) -> Result<String, String> {
        use crate::tools::filesystem::{FileOperationRequest, FileOperationResponse};
        
        let fs_request = FileOperationRequest {
            operation: "read".to_string(),
            path: path.to_string(),
            content: None,
            recursive: None,
        };

        match self.filesystem_tool.execute(fs_request).await {
            Ok(FileOperationResponse { success: true, data: Some(content), .. }) => {
                if let Some(content_value) = content.get("content") {
                    if let Some(content_str) = content_value.as_str() {
                        Ok(content_str.to_string())
                    } else {
                        Err("Invalid content format".to_string())
                    }
                } else {
                    Err("Missing content in response".to_string())
                }
            }
            Ok(FileOperationResponse { success: false, message, .. }) => {
                Err(message)
            }
            Ok(_) => {
                Err("Unknown error in filesystem operation".to_string())
            }
            Err(e) => {
                Err(e.to_string())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceTemplate {
    pub uri_template: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromptTemplate {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prompt {
    pub messages: Vec<MessageContent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageContent {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Capabilities,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Capabilities {
    pub list_resources: bool,
    pub read_resource: bool,
    pub list_prompts: bool,
    pub get_prompt: bool,
    pub list_tools: bool,
    pub call_tool: bool,
}

// Helper function to extract request ID from JSON string
fn extract_id(request_str: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(request_str) {
        if let Some(id_val) = value.get("id") {
            if let Some(id_str) = id_val.as_str() {
                return Some(id_str.to_string());
            } else if let Some(id_num) = id_val.as_i64() {
                return Some(id_num.to_string());
            }
        }
    }
    None
}