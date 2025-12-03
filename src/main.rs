use axum::Router;
use serde::Deserialize;
use std::{fs, net::SocketAddr};
use tokio::net::TcpListener;
use toml::from_str;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Config {
    pub port: u16,
    pub path: String,
}

impl Config {
    /// Load config from a TOML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = from_str(&content)?;
        Ok(config)
    }

    /// Load config from a TOML string
    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = from_str(content)?;
        Ok(config)
    }

    /// Get the socket address for this config
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self.port))
    }

    /// Get the URL for this config
    pub fn url(&self) -> String {
        format!("http://{}", self.socket_addr())
    }
}

/// Create the Axum router for serving files
pub fn create_app(path: &str) -> Router {
    Router::new()
        .fallback_service(ServeDir::new(path))
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    // Load config
    let config = Config::from_file("config.toml").unwrap();
    let addr = config.socket_addr();
    let url = config.url();

    // Launch browser (non-blocking)
    if let Err(e) = open::that(&url) {
        eprintln!("Failed to open browser: {}", e);
    }

    // Start server
    println!("Serving directory: {}", config.path);
    println!("Server will be available at: {}", url);

    let app = create_app(&config.path);
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server started successfully!");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    #[test]
    fn test_config_from_str() {
        let toml_content = r#"
            port = 8080
            path = "/tmp/test"
        "#;

        let config = Config::from_str(toml_content).unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.path, "/tmp/test");
    }

    #[test]
    fn test_config_from_str_invalid_toml() {
        let invalid_toml = "this is not valid toml";
        assert!(Config::from_str(invalid_toml).is_err());
    }

    #[test]
    fn test_config_from_str_missing_fields() {
        let incomplete_toml = r#"
            port = 8080
        "#;
        assert!(Config::from_str(incomplete_toml).is_err());
    }

    #[test]
    fn test_config_socket_addr() {
        let config = Config {
            port: 3000,
            path: "/tmp".to_string(),
        };
        let addr = config.socket_addr();
        assert_eq!(addr.port(), 3000);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[test]
    fn test_config_url() {
        let config = Config {
            port: 9000,
            path: "/tmp".to_string(),
        };
        assert_eq!(config.url(), "http://127.0.0.1:9000");
    }

    #[test]
    fn test_config_from_file() {
        // Create a temporary config file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        fs::write(
            &config_path,
            r#"
            port = 5000
            path = "/tmp/serve"
        "#,
        )
        .unwrap();

        let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.port, 5000);
        assert_eq!(config.path, "/tmp/serve");
    }

    #[test]
    fn test_config_from_file_not_found() {
        assert!(Config::from_file("nonexistent_config.toml").is_err());
    }

    /// Helper function to start a test server
    async fn start_test_server(path: &str, port: u16) -> (SocketAddr, JoinHandle<()>) {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let app = create_app(path);
        let listener = TcpListener::bind(addr).await.unwrap();
        let actual_addr = listener.local_addr().unwrap();

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        (actual_addr, handle)
    }

    #[tokio::test]
    async fn test_create_app_serves_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test.txt");
        fs::write(&test_file_path, "Hello, World!").unwrap();

        let (addr, handle) = start_test_server(
            temp_dir.path().to_str().unwrap(),
            0, // Use port 0 to get a random available port
        )
        .await;

        let url = format!("http://{}/test.txt", addr);
        let response = reqwest::get(&url).await.unwrap();

        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(response.text().await.unwrap(), "Hello, World!");

        handle.abort();
    }

    #[tokio::test]
    async fn test_create_app_file_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let (addr, handle) = start_test_server(temp_dir.path().to_str().unwrap(), 0).await;

        let url = format!("http://{}/nonexistent.txt", addr);
        let response = reqwest::get(&url).await.unwrap();

        assert_eq!(response.status().as_u16(), 404);

        handle.abort();
    }

    #[tokio::test]
    async fn test_create_app_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index.html");
        fs::write(&index_path, "<html><body>Index</body></html>").unwrap();

        let (addr, handle) = start_test_server(temp_dir.path().to_str().unwrap(), 0).await;

        let url = format!("http://{}/index.html", addr);
        let response = reqwest::get(&url).await.unwrap();

        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(
            response.text().await.unwrap(),
            "<html><body>Index</body></html>"
        );

        handle.abort();
    }
}
