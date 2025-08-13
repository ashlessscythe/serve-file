use axum::Router;
use serde::Deserialize;
use std::{fs, net::SocketAddr};
use tokio::net::TcpListener;
use toml::from_str;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Deserialize)]
struct Config {
    port: u16,
    path: String,
}

#[tokio::main]
async fn main() {
    // Load config
    let config: Config = from_str(&fs::read_to_string("config.toml").unwrap()).unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let url = format!("http://{}", addr);

    // Launch browser (non-blocking)
    if let Err(e) = open::that(&url) {
        eprintln!("Failed to open browser: {}", e);
    }

    // Start server
    println!("Serving directory: {}", config.path);
    println!("Server will be available at: {}", url);

    let app = Router::new()
        .fallback_service(ServeDir::new(&config.path))
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server started successfully!");
    axum::serve(listener, app).await.unwrap();
}
