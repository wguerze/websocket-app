# Load Balancing Guide for WebSocket Connections

## The Problem

When testing with `kubectl port-forward` and the client, all connections go to one pod instead of distributing across multiple replicas.

## Why This Happens

### 1. Port-Forward Limitation

`kubectl port-forward` **does NOT load balance**. It picks one random pod and forwards all traffic to it.

```bash
# This goes to ONE pod only, not load balanced
kubectl port-forward svc/dev-websocket-server 8080:8080
```

**How port-forward works:**
1. Queries service for endpoints
2. Picks the first pod (or random pod)
3. Creates direct tunnel to that specific pod
4. All traffic goes through that tunnel

### 2. Service Session Affinity

The service has `sessionAffinity: ClientIP`:

```yaml
# k8s/base/service.yaml
sessionAffinity: ClientIP
sessionAffinityConfig:
  clientIP:
    timeoutSeconds: 10800  # 3 hours
```

**What this means:**
- All connections from the **same client IP** go to the **same pod**
- Ensures WebSocket connections stay on the same backend
- Prevents connection issues when load balancing WebSocket
- Lasts for 3 hours (10800 seconds)

**Why it's needed:**
WebSocket connections are stateful and long-lived. If you load balance individual WebSocket messages to different pods, connections will break.

## Solutions

### Option 1: Use Ingress (Production Method)

The Ingress controller properly distributes connections with session affinity.

**Deploy with Ingress:**
```bash
# Ensure Ingress is deployed
kubectl get ingress -n websocket-app-dev

# Update DNS or use /etc/hosts
echo "<EXTERNAL_IP> websocket-dev.example.com" | sudo tee -a /etc/hosts

# Connect via Ingress
cargo run --bin client -- --server ws://websocket-dev.example.com
> connect 20
```

**How Ingress load balances:**
1. Each new **client** is routed to a pod (round-robin or least-connections)
2. That client's connections stick to the same pod (session affinity)
3. Different clients go to different pods
4. Result: Even distribution across pods

**Test with multiple clients:**
```bash
# Terminal 1
cargo run --bin client -- --server ws://websocket-dev.example.com
> connect 10

# Terminal 2
cargo run --bin client -- --server ws://websocket-dev.example.com
> connect 10

# Terminal 3 - Check pod distribution
kubectl get pods -n websocket-app-dev -o wide
kubectl logs -n websocket-app-dev -l app=websocket-server | grep "Active connections"
```

### Option 2: Port-Forward to Individual Pods

Test each pod separately to verify they both work:

```bash
# Get pod names
kubectl get pods -n websocket-app-dev

# Terminal 1: Port-forward to pod 1
kubectl port-forward -n websocket-app-dev pod/dev-websocket-server-xxxxx-aaaa 8080:8080

# Test pod 1
cargo run --bin client
> connect 10

# Terminal 2: Port-forward to pod 2
kubectl port-forward -n websocket-app-dev pod/dev-websocket-server-xxxxx-bbbb 8081:8080

# Test pod 2 (different local port)
cargo run --bin client -- --server ws://127.0.0.1:8081
> connect 10
```

This confirms both pods can handle 10 connections each = 20 total.

### Option 3: Create a Test Service Without Session Affinity

Create a separate service for testing load balancing:

```yaml
# test-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: websocket-server-test
  namespace: websocket-app-dev
spec:
  type: ClusterIP
  selector:
    app: websocket-server
  ports:
  - port: 8080
    targetPort: 8080
  sessionAffinity: None  # No session affinity
```

Apply:
```bash
kubectl apply -f test-service.yaml

# Port-forward to test service
kubectl port-forward -n websocket-app-dev svc/websocket-server-test 8080:8080

# Connect - may distribute better, but might break WebSocket
cargo run --bin client
> connect 20
```

**Warning**: Removing session affinity might cause WebSocket issues if the service load balances individual messages to different pods.

### Option 4: Test with Multiple Client IPs

Simulate multiple clients from different IPs:

**Method 1: Use different network interfaces**
```bash
# From host machine
cargo run --bin client -- --server ws://websocket-dev.example.com
> connect 10

# From inside a pod (different IP)
kubectl run -it --rm debug --image=alpine --restart=Never -- sh
apk add curl
# Install websocat and connect
```

**Method 2: Use HAProxy or NGINX locally**
```bash
# Create local load balancer that changes source IP
# This is complex, use Ingress instead
```

### Option 5: Use Ingress with Different Session Cookies

The Ingress uses cookie-based session affinity, which works better for testing from one client:

```bash
# Each browser/client gets a different cookie
# Can simulate multiple clients from same machine

# Client 1
cargo run --bin client -- --server ws://websocket-dev.example.com
> connect 10

# Client 2 (separate process = separate cookie)
cargo run --bin client -- --server ws://websocket-dev.example.com
> connect 10
```

## Understanding the Behavior

### With Session Affinity (Current Setup)

```
Client (127.0.0.1)
    ↓
Service (sessionAffinity: ClientIP)
    ↓
Pod 1 ← All 20 connections from this client
Pod 2 ← No connections
```

**This is CORRECT for production!** Different clients will go to different pods:

```
Client A (1.2.3.4)     Client B (5.6.7.8)
    ↓                      ↓
    Service (sessionAffinity: ClientIP)
    ↓                      ↓
Pod 1 ← Client A      Pod 2 ← Client B
(10 connections)      (10 connections)
```

### Via Ingress (Better Distribution)

```
Client 1 → Ingress → Pod 1 (10 connections)
Client 2 → Ingress → Pod 2 (10 connections)
```

Ingress uses cookies for session affinity, so different client processes = different cookies = different pods.

## Verification

### Check Which Pod Has Connections

```bash
# View logs from all pods
kubectl logs -n websocket-app-dev -l app=websocket-server --tail=20

# Check active connections per pod
kubectl get pods -n websocket-app-dev

for pod in $(kubectl get pods -n websocket-app-dev -l app=websocket-server -o name); do
  echo "=== $pod ==="
  kubectl logs -n websocket-app-dev $pod | grep "Active connections:" | tail -1
done
```

### Check Service Endpoints

```bash
# See which pods are behind the service
kubectl get endpoints -n websocket-app-dev dev-websocket-server

# Output shows both pod IPs
# NAME                   ENDPOINTS
# dev-websocket-server   10.1.2.3:8080,10.1.2.4:8080
```

### Verify Session Affinity

```bash
# Check service configuration
kubectl get svc -n websocket-app-dev dev-websocket-server -o yaml | grep -A 5 sessionAffinity

# Should show:
# sessionAffinity: ClientIP
# sessionAffinityConfig:
#   clientIP:
#     timeoutSeconds: 10800
```

## Recommended Testing Strategy

For proper load testing across multiple pods:

### 1. Local Testing (Port-Forward)
```bash
# Test individual pods to verify each can handle 10 connections
kubectl port-forward pod/pod1 8080:8080
# Connect 10 times

kubectl port-forward pod/pod2 8081:8080
# Connect 10 times
```

### 2. Ingress Testing (Production-Like)
```bash
# Run multiple client instances
for i in {1..4}; do
  (cargo run --bin client -- -s ws://websocket-dev.example.com &)
done

# Each client process gets different cookie → different pod
```

### 3. Load Testing Tool
```bash
# Use a proper load testing tool that simulates multiple IPs
# Example with websocat:
for i in {1..20}; do
  websocat ws://websocket-dev.example.com &
done
```

## Summary

| Method | Load Balances? | Production Ready? | Use Case |
|--------|---------------|-------------------|----------|
| `kubectl port-forward svc/...` | ❌ No | ❌ No | Quick debugging one pod |
| `kubectl port-forward pod/...` | ❌ No | ❌ No | Test specific pod |
| Service with session affinity | ⚠️ By client IP | ✅ Yes | Production (via Ingress) |
| Ingress with session affinity | ✅ Yes | ✅ Yes | **Recommended** |

## The Bottom Line

**Your setup is CORRECT!**

The reason you're seeing all connections go to one pod is:
1. Port-forward doesn't load balance
2. Session affinity (correctly) keeps your client's connections on one pod
3. In production via Ingress, different clients will hit different pods

**To test 20 concurrent connections across 2 pods:**

✅ **Option A**: Use Ingress and run 2+ client instances
✅ **Option B**: Port-forward to each pod individually and test separately
✅ **Option C**: Wait for production traffic from multiple real clients

**Don't remove session affinity** - it's required for WebSocket to work properly!

## Next Steps

1. **Deploy Ingress** if not already done
2. **Test via Ingress** with multiple client instances
3. **Monitor in production** with real traffic from different IPs
4. **Accept that port-forward is for debugging**, not load testing

For production load testing, use:
- **k6** with WebSocket support
- **Artillery** with WebSocket engine
- **JMeter** with WebSocket sampler
- **Gatling** with WebSocket DSL

These tools can simulate multiple client IPs and properly test load balancing.
