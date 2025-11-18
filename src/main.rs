mod mcp;
mod agents;
mod rag;
mod tools;
mod utils;
mod mcp_server;

use tracing_subscriber::{self};

use crate::{
    rag::client::RagClient,
    tools::{filesystem::FilesystemTool, search::SearchTool, code_context::ThreadsafeCodeContextTool},
    utils::config::{Settings, init_tracing},
    mcp_server::CerebusRexMcpServer,
};

// Main application entry point - MCP server only
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init_tracing();

    // Load settings
    let settings = Settings::new().unwrap_or_else(|_| Settings::default());

    // Create the RAG client
    let mut rag_client = RagClient::new(settings.qdrant.endpoint.clone(), settings.qdrant.api_key.clone());

    // Initialize the RAG client (this might fail if Qdrant is not available)
    if let Err(e) = rag_client.initialize().await {
        tracing::warn!("Failed to initialize RAG client: {}", e);
    }

    // Create tools
    let search_tool = SearchTool::new(
        RagClient::new(settings.qdrant.endpoint.clone(), settings.qdrant.api_key.clone())
    );
    let filesystem_tool = FilesystemTool;
    let context_tool = ThreadsafeCodeContextTool::new();

    // Create the MCP server
    let _server = CerebusRexMcpServer::new(
        rag_client,
        search_tool,
        filesystem_tool,
        context_tool,
    );

    // In a real implementation, you would set up a WebSocket server to handle connections
    tracing::info!("Cerebus-Rex MCP server initialized");

    // For now, just keep the server running
    tokio::signal::ctrl_c().await?;
    tracing::info!("Cerebus-Rex MCP server shutting down");

    Ok(())
}