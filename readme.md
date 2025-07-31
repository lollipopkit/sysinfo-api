# SysInfo API Server

**English** | [中文](readme-zh.md)

A high-performance system information monitoring API/MCP server built with Rust using the axum framework and sysinfo library. Supports both REST API and Model Context Protocol (MCP) interfaces for flexible integration with AI assistants and other tools.

## Quick Start

### Using Docker

```bash
docker compose up -d --build
```

### Local Build

```bash
git clone https://github.com/lollipopkit/sysinfo-api
cd sysinfo-api
cargo run --release
```

## API Documentation

For details, please refer to the [OpenAPI Documentation](docs/api.yaml).

## Default Configuration

- REST API Port: 8080
- MCP Server Port: 8081 (HTTP mode)
- Username: admin
- Password: password123
- Rate Limit: 60 requests per minute

## MCP Support

This server supports the Model Context Protocol, enabling AI assistants to directly access system information through standardized tools. The MCP server provides the following tools:

- `get_system_info` - Get complete system information including CPU, memory, and processes
- `get_system_overview` - Get system overview (OS, kernel, uptime, etc.)
- `get_cpu_info` - Get CPU information including usage and core details
- `get_memory_info` - Get memory information including RAM and swap usage
- `get_processes` - Get process information with top CPU and memory consumers
- `get_timestamp` - Get current system timestamp

### MCP Server Modes

The server can run in different modes controlled by the `MCP_MODE` environment variable:

- `stdio` - Run only as MCP server using stdio transport (for Claude Desktop)
- `http` - Run only as MCP server using HTTP transport
- `both` - Run both REST API and MCP HTTP server (default)
- `rest-only` - Run only the REST API server

### MCP Configuration for Claude Desktop

Add to your Claude Desktop configuration file:

```json
{
  "mcpServers": {
    "sysinfo-api": {
      "command": "/path/to/sysinfo-api",
      "env": {
        "MCP_MODE": "stdio"
      }
    }
  }
}
```

### MCP HTTP Server Usage

When running in HTTP mode, the MCP server is available at:

```text
http://localhost:8081/mcp
```

## Usage Examples

### REST API Examples

```bash
# Health check
curl -u admin:password123 http://localhost:8080/api/v1/health

# Get complete system information
curl -u admin:password123 http://localhost:8080/api/v1/system

# Get CPU information
curl -u admin:password123 http://localhost:8080/api/v1/system/cpu

# Get memory information
curl -u admin:password123 http://localhost:8080/api/v1/system/memory

# Get process information
curl -u admin:password123 http://localhost:8080/api/v1/system/processes
```

## License

[GPL v3](LICENSE) lollipopkit
