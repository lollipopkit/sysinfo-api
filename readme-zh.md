# SysInfo API 服务器

[English](readme.md) | **中文**

一个基于 Rust 构建的高性能系统信息监控 API/MCP 服务器，使用 axum 框架和 sysinfo 库开发。支持 REST API 和模型上下文协议 (MCP) 接口，可灵活集成到 AI 助手和其他工具中。

## 快速开始

### 使用 Docker

```bash
docker compose up -d --build
```

### 本地构建

```bash
git clone https://github.com/lollipopkit/sysinfo-api
cd sysinfo-api
cargo run --release
```

## API 文档

详细信息请参考 [OpenAPI 文档](docs/api.yaml)。

## 默认配置

- REST API 端口: 8080
- MCP 服务器端口: 8081 (HTTP 模式)
- 用户名: admin
- 密码: password123
- 速率限制: 每分钟 60 次请求

## MCP 支持

此服务器支持模型上下文协议，使 AI 助手能够通过标准化工具直接访问系统信息。MCP 服务器提供以下工具：

- `get_system_info` - 获取完整的系统信息，包括 CPU、内存和进程
- `get_system_overview` - 获取系统概览（操作系统、内核、运行时间等）
- `get_cpu_info` - 获取 CPU 信息，包括使用率和核心详情
- `get_memory_info` - 获取内存信息，包括 RAM 和交换分区使用情况
- `get_processes` - 获取进程信息，包括占用 CPU 和内存最多的进程
- `get_timestamp` - 获取当前系统时间戳

### MCP 服务器模式

服务器可以在不同模式下运行，通过 `MCP_MODE` 环境变量控制：

- `stdio` - 仅作为 MCP 服务器运行，使用 stdio 传输（适用于 Claude Desktop）
- `http` - 仅作为 MCP 服务器运行，使用 HTTP 传输
- `both` - 同时运行 REST API 和 MCP HTTP 服务器（默认）
- `rest-only` - 仅运行 REST API 服务器

### Claude Desktop 的 MCP 配置

添加到您的 Claude Desktop 配置文件：

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

### MCP HTTP 服务器使用

在 HTTP 模式下运行时，MCP 服务器可在以下地址访问：

```text
http://localhost:8081/mcp
```

## 使用示例

### REST API 示例

```bash
# 健康检查
curl -u admin:password123 http://localhost:8080/api/v1/health

# 获取完整系统信息
curl -u admin:password123 http://localhost:8080/api/v1/system

# 获取 CPU 信息
curl -u admin:password123 http://localhost:8080/api/v1/system/cpu

# 获取内存信息
curl -u admin:password123 http://localhost:8080/api/v1/system/memory

# 获取进程信息
curl -u admin:password123 http://localhost:8080/api/v1/system/processes
```

## 许可证

[GPL v3](LICENSE) lollipopkit
