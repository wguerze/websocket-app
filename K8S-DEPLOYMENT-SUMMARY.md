# Kubernetes & GitOps Deployment Summary

Complete Kubernetes deployment setup with Kustomize and FluxCD for GitOps automation.

## 🎯 What Was Created

### 1. Kubernetes Manifests (`k8s/`)

#### Base Resources (`k8s/base/`)
```
k8s/base/
├── deployment.yaml      # Base deployment with 2 replicas, health checks
├── service.yaml         # ClusterIP service with session affinity
├── namespace.yaml       # Namespace definition
└── kustomization.yaml   # Base kustomization file
```

**Features**:
- ✅ Non-root container (UID 1000)
- ✅ Read-only root filesystem
- ✅ Resource limits and requests
- ✅ Liveness and readiness probes
- ✅ Security context with dropped capabilities
- ✅ Session affinity for WebSocket connections

#### Environment Overlays (`k8s/overlays/`)

**Development** (`k8s/overlays/dev/`)
- Namespace: `websocket-app-dev`
- Replicas: 1
- Resources: Minimal (32Mi RAM, 50m CPU)
- Image: `ghcr.io/williamguerzeder/websocket-app:develop`
- Log Level: `debug`

**Staging** (`k8s/overlays/staging/`)
- Namespace: `websocket-app-staging`
- Replicas: 2
- Resources: Standard (64Mi RAM, 100m CPU)
- Image: `ghcr.io/williamguerzeder/websocket-app:staging`
- Log Level: `info`

**Production** (`k8s/overlays/production/`)
- Namespace: `websocket-app-production`
- Replicas: 3
- Resources: High (128Mi RAM, 200m CPU)
- Image: `ghcr.io/williamguerzeder/websocket-app:latest`
- Log Level: `info`

### 2. FluxCD GitOps Configuration (`flux-system/`)

```
flux-system/
├── gitrepository.yaml              # Watches GitHub repo
├── kustomization-dev.yaml          # Dev deployment (5min interval)
├── kustomization-staging.yaml      # Staging deployment (10min interval)
├── kustomization-production.yaml   # Production deployment (30min interval)
└── imagepolicy.yaml                # Automatic image updates
```

**Features**:
- ✅ Automatic Git synchronization
- ✅ Health checks for deployments
- ✅ Automatic rollbacks on failure
- ✅ Image automation (optional)
- ✅ Different reconciliation intervals per environment

### 3. Helper Scripts (`scripts/`)

```bash
scripts/
├── deploy-manual.sh      # Manual deployment to any environment
├── setup-flux.sh         # Bootstrap FluxCD
└── test-deployment.sh    # Test deployed application
```

### 4. Documentation

- **DEPLOYMENT.md** - Complete deployment guide
- **k8s/README.md** - Kubernetes manifests documentation
- **README.md** - Updated with Kubernetes section

## 🚀 Quick Start

### Option 1: Manual Deployment

```bash
# Deploy to development
./scripts/deploy-manual.sh dev

# Test deployment
./scripts/test-deployment.sh dev

# Access service
kubectl port-forward -n websocket-app-dev svc/dev-websocket-server 8080:8080
```

### Option 2: GitOps with FluxCD

```bash
# Set GitHub token
export GITHUB_TOKEN=<your-github-pat>

# Bootstrap FluxCD
./scripts/setup-flux.sh

# Watch deployments
flux logs --follow
```

## 📊 Architecture

### GitOps Workflow

```
┌──────────────┐
│  Developer   │
│  Push Code   │
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│  GitHub Actions  │
│  Build & Test    │
│  Push Image      │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  GitHub Registry │
│  ghcr.io         │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│     FluxCD       │
│  Detect Changes  │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│   Kubernetes     │
│  Deploy Pods     │
└──────────────────┘
```

### Deployment Flow

1. **Code Change** → Push to GitHub
2. **CI/CD** → GitHub Actions builds Docker image
3. **Registry** → Image pushed to ghcr.io
4. **GitOps** → FluxCD detects change in Git
5. **Deploy** → Kustomize applies manifests
6. **Kubernetes** → Pods updated with new image

## 🔧 Configuration

### Environment Variables

Set in overlays via `configMapGenerator`:

```yaml
- RUST_LOG=info        # Log level
- ENVIRONMENT=production  # Environment name
```

### Resource Allocation

| Environment | CPU Request | CPU Limit | Memory Request | Memory Limit |
|-------------|-------------|-----------|----------------|--------------|
| Dev         | 50m         | 200m      | 32Mi           | 64Mi         |
| Staging     | 100m        | 500m      | 64Mi           | 128Mi        |
| Production  | 200m        | 1000m     | 128Mi          | 256Mi        |

### Reconciliation Intervals

| Environment | Interval | Health Check Timeout |
|-------------|----------|----------------------|
| Dev         | 5 min    | 2 min                |
| Staging     | 10 min   | 2 min                |
| Production  | 30 min   | 5 min                |

## 📦 Deployment Commands

### Manual Deployment

```bash
# Deploy specific environment
kubectl apply -k k8s/overlays/dev
kubectl apply -k k8s/overlays/staging
kubectl apply -k k8s/overlays/production

# Preview changes
kubectl kustomize k8s/overlays/production

# Check status
kubectl get all -n websocket-app-production
```

### FluxCD Operations

```bash
# Check status
flux get all
flux get sources git
flux get kustomizations

# Force reconciliation
flux reconcile source git websocket-app
flux reconcile kustomization websocket-app-production

# Suspend/Resume
flux suspend kustomization websocket-app-production
flux resume kustomization websocket-app-production

# Watch logs
flux logs --follow
```

### Kubernetes Operations

```bash
# View pods
kubectl get pods -n websocket-app-production

# View logs
kubectl logs -n websocket-app-production -l app=websocket-server -f

# Scale deployment
kubectl scale deployment prod-websocket-server -n websocket-app-production --replicas=5

# Restart deployment
kubectl rollout restart deployment/prod-websocket-server -n websocket-app-production

# Rollback
kubectl rollout undo deployment/prod-websocket-server -n websocket-app-production
```

## 🔍 Monitoring

### Health Checks

**Liveness Probe**:
- Type: TCP Socket on port 8080
- Initial Delay: 10s
- Period: 10s
- Timeout: 5s
- Failure Threshold: 3

**Readiness Probe**:
- Type: TCP Socket on port 8080
- Initial Delay: 5s
- Period: 5s
- Timeout: 3s
- Failure Threshold: 2

### Viewing Logs

```bash
# All pods in environment
kubectl logs -n websocket-app-production -l app=websocket-server --tail=100 -f

# Specific pod
kubectl logs -n websocket-app-production <pod-name> -f

# Previous container (if crashed)
kubectl logs -n websocket-app-production <pod-name> --previous
```

### Events

```bash
# Recent events
kubectl get events -n websocket-app-production --sort-by='.lastTimestamp'

# Describe deployment
kubectl describe deployment prod-websocket-server -n websocket-app-production

# Describe pod
kubectl describe pod <pod-name> -n websocket-app-production
```

## 🔄 Update Workflows

### Update Application Code

```bash
# 1. Make code changes
vim src/server.rs

# 2. Commit and push
git add .
git commit -m "feat: Add new feature"
git push origin main

# 3. GitHub Actions builds image
# (automatic)

# 4. Update Kustomize overlay
vim k8s/overlays/production/image-patch.yaml
# Change image tag to new SHA

# 5. Commit and push
git add k8s/
git commit -m "deploy: Update production image"
git push origin main

# 6. FluxCD deploys automatically
# (if image automation is enabled, steps 4-5 are automatic)
```

### Update Configuration

```bash
# 1. Change replica count
vim k8s/overlays/production/replica-patch.yaml

# 2. Commit and push
git add k8s/
git commit -m "scale: Increase replicas to 5"
git push origin main

# 3. FluxCD applies changes
# (automatic within reconciliation interval)
```

### Emergency Rollback

```bash
# Option 1: kubectl
kubectl rollout undo deployment/prod-websocket-server -n websocket-app-production

# Option 2: Git revert
git revert <bad-commit-hash>
git push origin main
# FluxCD will apply the revert
```

## 🛠️ Troubleshooting

### Common Issues

**1. ImagePullBackOff**
```bash
# Create image pull secret
kubectl create secret docker-registry ghcr-secret \
  --docker-server=ghcr.io \
  --docker-username=williamguerzeder \
  --docker-password=${GITHUB_TOKEN} \
  --namespace=websocket-app-production
```

**2. CrashLoopBackOff**
```bash
# Check logs
kubectl logs <pod-name> -n websocket-app-production

# Check events
kubectl describe pod <pod-name> -n websocket-app-production
```

**3. FluxCD Not Reconciling**
```bash
# Check Flux status
flux get all

# Check specific resource
flux get source git websocket-app
flux get kustomization websocket-app-production

# Force reconcile
flux reconcile source git websocket-app --with-source
flux reconcile kustomization websocket-app-production
```

**4. Service Not Accessible**
```bash
# Check service
kubectl get svc -n websocket-app-production

# Check endpoints
kubectl get endpoints -n websocket-app-production

# Check pod labels
kubectl get pods -n websocket-app-production --show-labels
```

## 📝 File Structure

```
websocket-app/
├── k8s/
│   ├── base/
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   ├── namespace.yaml
│   │   └── kustomization.yaml
│   ├── overlays/
│   │   ├── dev/
│   │   │   ├── kustomization.yaml
│   │   │   ├── replica-patch.yaml
│   │   │   ├── resource-patch.yaml
│   │   │   └── image-patch.yaml
│   │   ├── staging/
│   │   │   ├── kustomization.yaml
│   │   │   ├── replica-patch.yaml
│   │   │   └── image-patch.yaml
│   │   └── production/
│   │       ├── kustomization.yaml
│   │       ├── replica-patch.yaml
│   │       ├── resource-patch.yaml
│   │       └── image-patch.yaml
│   └── README.md
├── flux-system/
│   ├── gitrepository.yaml
│   ├── kustomization-dev.yaml
│   ├── kustomization-staging.yaml
│   ├── kustomization-production.yaml
│   └── imagepolicy.yaml
├── scripts/
│   ├── deploy-manual.sh
│   ├── setup-flux.sh
│   └── test-deployment.sh
├── DEPLOYMENT.md
└── K8S-DEPLOYMENT-SUMMARY.md (this file)
```

## ✅ Summary

You now have:

- ✅ **Kubernetes manifests** with Kustomize overlays for 3 environments
- ✅ **FluxCD GitOps** configuration for automated deployments
- ✅ **Helper scripts** for manual deployment and testing
- ✅ **Complete documentation** for deployment workflows
- ✅ **Security hardening** (non-root, read-only filesystem, resource limits)
- ✅ **Health checks** (liveness and readiness probes)
- ✅ **Session affinity** for WebSocket connections
- ✅ **Automatic image updates** (optional)

## 🚀 Next Steps

1. **Deploy to your cluster**: Run `./scripts/deploy-manual.sh dev`
2. **Set up FluxCD**: Run `./scripts/setup-flux.sh`
3. **Add monitoring**: Integrate Prometheus and Grafana
4. **Add Ingress**: Expose service externally with TLS
5. **Add HPA**: Horizontal Pod Autoscaling based on metrics
6. **Add Network Policies**: Restrict pod-to-pod communication

Your WebSocket server is now production-ready with GitOps! 🎉
