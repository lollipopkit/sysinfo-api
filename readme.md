# SysInfo API Server

**English** | [中文](readme-zh.md)

A high-performance system information monitoring API server built with Rust using the axum framework and sysinfo library.

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

- Port: 8080
- Username: admin
- Password: password123
- Rate Limit: 60 requests per minute

## Usage Examples

```bash
# Health check
curl -u admin:password123 http://localhost:8080/api/v1/health
```

## License

[GPL v3](LICENSE) lollipopkit
