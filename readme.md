# SysInfo API Server

A high-performance system information monitoring API server built with Rust using the ntex framework and sysinfo library.

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

## Usage Examples

```bash
# Health check
curl -u admin:password123 http://localhost:8080/api/v1/health
```
