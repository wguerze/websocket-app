# WebSocket Test Server & Client

![CI/CD Pipeline](https://github.com/williamguerzeder/websocket-app/actions/workflows/ci-cd.yml/badge.svg)

A Rust-based WebSocket testing application with server and interactive client. Built for testing WebSocket connections, keeping them alive, and managing multiple concurrent connections.

## Features

### Server
- Accepts WebSocket connections on `127.0.0.1:8080`
- Supports up to **10 concurrent connections** (rejects additional connections)
- **Comprehensive logging**:
  - Connection opened/closed events with client addresses
  - Active connection count every 5 seconds
  - Message activity logging
- Keeps connections alive with periodic ping/pong
- Echo server functionality for testing

### Client
- Interactive CLI for managing multiple WebSocket connections
- Create single or multiple connections at once
- Keep connections alive until manually closed
- Send messages to specific connections
- List all active connections
- Colored output for better readability

## CI/CD & Docker

### Continuous Integration
The project uses GitHub Actions for automated testing and Docker image building:

- **Automated Testing**: Runs on every push and pull request
  - Code formatting checks (`cargo fmt`)
  - Linting with Clippy (`cargo clippy`)
  - Unit and integration tests (`cargo test`)
  - Dependency caching for faster builds

- **Docker Image Building**: Automatically builds and pushes Docker images
  - Multi-stage build for optimized image size
  - Published to GitHub Container Registry (ghcr.io)
  - Tagged with branch name, commit SHA, and `latest` for main branch

### Running with Docker

**Pull and run the pre-built image:**
```bash
docker pull ghcr.io/williamguerzeder/websocket-app:latest
docker run -p 8080:8080 ghcr.io/williamguerzeder/websocket-app:latest
```

**Build locally:**
```bash
docker build -t websocket-server .
docker run -p 8080:8080 websocket-server
```

**With custom configuration:**
```bash
docker run -p 9090:8080 -e RUST_LOG=debug websocket-server
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run only server tests
cargo test --bin server

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --bin server
```

## Requirements

- Rust 1.70+ (with Cargo)

## Build

```bash
cargo build --release
```

## Usage

### 1. Start the Server

In one terminal:

```bash
cargo run --bin server
```

Expected output:
```
[INFO] WebSocket Server listening on: 127.0.0.1:8080
[INFO] Maximum concurrent connections: 10
[INFO] Active connections: 0
```

The server will log every 5 seconds showing the number of active connections.

### 2. Start the Client

In another terminal:

```bash
cargo run --bin client
```

You'll see an interactive prompt:
```
=== WebSocket Test Client ===
Type 'help' for available commands

>
```

## Client Commands

### Connection Management

| Command | Description | Example |
|---------|-------------|---------|
| `connect` or `c` | Create a new connection | `connect` |
| `connect <count>` | Create multiple connections | `connect 5` |
| `close <id>` | Close a specific connection | `close 1` |
| `close all` | Close all connections | `close all` |
| `list` or `ls` | List active connections | `list` |

### Messaging

| Command | Description | Example |
|---------|-------------|---------|
| `send <id> <message>` or `s <id> <message>` | Send a message to a connection | `send 1 hello` |

### Other

| Command | Description |
|---------|-------------|
| `help` or `h` | Show help |
| `quit`, `exit`, or `q` | Quit the client |

## Testing Examples

### Test Single Connection

```
> connect
✓ Connection #1 established
← Connection #1: Connected to WebSocket server

> send 1 Hello Server
✓ Sent to connection #1: Hello Server
← Connection #1: Echo: Hello Server

> close 1
✓ Closed connection #1
```

### Test Multiple Connections

```
> connect 10
Creating 10 connections...
✓ Connection #1 established
✓ Connection #2 established
...
✓ Connection #10 established

> list
Active connections:
  • Connection #1
  • Connection #2
  ...
  • Connection #10

> connect
✗ Failed to connect: ...
```

The 11th connection should fail because the server limits connections to 10.

### Server Log Output

While connections are active, the server logs:

```
[INFO] Connection opened from 127.0.0.1:52431 (total active: 1)
[INFO] Connection opened from 127.0.0.1:52432 (total active: 2)
[INFO] Active connections: 2
[INFO] Received from 127.0.0.1:52431: Hello Server
[INFO] Active connections: 2
[INFO] Client 127.0.0.1:52431 initiated close
[INFO] Connection closed from 127.0.0.1:52431 (total active: 1)
```

### Test Connection Limit

```bash
# In the client
> connect 11
Creating 11 connections...
✓ Connection #1 established
...
✓ Connection #10 established
✗ Failed to connect: ...
```

Server will log:
```
[WARN] Connection limit reached (10), rejecting connection from 127.0.0.1:xxxxx
```

## Configuration

### Server (`src/server.rs`)

- `MAX_CONNECTIONS`: Maximum concurrent connections (default: 10)
- `PING_INTERVAL_SECS`: Seconds between keep-alive pings (default: 30)
- Server address: Change `127.0.0.1:8080` to bind to different address/port

### Client (`src/client.rs`)

- `SERVER_URL`: Server URL to connect to (default: `ws://127.0.0.1:8080`)

## Project Structure

```
websocket-app/
├── Cargo.toml              # Project dependencies
├── src/
│   ├── server.rs           # WebSocket server
│   └── client.rs           # Interactive client
├── audio/
│   └── sample.mp3          # (Legacy file, not used)
└── README.md               # This file
```

## How It Works

### Server
1. Accepts WebSocket connections up to the limit (10)
2. Sends a welcome message to each client
3. Echoes back any text messages received
4. Sends periodic pings to keep connections alive
5. Logs connection events and counts active connections every 5 seconds
6. Gracefully handles client disconnections

### Client
1. Creates WebSocket connections to the server
2. Each connection runs in its own async task
3. Displays messages received from the server
4. Allows sending messages through an interactive CLI
5. Manages multiple connections with unique IDs
6. Closes connections on demand

## License

See LICENSE file for details.
