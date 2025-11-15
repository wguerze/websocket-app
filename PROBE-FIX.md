# Kubernetes Probe Configuration Fix

## Problem

The Kubernetes liveness and readiness probes were failing with the following symptoms:
- Pods stuck in `CrashLoopBackOff` or not becoming `Ready`
- Probe failures in pod events
- Containers being restarted repeatedly

## Root Cause

The WebSocket server was binding to `127.0.0.1:8080` (localhost only) instead of `0.0.0.0:8080` (all network interfaces).

**Why this matters in Kubernetes:**
- Kubernetes probes originate from outside the container
- They cannot reach `127.0.0.1` (localhost) from outside
- The server must listen on `0.0.0.0` to accept external connections

## Solution

### 1. Server Code Changes (`src/server.rs`)

**Before:**
```rust
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8080".to_string(),  // ❌ Localhost only
            max_connections: MAX_CONNECTIONS,
            ping_interval_secs: PING_INTERVAL_SECS,
        }
    }
}
```

**After:**
```rust
impl Default for ServerConfig {
    fn default() -> Self {
        // Read bind address from environment variable, default to 0.0.0.0:8080 for containers
        let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        Self {
            addr,  // ✅ Now binds to all interfaces
            max_connections: MAX_CONNECTIONS,
            ping_interval_secs: PING_INTERVAL_SECS,
        }
    }
}
```

### 2. Deployment Configuration (`k8s/base/deployment.yaml`)

**No changes needed** - Server defaults to `0.0.0.0:8080` now.

You can optionally override with:
```yaml
env:
- name: BIND_ADDR
  value: "0.0.0.0:8080"  # Optional override
```

**Improved probe configuration:**

Added **startup probe** (allows up to 60 seconds for app to start):
```yaml
startupProbe:
  tcpSocket:
    port: 8080
  initialDelaySeconds: 0
  periodSeconds: 2
  timeoutSeconds: 1
  failureThreshold: 30    # 30 attempts × 2s = 60s max startup time
  successThreshold: 1
```

Updated **liveness probe** (checks if app is still running):
```yaml
livenessProbe:
  tcpSocket:
    port: 8080
  initialDelaySeconds: 0  # Startup probe handles initial delay
  periodSeconds: 10
  timeoutSeconds: 1
  failureThreshold: 3     # Restart after 3 consecutive failures
  successThreshold: 1
```

Updated **readiness probe** (checks if app can accept traffic):
```yaml
readinessProbe:
  tcpSocket:
    port: 8080
  initialDelaySeconds: 0  # Startup probe handles initial delay
  periodSeconds: 5
  timeoutSeconds: 1
  failureThreshold: 2     # Mark not ready after 2 consecutive failures
  successThreshold: 1
```

### 3. Dockerfile

**No changes needed** - Server code defaults to `0.0.0.0:8080`.

## How Probes Work Now

### 1. Startup Probe (First)
- Starts immediately when container starts
- Checks every 2 seconds
- Allows up to 30 failures (60 seconds total)
- Once it succeeds, liveness and readiness probes take over
- **Purpose**: Give app time to start without being killed

### 2. Liveness Probe (After Startup)
- Checks every 10 seconds
- Restarts container after 3 consecutive failures (30 seconds)
- **Purpose**: Detect and restart crashed/hung apps

### 3. Readiness Probe (After Startup)
- Checks every 5 seconds
- Removes from service after 2 consecutive failures (10 seconds)
- **Purpose**: Stop sending traffic to unhealthy instances

## Probe Timeline Example

```
0s  → Container starts
0s  → Startup probe begins checking (every 2s)
2s  → Startup probe check #1 (may fail - app still starting)
4s  → Startup probe check #2 (may fail)
6s  → Startup probe check #3 (succeeds - app is up!)
6s  → Liveness probe activates
6s  → Readiness probe activates
11s → Readiness probe check #1 (app ready for traffic)
16s → Liveness probe check #1 (app still alive)
...continues monitoring
```

## Testing the Fix

### 1. Check Pod Status
```bash
kubectl get pods -n websocket-app-dev
```

Expected output:
```
NAME                                   READY   STATUS    RESTARTS   AGE
dev-websocket-server-xxxxx-xxxxx      1/1     Running   0          1m
```

### 2. Check Pod Events
```bash
kubectl describe pod <pod-name> -n websocket-app-dev
```

Look for:
- ✅ `Liveness probe succeeded`
- ✅ `Readiness probe succeeded`
- ✅ `Started container websocket-server`

### 3. Test Probe Endpoints
```bash
# Port-forward to the pod
kubectl port-forward -n websocket-app-dev <pod-name> 8080:8080

# In another terminal, test TCP connection
nc -zv localhost 8080
# Should output: Connection to localhost port 8080 [tcp/*] succeeded!
```

### 4. Check Logs
```bash
kubectl logs -n websocket-app-dev <pod-name>
```

Expected output:
```
[INFO] WebSocket Server listening on: 0.0.0.0:8080  ✅ Note: 0.0.0.0, not 127.0.0.1
[INFO] Maximum concurrent connections: 10
[INFO] Active connections: 0
```

## Troubleshooting

### Probes Still Failing?

**1. Check if server is binding correctly:**
```bash
kubectl logs -n websocket-app-dev <pod-name> | grep "listening on"
```

Should show `0.0.0.0:8080`, not `127.0.0.1:8080`

**2. Check if port is actually open:**
```bash
kubectl exec -it <pod-name> -n websocket-app-dev -- /bin/sh
# (May fail due to read-only filesystem, use next command)

kubectl debug -it <pod-name> -n websocket-app-dev --image=busybox
nc -zv 127.0.0.1 8080
```

**3. Check probe configuration:**
```bash
kubectl get pod <pod-name> -n websocket-app-dev -o yaml | grep -A 10 "livenessProbe"
```

**4. Increase probe timeout if needed:**
Edit `k8s/base/deployment.yaml`:
```yaml
startupProbe:
  failureThreshold: 60  # Increase from 30 to 60 (120 seconds)
```

### Common Issues

**Issue**: Pods restart frequently
- **Cause**: Liveness probe failing
- **Solution**: Check logs for errors, increase `failureThreshold`

**Issue**: Service has no endpoints
- **Cause**: Readiness probe failing
- **Solution**: Check if app is actually ready, increase `failureThreshold`

**Issue**: Pods stuck in `CrashLoopBackOff`
- **Cause**: Startup probe failing before app can start
- **Solution**: Increase `failureThreshold` or `periodSeconds` in startup probe

## Local Testing

To test locally with the new bind address:

```bash
# Server will bind to 0.0.0.0:8080
cargo run --bin server

# Or override bind address
BIND_ADDR=127.0.0.1:8080 cargo run --bin server
```

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `BIND_ADDR` | `0.0.0.0:8080` | Server bind address |
| `RUST_LOG` | `info` | Log level (debug, info, warn, error) |

## Summary of Changes

✅ **Server (`src/server.rs`)**: Reads `BIND_ADDR` from environment, defaults to `0.0.0.0:8080`
✅ **Probes (`k8s/base/deployment.yaml`)**: Added startup probe, improved liveness/readiness timing
✅ **Documentation**: Added this troubleshooting guide

**Note**: No Dockerfile or deployment.yaml env changes needed - the server code default handles it!

## Verification Checklist

After deploying the fix:

- [ ] Pods reach `Running` state
- [ ] Pods show `1/1 READY`
- [ ] No probe failures in `kubectl describe pod`
- [ ] Server logs show binding to `0.0.0.0:8080`
- [ ] Service has endpoints (`kubectl get endpoints`)
- [ ] Client can connect via port-forward
- [ ] No unexpected restarts (`RESTARTS` column = 0)

## Next Steps

Consider adding:
1. **HTTP health endpoint**: More informative than TCP checks
2. **Metrics endpoint**: For Prometheus monitoring
3. **Graceful shutdown**: Handle SIGTERM properly
4. **Connection draining**: Close connections gracefully on shutdown
