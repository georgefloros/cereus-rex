# Cerebus-Rex MCP Server Configuration Examples

This document provides detailed configuration examples for integrating Cerebus-Rex with various development tools that support the Model Context Protocol (MCP).

## Server Configuration

First, ensure your Cerebus-Rex server is properly configured. Create a `config/local.toml` file:

```toml
[server]
host = "0.0.0.0"
port = 3000
cors_origins = ["http://localhost:3000", "http://localhost:3001", "https://claude.ai", "https://copilot.github.com"]
max_connections = 100

[qdrant]
endpoint = "http://localhost:6333"
api_key = "your-api-key-here"  # Optional, remove if not using authentication
timeout_seconds = 30

[agents]
max_agents = 10
session_timeout_seconds = 3600  # 1 hour
sync_interval_seconds = 30
```

## Integration Examples

### 1. Claude Desktop Configuration

Claude Desktop allows direct MCP server connections:

1. Open Claude Desktop
2. Go to Settings → MCP Connectors
3. Add your Cerebus-Rex server with these details:
   - Server Name: `Cerebus-Rex`
   - Server URL: `ws://localhost:3000/mcp/ws`

Alternatively, Claude supports configuration via environment variables or local configuration files. Create a configuration profile that Claude can use:

```
MCP_SERVERS='[{
  "name": "cerebus-rex",
  "url": "ws://localhost:3000/mcp/ws",
  "enabled": true
}]'
```

### 2. Qwen-Code Configuration

Qwen-Code can be configured with MCP servers in multiple ways:

**Method 1: Using the CLI**
```bash
# Add the server
qwen code mcp add --name cerebus-rex --url ws://localhost:3000/mcp/ws

# List configured servers
qwen code mcp list

# Enable the server
qwen code mcp enable cerebus-rex
```

**Method 2: Configuration file (qwen-code-config.json)**
```json
{
  "mcp": {
    "servers": [
      {
        "name": "cerebus-rex",
        "url": "ws://localhost:3000/mcp/ws",
        "enabled": true,
        "settings": {
          "timeout": 30000,
          "retry_attempts": 3
        }
      }
    ],
    "discovery": {
      "auto_refresh": true,
      "cache_ttl": 300
    }
  },
  "context": {
    "max_tokens": 4096,
    "history_size": 10
  }
}
```

**Method 3: Environment variables**
```bash
export QWEN_MCP_SERVERS='[{"name": "cerebus-rex", "url": "ws://localhost:3000/mcp/ws", "enabled": true}]'
```

### 3. Factory AI Droid Configuration

Factory Droid supports MCP server configuration through several methods:

**Method 1: Using the CLI**
```bash
# Add the MCP server
droid mcp add --name cerebus-rex --url ws://localhost:3000/mcp/ws

# Verify the server is connected
droid mcp list

# Enable the server
droid mcp enable cerebus-rex

# Test the connection
droid mcp test cerebus-rex
```

**Method 2: Configuration file (.droid/config.yaml)**
```yaml
mcp:
  servers:
    - name: cerebus-rex
      url: ws://localhost:3000/mcp/ws
      enabled: true
      settings:
        timeout: 30000
        retry_attempts: 3
        connection_pool_size: 5

  discovery:
    auto_refresh: true
    cache_ttl: 300  # 5 minutes

droid:
  agent:
    enabled: true
    context_window: 8192
    max_tool_calls_per_request: 10

tools:
  enabled:
    - search
    - filesystem_operation
    - repository_indexing
  disabled: []
```

**Method 3: Environment variables**
```bash
export DROID_MCP_SERVERS='[{"name": "cerebus-rex", "url": "ws://localhost:3000/mcp/ws", "enabled": true}]'
```

### 4. GitHub Copilot Configuration

GitHub Copilot supports MCP servers in various IDEs:

**For JetBrains IDEs (IntelliJ, PyCharm, WebStorm, etc.):**

1. Install the GitHub Copilot plugin
2. Go to Settings → GitHub → Model Context Protocol
3. Click "Add Server" and enter:
   - Server Name: `Cerebus-Rex`
   - Server URL: `ws://localhost:3000/mcp/ws`
4. Click "Test Connection" to verify
5. Enable the server and restart the IDE

**For VS Code:**

Add to your `settings.json`:
```json
{
  "github.copilot.mcpServers": [
    {
      "name": "cerebus-rex",
      "url": "ws://localhost:3000/mcp/ws",
      "enabled": true
    }
  ]
}
```

**For Xcode:**

GitHub Copilot for Xcode supports MCP through the GitHub Copilot extension settings. Add the server URL in the MCP configuration section.

**Method 2: Configuration file approach**

Create a configuration file for Copilot extensions:

`.github/copilot-mcp.json`:
```json
{
  "mcpServers": [
    {
      "name": "cerebus-rex",
      "url": "ws://localhost:3000/mcp/ws",
      "enabled": true,
      "metadata": {
        "vendor": "Cerebus-Rex",
        "version": "0.1.0",
        "description": "RAG-enhanced development tools for AI agents"
      }
    }
  ]
}
```

### 5. Generic MCP Client Configuration

For any MCP-compatible client, the connection details are:

- **WebSocket Endpoint**: `ws://localhost:3000/mcp/ws`
- **Supported Protocols**: 
  - `mcp.tool.list` - List available tools
  - `mcp.tool.call` - Execute tools
  - `mcp.ping` - Health check
  - `mcp.metadata.get` - Get server metadata

**Example client configuration:**
```json
{
  "servers": [
    {
      "name": "cerebus-rex",
      "url": "ws://localhost:3000/mcp/ws",
      "protocol": "mcp",
      "enabled": true,
      "capabilities": [
        "search",
        "filesystem",
        "context",
        "indexing"
      ],
      "settings": {
        "timeout": 30000,
        "retry_attempts": 3,
        "connection_timeout": 10000
      }
    }
  ]
}
```

## Security Considerations

When configuring MCP servers, consider the following:

1. **Network Security**: If deploying in production, use HTTPS/WSS endpoints
2. **Authentication**: Implement API key authentication if required
3. **Rate Limiting**: Configure appropriate rate limits to prevent abuse
4. **Access Control**: Limit which tools are available to different clients

Example with authentication:
```toml
[server]
host = "0.0.0.0"
port = 3000
cors_origins = ["https://yourdomain.com"]
max_connections = 100

[qdrant]
endpoint = "https://your-qdrant-instance.com"
api_key = "your-production-api-key"
timeout_seconds = 30

[agents]
max_agents = 50
session_timeout_seconds = 7200  # 2 hours
sync_interval_seconds = 60

[security]
api_key = "your-mcp-auth-token"  # Used for MCP authentication
rate_limit_requests = 100
rate_limit_window_seconds = 60
```

## Troubleshooting

1. **Connection Issues**: Verify the server is running and accessible at the configured URL
2. **Tool Discovery**: Check that the `/mcp/tools` endpoint returns the expected tools
3. **Authentication**: Ensure API keys match between client and server if configured
4. **Firewall**: Make sure the port (default 3000) is accessible to your development tools

To test your server manually:
```bash
# Test health endpoint
curl http://localhost:3000/v1/health

# Test tool listing
curl http://localhost:3000/mcp/tools
```