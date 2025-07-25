# SysInfo API 服务器

[English](readme.md) | **中文**

一个基于 Rust 构建的高性能系统信息监控 API 服务器，使用 ntex 框架和 sysinfo 库开发。

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

## 使用示例

```bash
# 健康检查
curl -u admin:password123 http://localhost:8080/api/v1/health
```

## 默认配置

- 端口: 8080
- 用户名: admin
- 密码: password123
- 速率限制: 每分钟 60 次请求

## 许可证

[GPL v3](LICENSE) lollipopkit
