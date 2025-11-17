# WebSocket Client Usage Guide

The interactive WebSocket client supports connecting to custom server URLs via command-line arguments.

## Basic Usage

### Default Server (localhost)

```bash
cargo run --bin client
```

Connects to: `ws://127.0.0.1:8080`

### Custom Server URL

```bash
# Using long flag
cargo run --bin client -- --server ws://example.com:8080

# Using short flag
cargo run --bin client -- -s ws://example.com:8080

# With wss:// (secure WebSocket)
cargo run --bin client -- --server wss://secure.example.com
```

## Environment-Specific Examples

### Development
```bash
cargo run --bin client -- --server ws://websocket-dev.example.com
```

### Staging
```bash
cargo run --bin client -- --server ws://websocket-staging.example.com
```

### Production (with TLS)
```bash
cargo run --bin client -- --server wss://websocket.example.com
```

### Local Kubernetes (port-forward)
```bash
# In terminal 1: Port forward
kubectl port-forward -n websocket-app-dev svc/dev-websocket-server 8080:8080

# In terminal 2: Connect client (default works)
cargo run --bin client
```

## Help

### Command-Line Help

```bash
cargo run --bin client -- --help
```

Output:
```
Interactive WebSocket client for testing

Usage: client [OPTIONS]

Options:
  -s, --server <SERVER>  WebSocket server URL to connect to [default: ws://127.0.0.1:8080]
  -h, --help            Print help
  -V, --version         Print version
```

### Interactive Help

Once connected, type `help` for available commands:

```
> help

Available Commands:

  connect  [count]  - Create a new WebSocket connection
  c     [count]  - Alias for connect
  close    <id|all>  - Close a connection (or 'all')
  list          - List all active connections
  ls            - Alias for list
  send <id> <message> - Send a message to a connection
  s      <id> <message> - Alias for send
  help          - Show this help message
  h            - Alias for help
  quit    - Quit the client
  exit    - Alias for quit
  q      - Alias for quit

Note:
  Use --server or -s flag to specify a custom server URL:
  cargo run --bin client -- --server ws://example.com

Examples:
  connect       - Create 1 connection
  connect 5     - Create 5 connections
  list          - Show all connections
  send 1 hello  - Send 'hello' to connection #1
  close 1       - Close connection #1
  close all     - Close all connections
```

## Interactive Commands

Once the client is running, use these commands:

### Connection Management

| Command | Description | Example |
|---------|-------------|---------|
| `connect` or `c` | Create 1 connection | `connect` |
| `connect <n>` | Create n connections | `connect 5` |
| `close <id>` | Close specific connection | `close 1` |
| `close all` | Close all connections | `close all` |
| `list` or `ls` | List active connections | `list` |

### Messaging

| Command | Description | Example |
|---------|-------------|---------|
| `send <id> <msg>` or `s <id> <msg>` | Send message | `send 1 Hello!` |

### Other

| Command | Description |
|---------|-------------|
| `help` or `h` | Show help |
| `quit`, `exit`, or `q` | Exit client |

## Example Session

```bash
# Start client with custom server
$ cargo run --bin client -- --server wss://websocket.example.com

=== WebSocket Test Client ===
Server URL: wss://websocket.example.com
Type 'help' for available commands

> connect 3
Creating 3 connections...
✓ Connection #1 established
← Connection #1: Connected to WebSocket server
✓ Connection #2 established
← Connection #2: Connected to WebSocket server
✓ Connection #3 established
← Connection #3: Connected to WebSocket server

> list
Active connections:
  • Connection #1
  • Connection #2
  • Connection #3

> send 1 Hello from client!
✓ Sent to connection #1: Hello from client!
← Connection #1: Echo: Hello from client!

> send 2 Testing connection 2
✓ Sent to connection #2: Testing connection 2
← Connection #2: Echo: Testing connection 2

> close 1
✓ Closed connection #1

> list
Active connections:
  • Connection #2
  • Connection #3

> close all
✓ Closed 2 connection(s)

> quit
Closing all connections and exiting...
```

## Testing Different Environments

### Test Development Server
```bash
# Option 1: Port-forward
kubectl port-forward -n websocket-app-dev svc/dev-websocket-server 8080:8080
cargo run --bin client

# Option 2: Via Ingress
cargo run --bin client -- --server ws://websocket-dev.example.com
```

### Test Staging Server
```bash
cargo run --bin client -- --server ws://websocket-staging.example.com
```

### Test Production Server
```bash
cargo run --bin client -- --server wss://websocket.example.com
```

## URL Format

The client supports standard WebSocket URL formats:

### Insecure (ws://)
```bash
ws://hostname:port
ws://127.0.0.1:8080
ws://websocket.example.com
ws://websocket.example.com:9090
```

### Secure (wss://)
```bash
wss://hostname:port
wss://websocket.example.com
wss://websocket.example.com:443
```

**Note**:
- `ws://` uses port 80 by default
- `wss://` uses port 443 by default
- Explicit port can be specified with `:port`

## Environment Variables

You can also use environment variables (requires code modification):

```rust
// src/client.rs - add this
const DEFAULT_SERVER_URL: &str = option_env!("WS_SERVER_URL")
    .unwrap_or("ws://127.0.0.1:8080");
```

Then:
```bash
WS_SERVER_URL=ws://example.com cargo run --bin client
```

## Troubleshooting

### Connection Refused

**Error**: `Failed to connect: Connection refused`

**Causes**:
1. Server not running
2. Wrong URL/port
3. Firewall blocking connection

**Solutions**:
```bash
# Check if server is accessible
nc -zv hostname port

# Test with curl
curl -I http://hostname:port
```

### TLS/SSL Errors

**Error**: `Failed to connect: SSL error`

**Causes**:
1. Using `wss://` but server doesn't have TLS
2. Invalid certificate
3. Self-signed certificate

**Solutions**:
```bash
# Use ws:// instead of wss:// for local testing
cargo run --bin client -- --server ws://example.com

# For production with valid TLS
cargo run --bin client -- --server wss://example.com
```

### Invalid URL

**Error**: `Failed to connect: invalid url`

**Solution**: Ensure URL starts with `ws://` or `wss://`:
```bash
# ✗ Wrong
cargo run --bin client -- --server example.com

# ✓ Correct
cargo run --bin client -- --server ws://example.com
```

## Tips

1. **Tab completion**: Build first, then use autocomplete
   ```bash
   cargo build --bin client
   ./target/debug/client --server ws://example.com
   ```

2. **Multiple terminals**: Run multiple clients for load testing
   ```bash
   # Terminal 1
   cargo run --bin client -- --server ws://example.com

   # Terminal 2
   cargo run --bin client -- --server ws://example.com
   ```

3. **Test connection limits**: Create 11+ connections to test the 10-connection limit
   ```bash
   > connect 11
   # 10 should succeed, 11th should fail
   ```

4. **Keep connections alive**: Connections stay open until you close them or quit
   - Server sends ping every 30 seconds
   - Client responds automatically
   - No manual action needed

## Integration with Kubernetes

### Port-Forward Pattern
```bash
# Terminal 1: Port-forward
kubectl port-forward -n websocket-app-dev svc/dev-websocket-server 8080:8080

# Terminal 2: Client (use default URL)
cargo run --bin client
> connect
```

### Ingress Pattern
```bash
# Direct connection via Ingress
cargo run --bin client -- --server ws://websocket-dev.example.com
> connect
```

### Testing All Environments
```bash
# Dev
cargo run --bin client -- -s ws://websocket-dev.example.com

# Staging
cargo run --bin client -- -s ws://websocket-staging.example.com

# Production
cargo run --bin client -- -s wss://websocket.example.com
```

## Summary

✅ **Default**: Connects to `ws://127.0.0.1:8080`
✅ **Custom URL**: Use `--server` or `-s` flag
✅ **Secure**: Support for `wss://` (secure WebSocket)
✅ **Flexible**: Works with localhost, Ingress, or any WebSocket server
✅ **Interactive**: Full connection management via CLI commands
