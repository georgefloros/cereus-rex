// examples/simple_client.rs
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use futures_util::{SinkExt, StreamExt};

#[derive(Serialize, Deserialize, Debug)]
struct McpRequest {
    method: String,
    params: Option<serde_json::Value>,
    id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct McpResponse {
    result: Option<serde_json::Value>,
    error: Option<McpError>,
    id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct McpError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse("ws://localhost:3000/mcp/ws")?;
    
    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to Cerebus-Rex MCP server");

    let (mut write, mut read) = ws_stream.split();
    
    // Request available tools
    let tools_request = McpRequest {
        method: "list_tools".to_string(),
        params: None,
        id: Some("1".to_string()),
    };

    let request_json = serde_json::to_string(&tools_request)?;
    write.send(Message::Text(request_json)).await?;
    println!("Sent tools request");

    if let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                let response: McpResponse = serde_json::from_str(&text)?;
                println!("Received tools response: {:?}", response.result);
            },
            _ => println!("Unexpected message type"),
        }
    }

    // Perform a search
    let search_request = McpRequest {
        method: "search".to_string(),
        params: Some(serde_json::json!({
            "query": "example search",
            "context": "documentation"
        })),
        id: Some("2".to_string()),
    };

    let request_json = serde_json::to_string(&search_request)?;
    write.send(Message::Text(request_json)).await?;
    println!("Sent search request");

    if let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                let response: McpResponse = serde_json::from_str(&text)?;
                println!("Received search response: {:?}", response.result);
            },
            _ => println!("Unexpected message type"),
        }
    }

    // Close the connection
    write.close().await?;
    println!("Disconnected from Cerebus-Rex MCP server");

    Ok(())
}