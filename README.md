# serve-file

A simple HTTP file server written in Rust using Axum. Serves static files from a specified directory with automatic browser launch.

## Features

- Static file serving from configurable directory
- Automatic browser launch on startup
- HTTP request logging
- Configurable port via TOML configuration

## Usage

1. Configure `config.toml` with your desired port and file path
2. Run the server: `cargo run`
3. Browser will automatically open to the served directory

## Configuration

Create a `config.toml` file:

```toml
port = 3000
path = "./public"
```

## Dependencies

- axum - Web framework
- tokio - Async runtime
- tower-http - HTTP middleware and services
- serde - Configuration deserialization
- toml - Configuration file parsing
- open - Browser launching

## License

MIT
