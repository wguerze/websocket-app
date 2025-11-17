use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::{interval, Duration};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

pub const MAX_CONNECTIONS: usize = 10;
pub const PING_INTERVAL_SECS: u64 = 30;

pub struct ServerConfig {
    pub addr: String,
    pub max_connections: usize,
    pub ping_interval_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        // Read bind address from environment variable, default to 0.0.0.0:8080 for containers
        let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        Self {
            addr,
            max_connections: MAX_CONNECTIONS,
            ping_interval_secs: PING_INTERVAL_SECS,
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let config = ServerConfig::default();

    // Start health check server on port 8081
    tokio::spawn(run_health_server());

    run_server(config).await;
}

pub async fn run_server(config: ServerConfig) {
    let listener = TcpListener::bind(&config.addr)
        .await
        .expect("Failed to bind");
    info!("WebSocket Server listening on: {}", config.addr);
    info!("Maximum concurrent connections: {}", config.max_connections);

    // Semaphore to limit concurrent connections
    let connection_limit = Arc::new(Semaphore::new(config.max_connections));
    let active_connections = Arc::new(tokio::sync::RwLock::new(0u32));

    // Spawn periodic connection counter logger
    let active_conn_clone = active_connections.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let count = *active_conn_clone.read().await;
            info!("Active connections: {}", count);
        }
    });

    // Accept incoming connections
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let permit = connection_limit.clone().try_acquire_owned();
                let active_conn = active_connections.clone();

                match permit {
                    Ok(permit) => {
                        tokio::spawn(async move {
                            handle_connection(
                                stream,
                                addr,
                                active_conn,
                                permit,
                                config.ping_interval_secs,
                            )
                            .await;
                        });
                    }
                    Err(_) => {
                        warn!(
                            "Connection limit reached ({}), rejecting connection from {}",
                            config.max_connections, addr
                        );
                        tokio::spawn(async move {
                            let _ = send_503_response(stream).await;
                        });
                    }
                }
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    active_connections: Arc<tokio::sync::RwLock<u32>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    ping_interval_secs: u64,
) {
    // Increment active connection counter
    {
        let mut count = active_connections.write().await;
        *count += 1;
        info!("Connection opened from {} (total active: {})", addr, *count);
    }

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed for {}: {}", addr, e);
            decrement_counter(active_connections, addr).await;
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Send initial welcome message
    if let Err(e) = write
        .send(Message::Text("Connected to WebSocket server".to_string()))
        .await
    {
        error!("Failed to send welcome message to {}: {}", addr, e);
        decrement_counter(active_connections, addr).await;
        return;
    }

    // Spawn ping task to keep connection alive
    let (ping_tx, mut ping_rx) = tokio::sync::mpsc::channel::<()>(1);
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(ping_interval_secs));
        loop {
            interval.tick().await;
            if ping_tx.send(()).await.is_err() {
                break; // Connection closed
            }
        }
    });

    // Handle incoming messages and pings
    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = read.next() => {
                match msg {
                    Some(Ok(message)) => {
                        match message {
                            Message::Text(text) => {
                                info!("Received from {}: {}", addr, text);
                                // Echo back
                                if let Err(e) = write.send(Message::Text(format!("Echo: {}", text))).await {
                                    error!("Failed to send echo to {}: {}", addr, e);
                                    break;
                                }
                            }
                            Message::Binary(data) => {
                                info!("Received {} bytes from {}", data.len(), addr);
                            }
                            Message::Close(_) => {
                                info!("Client {} initiated close", addr);
                                break;
                            }
                            Message::Ping(data) => {
                                if let Err(e) = write.send(Message::Pong(data)).await {
                                    error!("Failed to send pong to {}: {}", addr, e);
                                    break;
                                }
                            }
                            Message::Pong(_) => {
                                // Received pong response
                            }
                            _ => {}
                        }
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error for {}: {}", addr, e);
                        break;
                    }
                    None => {
                        info!("Connection closed by {}", addr);
                        break;
                    }
                }
            }
            // Send periodic pings
            _ = ping_rx.recv() => {
                if let Err(e) = write.send(Message::Ping(vec![])).await {
                    error!("Failed to send ping to {}: {}", addr, e);
                    break;
                }
            }
        }
    }

    // Close the connection gracefully
    let _ = write.close().await;

    decrement_counter(active_connections, addr).await;
}

async fn decrement_counter(active_connections: Arc<tokio::sync::RwLock<u32>>, addr: SocketAddr) {
    let mut count = active_connections.write().await;
    *count = count.saturating_sub(1);
    info!("Connection closed from {} (total active: {})", addr, *count);
}

async fn send_503_response(mut stream: TcpStream) -> std::io::Result<()> {
    let response = "HTTP/1.1 503 Service Unavailable\r\n\
                    Content-Type: text/plain\r\n\
                    Content-Length: 50\r\n\
                    Connection: close\r\n\
                    \r\n\
                    Maximum concurrent connections limit reached (10)";

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    stream.shutdown().await?;
    Ok(())
}

pub async fn run_health_server() {
    let health_addr = "0.0.0.0:8081";
    let listener = match TcpListener::bind(health_addr).await {
        Ok(l) => l,
        Err(e) => {
            error!(
                "Failed to bind health check server to {}: {}",
                health_addr, e
            );
            return;
        }
    };

    info!("Health check server listening on: {}", health_addr);

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                tokio::spawn(async move {
                    let response = "HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/plain\r\n\
                                    Content-Length: 2\r\n\
                                    Connection: close\r\n\
                                    \r\n\
                                    OK";

                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.flush().await;
                    let _ = stream.shutdown().await;
                });
            }
            Err(e) => {
                error!("Failed to accept health check connection: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.addr, "0.0.0.0:8080");
        assert_eq!(config.max_connections, MAX_CONNECTIONS);
        assert_eq!(config.ping_interval_secs, PING_INTERVAL_SECS);
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_CONNECTIONS, 10);
        assert_eq!(PING_INTERVAL_SECS, 30);
    }

    #[tokio::test]
    async fn test_server_starts_and_accepts_connection() {
        // Start server on a random port
        let config = ServerConfig {
            addr: "127.0.0.1:0".to_string(),
            max_connections: 10,
            ping_interval_secs: 30,
        };

        let listener = TcpListener::bind(&config.addr).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_url = format!("ws://{}", addr);

        // Spawn server task
        tokio::spawn(async move {
            if let Ok((stream, client_addr)) = listener.accept().await {
                let active_connections = Arc::new(tokio::sync::RwLock::new(0u32));
                let permit = Arc::new(Semaphore::new(10)).try_acquire_owned().unwrap();
                handle_connection(stream, client_addr, active_connections, permit, 30).await;
            }
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Try to connect
        let connect_result = timeout(
            tokio::time::Duration::from_secs(5),
            connect_async(&server_url),
        )
        .await;

        assert!(connect_result.is_ok(), "Should connect to server");
        if let Ok(Ok((mut ws_stream, _))) = connect_result {
            // Receive welcome message
            if let Ok(Some(Ok(msg))) =
                timeout(tokio::time::Duration::from_secs(2), ws_stream.next()).await
            {
                if let Message::Text(text) = msg {
                    assert_eq!(text, "Connected to WebSocket server");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_active_connection_counter() {
        let active_connections = Arc::new(tokio::sync::RwLock::new(0u32));

        // Initially 0
        assert_eq!(*active_connections.read().await, 0);

        // Increment
        {
            let mut count = active_connections.write().await;
            *count += 1;
        }
        assert_eq!(*active_connections.read().await, 1);

        // Increment again
        {
            let mut count = active_connections.write().await;
            *count += 1;
        }
        assert_eq!(*active_connections.read().await, 2);

        // Decrement
        {
            let mut count = active_connections.write().await;
            *count = count.saturating_sub(1);
        }
        assert_eq!(*active_connections.read().await, 1);
    }

    #[tokio::test]
    async fn test_connection_limit_with_semaphore() {
        let max_connections = 3;
        let semaphore = Arc::new(Semaphore::new(max_connections));

        // Acquire 3 permits
        let permit1 = semaphore.clone().try_acquire_owned();
        assert!(permit1.is_ok());

        let permit2 = semaphore.clone().try_acquire_owned();
        assert!(permit2.is_ok());

        let permit3 = semaphore.clone().try_acquire_owned();
        assert!(permit3.is_ok());

        // 4th should fail
        let permit4 = semaphore.clone().try_acquire_owned();
        assert!(permit4.is_err());

        // Drop one permit and try again
        drop(permit1);
        let permit5 = semaphore.clone().try_acquire_owned();
        assert!(permit5.is_ok());
    }

    #[test]
    fn test_server_config_custom() {
        let config = ServerConfig {
            addr: "0.0.0.0:9090".to_string(),
            max_connections: 5,
            ping_interval_secs: 60,
        };

        assert_eq!(config.addr, "0.0.0.0:9090");
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.ping_interval_secs, 60);
    }
}
