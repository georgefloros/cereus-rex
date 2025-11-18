# Cerebus-Rex MCP Server

Cerebus-Rex is a Model Context Protocol (MCP) server written in Rust that enables AI coding agents to access documentation, code context, and other development tools through a standardized protocol. The server integrates with a RAG (Retrieval-Augmented Generation) system for semantic search capabilities and provides synchronization between multiple AI agents.

## Features

- **MCP Protocol Support**: Implements the Model Context Protocol for standardized AI tool integration
- **RAG Integration**: Connects to Qdrant vector database for semantic search of documentation and code
- **Multi-Agent Synchronization**: Coordinates state and context between multiple AI coding agents
- **Repository Indexing**: Maintains local repository indexes for quick searches, code updates, and documentation access
- **Development Tools**: Provides filesystem operations, search capabilities, and context management as tools for AI agents
- **Scalable Architecture**: Built with Rust and Axum for performance and reliability

## Architecture

The server is organized into several core modules:

- **MCP**: Handles the Model Context Protocol communication
- **Agents**: Manages multi-agent synchronization and coordination
- **RAG**: Integrates with vector databases for semantic search
- **Tools**: Provides various development tools to AI agents
- **Utils**: Configuration and utility functions

## Prerequisites

- Rust (1.70 or higher)
- Cargo
- Qdrant vector database (optional, for full RAG functionality)

## Installation

```bash
# Clone the repository
git clone https://github.com/your-org/cerebus-rex.git
cd cerebus-rex

# Build the project
cargo build --release

# Run the server
cargo run
```

## Configuration

The server can be configured using:

1. Environment variables prefixed with `APP_`
2. Configuration files in the `config/` directory
3. Default values defined in the code

Key configuration options:
- `APP_SERVER_HOST`: Server host (default: 0.0.0.0)
- `APP_SERVER_PORT`: Server port (default: 3000)
- `APP_QDRANT_ENDPOINT`: Qdrant endpoint (default: http://localhost:6333)
- `APP_QDRANT_API_KEY`: Qdrant API key (optional)

## API Endpoints

- `GET /v1/health` - Health check
- `POST /v1/search` - Semantic search across documentation and code
- `POST /v1/index` - Index repository files for quick access
- `GET /v1/agents` - List active agents
- `GET /v1/agents/{id}` - Get specific agent information
- `GET /mcp/ws` - WebSocket endpoint for MCP communication
- `GET /mcp/tools` - List available tools
- `POST /mcp/tools/{id}` - Execute specific tool

## MCP Integration

AI agents can connect to Cerebus-Rex via the WebSocket endpoint at `/mcp/ws` and use the following MCP methods:

- `search` - Perform semantic searches
- `index_repo` - Index repository files
- `get_context` - Access and manage context information
- `fs_operation` - Perform filesystem operations
- `list_tools` - Get available tools

## Integration Examples

### 1. Claude (Anthropic)

Claude Desktop and Claude Code support connecting to MCP servers directly:

**Configuration via Claude Desktop Settings:**
1. Open Claude Desktop
2. Go to Settings → MCP Connectors
3. Add the Cerebus-Rex server endpoint: `ws://localhost:3000/mcp/ws`
4. Claude will automatically discover available tools from the server

**Using MCP tools in Claude:**
Claude will automatically use the tools provided by Cerebus-Rex when it determines they would be helpful for your request. For example, when you ask about code in your repository, Claude will:
- Use the search tool to find relevant documentation
- Use the filesystem tool to read necessary files
- Use the indexing tool to keep repository context up-to-date

### 2. Qwen-Code (Alibaba Cloud)

Qwen-Code natively supports the Model Context Protocol and can connect to MCP servers:

**Configuration:**
1. In Qwen Code CLI, configure MCP servers in your settings:
```json
{
  "mcp": {
    "servers": [
      {
        "name": "cerebus-rex",
        "url": "ws://localhost:3000/mcp/ws",
        "enabled": true
      }
    ]
  }
}
```

2. Or use the Qwen Code configuration file to add the server:
```bash
# Add the MCP server
qwen code mcp add --name cerebus-rex --url ws://localhost:3000/mcp/ws
```

**Usage:**
Qwen-Code will automatically detect tools provided by Cerebus-Rex and use them as needed. When working on code, Qwen-Code can:
- Search your documentation using the RAG system
- Index repository files for better context
- Access project files through the filesystem tools

### 3. Factory AI Droid

Factory AI Droid has native MCP support for extending its capabilities:

**Configuration:**
```bash
# Add the Cerebus-Rex MCP server
droid mcp add --name cerebus-rex --url ws://localhost:3000/mcp/ws

# Verify the server is connected
droid mcp list
```

**Configuration file approach:**
```yaml
mcp:
  servers:
    - name: cerebus-rex
      url: ws://localhost:3000/mcp/ws
      enabled: true
```

**Usage:**
Factory Droid will automatically detect and use tools from Cerebus-Rex during development tasks. The Droid agent can:
- Use RAG search to understand your codebase
- Access file system tools for reading/writing operations
- Maintain synchronized context across development sessions
- Index new repository files as they're added

### 4. GitHub Copilot

GitHub Copilot supports MCP servers in various IDEs (JetBrains, Eclipse, Xcode) and has native MCP support:

**For Copilot in JetBrains IDEs:**
1. Install the GitHub Copilot plugin
2. Go to Settings → GitHub → Model Context Protocol
3. Add the server URL: `ws://localhost:3000/mcp/ws`
4. Restart the IDE to connect to the MCP server

**Configuration file approach:**
In your `.vscode/settings.json` or IDE-specific config:
```json
{
  "github.copilot.mcpServers": [
    {
      "name": "cerebus-rex",
      "url": "ws://localhost:3000/mcp/ws"
    }
  ]
}
```

**Usage:**
GitHub Copilot will use tools provided by Cerebus-Rex to enhance code suggestions:
- Semantic search of your documentation and codebase
- Repository indexing for better context
- File operations for comprehensive code understanding

### 5. Visual Studio Code with MCP Extensions

VS Code can connect to MCP servers using MCP-compatible extensions:

**Installation:**
1. Install an MCP connector extension (if available)
2. Or configure through settings.json:
```json
{
  "mcp.servers": [
    {
      "name": "cerebus-rex",
      "url": "ws://localhost:3000/mcp/ws",
      "enabled": true
    }
  ]
}
```

**Usage:**
VS Code will allow MCP-enabled extensions to use Cerebus-Rex's tools:
- Search documentation directly from the editor
- Use filesystem tools for file operations
- Access indexed repository information

## Using MCP Protocol with Cerebus-Rex

The MCP protocol follows the standard specification with the following endpoints and methods:

### Server Information
```
{
  "method": "server/info",
  "id": "req-1"
}
```

### Tool Discovery
```
{
  "method": "tools/list",
  "id": "req-2"
}
```

Response includes tools like:
- `search` - Search documentation and code
- `filesystem_read` - Read a file from the filesystem
- `index_repo` - Index repository files for quick access
- `initialize_context` - Initialize AI agent context by indexing repository and documentation

### Tool Execution
```
{
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "query": "How to implement authentication",
      "context": "code"
    }
  },
  "id": "req-3"
}
```

#### Special Context Initialization Tool
The `initialize_context` tool allows AI agents to start creating their context by indexing repository files:

```
{
  "method": "tools/call",
  "params": {
    "name": "initialize_context",
    "arguments": {
      "repo_path": "/path/to/project/repository",
      "include_docs": true,
      "include_code": true
    }
  },
  "id": "req-initialize-1"
}
```

When an AI agent receives a prompt like "create your context using cerebus-rex", it can call this tool to auto-initialize its context with repository documentation and code.

### Resource Access
```
{
  "method": "resources/list",
  "id": "req-4"
}
```

```
{
  "method": "resources/read",
  "params": {
    "uri": "cerebus-rex://docs/getting-started"
  },
  "id": "req-5"
}
```

## Running with Docker

Cerebus-Rex can be deployed using Docker Compose with all required infrastructure:

### Prerequisites
- Docker Engine (v20.10.0 or higher)
- Docker Compose (v2.0.0 or higher)

### Quick Start
1. Copy the example environment file:
```bash
cp .env.example .env
```

2. Edit `.env` to set your configuration:
- Set `QDRANT_API_KEY` if using authentication
- Adjust other settings as needed

3. Start the services:
```bash
docker-compose up -d
```

4. The MCP server will be available at `ws://localhost:3000/mcp/ws`

### Docker Compose Services
- **qdrant**: Vector database for RAG system
- **cerebus-rex**: MCP server with all capabilities

### Configuration
The service supports configuration via environment variables:
- `APP__QDRANT__ENDPOINT`: Qdrant endpoint (default: http://qdrant:6333)
- `APP__QDRANT__API_KEY`: Qdrant API key (optional)
- `APP__SERVER__HOST`: Server host (default: 0.0.0.0)
- `APP__SERVER__PORT`: Server port (default: 3000)

## Development

To contribute to Cerebus-Rex:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests where applicable
5. Run the test suite: `cargo test`
6. Submit a pull request

## License

MIT License - see the LICENSE file for details.