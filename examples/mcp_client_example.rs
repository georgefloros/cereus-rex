// examples/mcp_client_example.rs
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use futures_util::{SinkExt, StreamExt};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    #[serde(rename = "tools/list")]
    ListTools {},
    #[serde(rename = "tools/call")]
    CallTool {
        name: String,
        arguments: Option<std::collections::HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "server/info")]
    GetServerInfo {},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Cerebus-Rex MCP server...");
    
    // Connect to the MCP server WebSocket endpoint
    let url = Url::parse("ws://localhost:3000/mcp/ws")?;
    
    match connect_async(url).await {
        Ok((ws_stream, _)) => {
            println!("Connected to Cerebus-Rex MCP server");
            
            let (mut write, mut read) = ws_stream.split();

            // Request server info
            let info_request = McpRequest::GetServerInfo {};
            let request_json = serde_json::to_string(&serde_json::json!({
                "method": "server/info",
                "id": "1"
            }))?;
            write.send(Message::Text(request_json)).await?;
            println!("Sent server info request");

            // Read response
            if let Some(msg) = read.next().await {
                match msg? {
                    Message::Text(text) => {
                        println!("Server info response: {}", text);
                    },
                    _ => println!("Unexpected message type for server info"),
                }
            }

            // Request available tools
            let tools_request_json = serde_json::to_string(&serde_json::json!({
                "method": "tools/list",
                "id": "2"
            }))?;
            write.send(Message::Text(tools_request_json)).await?;
            println!("Sent tools list request");

            // Read response
            if let Some(msg) = read.next().await {
                match msg? {
                    Message::Text(text) => {
                        println!("Tools list response: {}", text);
                    },
                    _ => println!("Unexpected message type for tools list"),
                }
            }

            // Call a tool (example search)
            let search_request_json = serde_json::to_string(&serde_json::json!({
                "method": "tools/call",
                "params": {
                    "name": "search",
                    "arguments": {
                        "query": "example search query",
                        "context": "documentation"
                    }
                },
                "id": "3"
            }))?;
            write.send(Message::Text(search_request_json)).await?;
            println!("Sent search tool request");

            // Read response
            if let Some(msg) = read.next().await {
                match msg? {
                    Message::Text(text) => {
                        println!("Search tool response: {}", text);
                    },
                    _ => println!("Unexpected message type for search tool"),
                }
            }

            // Close connection
            write.close().await?;
            println!("Disconnected from Cerebus-Rex MCP server");
        },
        Err(e) => {
            eprintln!("Failed to connect to Cerebus-Rex MCP server: {}", e);
            println!("Make sure the server is running on ws://localhost:3000/mcp/ws");
        }
    }

    Ok(())
}