# Ingress Setup Guide

Complete guide for exposing the WebSocket server externally using Kubernetes Ingress with support for WebSocket connections.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Configuration](#configuration)
- [DNS Setup](#dns-setup)
- [TLS/SSL Setup](#tlsssl-setup)
- [Deployment](#deployment)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)

## Overview

The Ingress configuration provides external access to the WebSocket server with:

- ✅ **WebSocket support** with proper upgrade headers
- ✅ **Session affinity** for persistent connections
- ✅ **Long-lived connection timeouts** (up to 2 hours)
- ✅ **TLS/SSL support** for secure connections
- ✅ **Environment-specific hostnames**
- ✅ **Production-ready security settings**

## Prerequisites

### 1. Ingress Controller

You need an Ingress controller installed in your cluster. We recommend **NGINX Ingress Controller**.

**Install NGINX Ingress Controller:**

```bash
# Using Helm
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm repo update

helm install ingress-nginx ingress-nginx/ingress-nginx \
  --namespace ingress-nginx \
  --create-namespace \
  --set controller.service.type=LoadBalancer

# Verify installation
kubectl get pods -n ingress-nginx
kubectl get svc -n ingress-nginx
```

**Alternative: Using kubectl**

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.9.5/deploy/static/provider/cloud/deploy.yaml
```

### 2. External IP or Load Balancer

Get the external IP of your Ingress controller:

```bash
kubectl get svc -n ingress-nginx ingress-nginx-controller

# Output example:
# NAME                       TYPE           EXTERNAL-IP     PORT(S)
# ingress-nginx-controller   LoadBalancer   35.123.45.67    80:30080/TCP,443:30443/TCP
```

## Configuration

### Environment-Specific Hostnames

| Environment | Hostname | TLS | Timeout |
|-------------|----------|-----|---------|
| Development | `websocket-dev.example.com` | Optional | 30 min |
| Staging | `websocket-staging.example.com` | Optional | 1 hour |
| Production | `websocket.example.com` | Required | 2 hours |

### WebSocket Annotations

The Ingress includes these NGINX-specific annotations:

```yaml
# Long-lived connection timeouts
nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"    # 1 hour
nginx.ingress.kubernetes.io/proxy-send-timeout: "3600"    # 1 hour
nginx.ingress.kubernetes.io/proxy-connect-timeout: "60"   # 1 minute

# WebSocket upgrade support
nginx.ingress.kubernetes.io/websocket-services: "websocket-server"
nginx.ingress.kubernetes.io/configuration-snippet: |
  proxy_set_header Upgrade $http_upgrade;
  proxy_set_header Connection "upgrade";

# Session affinity for WebSocket persistence
nginx.ingress.kubernetes.io/affinity: "cookie"
nginx.ingress.kubernetes.io/affinity-mode: "persistent"
nginx.ingress.kubernetes.io/session-cookie-name: "websocket-route"
```

## DNS Setup

### 1. Update DNS Records

Point your domain to the Ingress controller's external IP:

```bash
# Get external IP
EXTERNAL_IP=$(kubectl get svc -n ingress-nginx ingress-nginx-controller -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
echo $EXTERNAL_IP
```

**Create DNS A Records:**

| Record | Type | Value |
|--------|------|-------|
| `websocket-dev.example.com` | A | `<EXTERNAL_IP>` |
| `websocket-staging.example.com` | A | `<EXTERNAL_IP>` |
| `websocket.example.com` | A | `<EXTERNAL_IP>` |

### 2. Update Ingress Hostnames

Edit the ingress patches to use your actual domain:

**Development** (`k8s/overlays/dev/ingress-patch.yaml`):
```yaml
spec:
  rules:
  - host: websocket-dev.yourdomain.com  # ← Update this
```

**Staging** (`k8s/overlays/staging/ingress-patch.yaml`):
```yaml
spec:
  rules:
  - host: websocket-staging.yourdomain.com  # ← Update this
```

**Production** (`k8s/overlays/production/ingress-patch.yaml`):
```yaml
spec:
  rules:
  - host: websocket.yourdomain.com  # ← Update this
  tls:
  - hosts:
    - websocket.yourdomain.com  # ← Update this
```

## TLS/SSL Setup

### Option 1: Cert-Manager (Recommended)

**Install Cert-Manager:**

```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.3/cert-manager.yaml

# Verify installation
kubectl get pods -n cert-manager
```

**Create ClusterIssuer for Let's Encrypt:**

```yaml
# cert-manager-letsencrypt.yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com  # ← Update this
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
```

Apply:
```bash
kubectl apply -f cert-manager-letsencrypt.yaml
```

**Update Production Ingress for Automatic Certificates:**

Add annotation to `k8s/overlays/production/ingress-patch.yaml`:

```yaml
metadata:
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - websocket.yourdomain.com
    secretName: websocket-tls-production  # Cert-manager will create this
```

### Option 2: Manual Certificate

**Create TLS secret from certificate files:**

```bash
kubectl create secret tls websocket-tls-production \
  --cert=/path/to/tls.crt \
  --key=/path/to/tls.key \
  --namespace=websocket-app-production
```

### Option 3: Self-Signed Certificate (Dev/Testing Only)

```bash
# Generate self-signed certificate
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout tls.key \
  -out tls.crt \
  -subj "/CN=websocket-dev.example.com"

# Create secret
kubectl create secret tls websocket-tls-dev \
  --cert=tls.crt \
  --key=tls.key \
  --namespace=websocket-app-dev
```

## Deployment

### Deploy Ingress Resources

```bash
# Development
kubectl apply -k k8s/overlays/dev

# Staging
kubectl apply -k k8s/overlays/staging

# Production
kubectl apply -k k8s/overlays/production
```

### Verify Ingress

```bash
# Check Ingress resources
kubectl get ingress -n websocket-app-dev
kubectl get ingress -n websocket-app-staging
kubectl get ingress -n websocket-app-production

# Describe Ingress (check for errors)
kubectl describe ingress dev-websocket-server -n websocket-app-dev
```

Expected output:
```
Name:             dev-websocket-server
Namespace:        websocket-app-dev
Address:          35.123.45.67
Default backend:  default-http-backend:80 (<error: endpoints "default-http-backend" not found>)
Rules:
  Host                        Path  Backends
  ----                        ----  --------
  websocket-dev.example.com
                              /     dev-websocket-server:8080 (10.1.2.3:8080,10.1.2.4:8080)
```

## Testing

### 1. Test HTTP/HTTPS Access

```bash
# Without TLS (development)
curl -I http://websocket-dev.example.com

# With TLS (production)
curl -I https://websocket.example.com
```

### 2. Test WebSocket Connection

**Using websocat:**

```bash
# Install websocat
# macOS: brew install websocat
# Linux: cargo install websocat

# Connect to WebSocket
websocat ws://websocket-dev.example.com

# Or with TLS
websocat wss://websocket.example.com
```

**Using the client:**

Update client to connect to Ingress:

```rust
// src/client.rs
const SERVER_URL: &str = "ws://websocket-dev.example.com";
// or
const SERVER_URL: &str = "wss://websocket.example.com";
```

Run:
```bash
cargo run --bin client
> connect
```

### 3. Test Session Affinity

```bash
# Make multiple connections - should route to same pod due to session affinity
for i in {1..5}; do
  curl -I http://websocket-dev.example.com -c cookies.txt -b cookies.txt
done
```

### 4. Test Long-Lived Connections

```bash
# Connect and keep connection open
websocat ws://websocket-dev.example.com

# Send messages periodically to test timeout
# Connection should stay open for configured timeout period (30-120 minutes)
```

## Monitoring

### Check Ingress Logs

```bash
# Get Ingress controller pod name
INGRESS_POD=$(kubectl get pods -n ingress-nginx -l app.kubernetes.io/component=controller -o jsonpath='{.items[0].metadata.name}')

# View logs
kubectl logs -n ingress-nginx $INGRESS_POD -f

# Filter for your service
kubectl logs -n ingress-nginx $INGRESS_POD -f | grep websocket
```

### Check Backend Endpoints

```bash
# Verify Ingress is routing to correct pods
kubectl get endpoints -n websocket-app-dev

# Should show pod IPs
NAME                    ENDPOINTS
dev-websocket-server    10.1.2.3:8080,10.1.2.4:8080
```

## Troubleshooting

### Issue 1: 502 Bad Gateway

**Symptoms**: Ingress returns 502 error

**Causes & Solutions**:

1. **Backend pods not ready**
   ```bash
   kubectl get pods -n websocket-app-dev
   # Ensure pods are Running and Ready (1/1)
   ```

2. **Service has no endpoints**
   ```bash
   kubectl get endpoints -n websocket-app-dev
   # Should list pod IPs
   ```

3. **Probe failures**
   ```bash
   kubectl describe pod <pod-name> -n websocket-app-dev
   # Check for probe failures in Events
   ```

### Issue 2: WebSocket Connection Fails

**Symptoms**: Connection closes immediately or upgrade fails

**Solutions**:

1. **Check WebSocket annotations**
   ```bash
   kubectl get ingress dev-websocket-server -n websocket-app-dev -o yaml | grep websocket
   ```

2. **Verify upgrade headers in logs**
   ```bash
   kubectl logs -n ingress-nginx $INGRESS_POD | grep -i upgrade
   ```

3. **Test direct pod connection**
   ```bash
   kubectl port-forward -n websocket-app-dev <pod-name> 8080:8080
   websocat ws://localhost:8080
   # If this works, issue is with Ingress configuration
   ```

### Issue 3: Connection Timeout Too Short

**Symptoms**: WebSocket disconnects after short period

**Solution**: Increase timeout in ingress-patch.yaml:

```yaml
metadata:
  annotations:
    nginx.ingress.kubernetes.io/proxy-read-timeout: "7200"  # 2 hours
    nginx.ingress.kubernetes.io/proxy-send-timeout: "7200"
```

### Issue 4: TLS Certificate Not Working

**Symptoms**: HTTPS not working or certificate errors

**Solutions**:

1. **Check cert-manager status**
   ```bash
   kubectl get certificate -n websocket-app-production
   kubectl describe certificate websocket-tls-production -n websocket-app-production
   ```

2. **Check certificate secret**
   ```bash
   kubectl get secret websocket-tls-production -n websocket-app-production
   ```

3. **Check cert-manager logs**
   ```bash
   kubectl logs -n cert-manager deploy/cert-manager
   ```

### Issue 5: Session Affinity Not Working

**Symptoms**: Requests routing to different pods

**Solution**: Check cookie in browser/client:

```bash
curl -I http://websocket-dev.example.com -c cookies.txt
cat cookies.txt
# Should contain: websocket-route cookie
```

## Advanced Configuration

### Enable CORS

Add to ingress annotations:

```yaml
metadata:
  annotations:
    nginx.ingress.kubernetes.io/enable-cors: "true"
    nginx.ingress.kubernetes.io/cors-allow-origin: "https://yourfrontend.com"
    nginx.ingress.kubernetes.io/cors-allow-methods: "GET, POST, OPTIONS"
    nginx.ingress.kubernetes.io/cors-allow-headers: "Authorization, Content-Type"
```

### Rate Limiting

Add to production ingress:

```yaml
metadata:
  annotations:
    nginx.ingress.kubernetes.io/limit-connections: "100"
    nginx.ingress.kubernetes.io/limit-rps: "50"
    nginx.ingress.kubernetes.io/limit-whitelist: "10.0.0.0/8"
```

### Custom Error Pages

```yaml
metadata:
  annotations:
    nginx.ingress.kubernetes.io/custom-http-errors: "404,503"
    nginx.ingress.kubernetes.io/default-backend: custom-error-pages
```

### IP Whitelisting

```yaml
metadata:
  annotations:
    nginx.ingress.kubernetes.io/whitelist-source-range: "10.0.0.0/8,192.168.0.0/16"
```

## Alternative Ingress Controllers

While this guide focuses on NGINX, the WebSocket server also works with:

### Traefik

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  annotations:
    traefik.ingress.kubernetes.io/router.entrypoints: websecure
    traefik.ingress.kubernetes.io/router.tls: "true"
```

### HAProxy

```yaml
metadata:
  annotations:
    haproxy.org/timeout-tunnel: "3600s"
    haproxy.org/backend-config-snippet: |
      option http-server-close
      option forwardfor
```

### AWS ALB

```yaml
metadata:
  annotations:
    alb.ingress.kubernetes.io/scheme: internet-facing
    alb.ingress.kubernetes.io/target-type: ip
    alb.ingress.kubernetes.io/healthcheck-path: /
```

## Summary

✅ **Ingress created** for all three environments
✅ **WebSocket support** with proper headers and timeouts
✅ **Session affinity** for connection persistence
✅ **TLS/SSL ready** with cert-manager support
✅ **Production hardened** with security annotations

## Next Steps

1. Update DNS records with your domain
2. Install cert-manager for automatic TLS
3. Update ingress-patch.yaml files with your hostnames
4. Deploy and test connectivity
5. Monitor Ingress logs for issues
6. Consider adding WAF/DDoS protection
