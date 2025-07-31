use axum::Router;
use serde::Deserialize;
use std::{fs, net::SocketAddr};
use tower_http::services::ServeDir;

#[derive(Deserialize)]
struct Config {
    port: u16,
    path: String,
}

#[tokio::main]
async fn main() {
    // Load config
    let config: Config = toml::from_str(&fs::read_to_string("config.toml").unwrap()).unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let url = format!("http://{}", addr);

    // Launch browser (non-blocking)
    if let Err(e) = open::that(&url) {
        eprintln!("Failed to open browser: {}", e);
    }

    // Start server
    let app = Router::new().fallback_service(ServeDir::new(config.path));
    println!("Serving on {}", url);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
