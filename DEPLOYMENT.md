# Kubernetes Deployment Guide

Complete guide for deploying the WebSocket server to Kubernetes using Kustomize and FluxCD for GitOps.

## Table of Contents

- [Quick Start](#quick-start)
- [Prerequisites](#prerequisites)
- [Architecture](#architecture)
- [Manual Deployment](#manual-deployment)
- [GitOps with FluxCD](#gitops-with-fluxcd)
- [Environment Configuration](#environment-configuration)
- [Monitoring and Troubleshooting](#monitoring-and-troubleshooting)

## Quick Start

### Option 1: Manual Deployment (5 minutes)

```bash
# Deploy to dev environment
./scripts/deploy-manual.sh dev

# Test the deployment
./scripts/test-deployment.sh dev

# Port-forward to test locally
kubectl port-forward -n websocket-app-dev svc/dev-websocket-server 8080:8080
```

### Option 2: GitOps with FluxCD (10 minutes)

```bash
# Set up GitHub token
export GITHUB_TOKEN=<your-github-personal-access-token>

# Bootstrap FluxCD
./scripts/setup-flux.sh

# Watch FluxCD deploy automatically
flux logs --follow
```

## Prerequisites

### Required Tools

1. **kubectl** (v1.25+)
   ```bash
   curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
   chmod +x kubectl
   sudo mv kubectl /usr/local/bin/
   ```

2. **kustomize** (built into kubectl)
   ```bash
   kubectl kustomize --help
   ```

3. **flux** (for GitOps) - Optional
   ```bash
   curl -s https://fluxcd.io/install.sh | sudo bash
   ```

### Kubernetes Cluster

You need access to a Kubernetes cluster. Options:

- **Local**: minikube, kind, k3s, Docker Desktop
- **Cloud**: GKE, EKS, AKS, DigitalOcean
- **On-premise**: Custom cluster

```bash
# Verify cluster access
kubectl cluster-info
kubectl get nodes
```

### GitHub Container Registry Access

Create a Kubernetes secret for pulling images:

```bash
kubectl create secret docker-registry ghcr-secret \
  --docker-server=ghcr.io \
  --docker-username=<github-username> \
  --docker-password=<github-token> \
  --namespace=<target-namespace>
```

## Architecture

### Kustomize Structure

```
k8s/
├── base/                    # Common resources
│   ├── deployment.yaml     # Base deployment
│   ├── service.yaml        # ClusterIP service
│   ├── namespace.yaml      # Namespace
│   └── kustomization.yaml  # Base kustomization
│
└── overlays/               # Environment-specific
    ├── dev/               # Development
    ├── staging/           # Staging
    └── production/        # Production
```

### FluxCD Components

```
flux-system/
├── gitrepository.yaml              # Watches GitHub repo
├── kustomization-dev.yaml          # Dev deployment
├── kustomization-staging.yaml      # Staging deployment
├── kustomization-production.yaml   # Production deployment
└── imagepolicy.yaml                # Auto-update images
```

### Resource Allocation

| Environment | Replicas | CPU Request | Memory Request | CPU Limit | Memory Limit |
|-------------|----------|-------------|----------------|-----------|--------------|
| Dev         | 1        | 50m         | 32Mi           | 200m      | 64Mi         |
| Staging     | 2        | 100m        | 64Mi           | 500m      | 128Mi        |
| Production  | 3        | 200m        | 128Mi          | 1000m     | 256Mi        |

## Manual Deployment

### Step 1: Choose Environment

```bash
# Development
ENVIRONMENT=dev

# Staging
ENVIRONMENT=staging

# Production
ENVIRONMENT=production
```

### Step 2: Create Namespace

```bash
NAMESPACE="websocket-app-${ENVIRONMENT}"
kubectl create namespace ${NAMESPACE}
```

### Step 3: Create Image Pull Secret

```bash
kubectl create secret docker-registry ghcr-secret \
  --docker-server=ghcr.io \
  --docker-username=williamguerzeder \
  --docker-password=${GITHUB_TOKEN} \
  --namespace=${NAMESPACE}
```

### Step 4: Deploy with Kustomize

```bash
# Preview what will be deployed
kubectl kustomize k8s/overlays/${ENVIRONMENT}

# Apply the configuration
kubectl apply -k k8s/overlays/${ENVIRONMENT}
```

### Step 5: Verify Deployment

```bash
# Check all resources
kubectl get all -n ${NAMESPACE}

# Check deployment status
kubectl rollout status deployment/${ENVIRONMENT}-websocket-server -n ${NAMESPACE}

# View logs
kubectl logs -n ${NAMESPACE} -l app=websocket-server --tail=50
```

### Step 6: Test Connection

```bash
# Port-forward
kubectl port-forward -n ${NAMESPACE} svc/${ENVIRONMENT}-websocket-server 8080:8080

# In another terminal, run the client
cargo run --bin client
```

## GitOps with FluxCD

### Step 1: Install FluxCD

```bash
# Install CLI
curl -s https://fluxcd.io/install.sh | sudo bash

# Verify installation
flux --version

# Check cluster prerequisites
flux check --pre
```

### Step 2: Bootstrap FluxCD

```bash
# Set GitHub credentials
export GITHUB_TOKEN=<your-token>
export GITHUB_USER=williamguerzeder
export GITHUB_REPO=websocket-app

# Bootstrap Flux
flux bootstrap github \
  --owner=${GITHUB_USER} \
  --repository=${GITHUB_REPO} \
  --branch=main \
  --path=./flux-system \
  --personal
```

Or use the helper script:

```bash
export GITHUB_TOKEN=<your-token>
./scripts/setup-flux.sh
```

### Step 3: Verify FluxCD Installation

```bash
# Check Flux components
kubectl get pods -n flux-system

# Check Flux status
flux check

# View all Flux resources
flux get all
```

### Step 4: Apply GitOps Configurations

```bash
# GitRepository - monitors GitHub
kubectl apply -f flux-system/gitrepository.yaml

# Kustomizations - deploy environments
kubectl apply -f flux-system/kustomization-dev.yaml
kubectl apply -f flux-system/kustomization-staging.yaml
kubectl apply -f flux-system/kustomization-production.yaml

# Optional: Image automation
kubectl apply -f flux-system/imagepolicy.yaml
```

### Step 5: Monitor Deployments

```bash
# Watch Flux reconciliation
flux logs --follow

# Check GitRepository
flux get sources git

# Check Kustomizations
flux get kustomizations

# View specific environment
kubectl get all -n websocket-app-dev
```

### Step 6: Trigger Manual Reconciliation

```bash
# Reconcile Git source
flux reconcile source git websocket-app

# Reconcile specific environment
flux reconcile kustomization websocket-app-dev
flux reconcile kustomization websocket-app-staging
flux reconcile kustomization websocket-app-production
```

## GitOps Workflow

### How It Works

1. **Developer pushes code** to GitHub
2. **GitHub Actions** builds and pushes Docker image
3. **FluxCD** detects changes in Git repository
4. **FluxCD** applies Kustomize overlays to cluster
5. **Kubernetes** deploys updated application
6. **Optional**: FluxCD detects new image and updates Git

### Update Workflow

#### Update Application Code

```bash
# 1. Make changes to Rust code
vim src/server.rs

# 2. Commit and push
git add .
git commit -m "feat: Add new feature"
git push origin main

# 3. GitHub Actions builds Docker image
# (automatically tagged with main-<sha>)

# 4. FluxCD detects changes and deploys
# (if image automation is enabled)
```

#### Update Configuration

```bash
# 1. Change replica count
vim k8s/overlays/production/replica-patch.yaml

# 2. Commit and push
git add k8s/
git commit -m "scale: Increase production replicas to 5"
git push origin main

# 3. FluxCD automatically applies changes
# No manual kubectl needed!
```

#### Manual Image Update

```bash
# 1. Update image tag
vim k8s/overlays/production/image-patch.yaml

# 2. Change image tag to specific version
# image: ghcr.io/williamguerzeder/websocket-app:v1.2.3

# 3. Commit and push
git add k8s/
git commit -m "deploy: Update production to v1.2.3"
git push origin main

# 4. FluxCD applies the change
```

## Environment Configuration

### Development

**Purpose**: Active development and testing

```yaml
# k8s/overlays/dev/kustomization.yaml
namespace: websocket-app-dev
namePrefix: dev-
replicas: 1
image: ghcr.io/williamguerzeder/websocket-app:develop
env:
  RUST_LOG: debug
```

### Staging

**Purpose**: Pre-production testing

```yaml
# k8s/overlays/staging/kustomization.yaml
namespace: websocket-app-staging
namePrefix: staging-
replicas: 2
image: ghcr.io/williamguerzeder/websocket-app:staging
env:
  RUST_LOG: info
```

### Production

**Purpose**: Live production environment

```yaml
# k8s/overlays/production/kustomization.yaml
namespace: websocket-app-production
namePrefix: prod-
replicas: 3
image: ghcr.io/williamguerzeder/websocket-app:latest
env:
  RUST_LOG: info
```

## Monitoring and Troubleshooting

### Check Pod Status

```bash
kubectl get pods -n websocket-app-production

# Describe pod for events
kubectl describe pod <pod-name> -n websocket-app-production

# View logs
kubectl logs <pod-name> -n websocket-app-production -f
```

### Check Deployment Health

```bash
# Deployment status
kubectl get deployment -n websocket-app-production

# Rollout status
kubectl rollout status deployment/prod-websocket-server -n websocket-app-production

# Rollout history
kubectl rollout history deployment/prod-websocket-server -n websocket-app-production
```

### Common Issues

#### 1. ImagePullBackOff

**Cause**: Cannot pull Docker image from GHCR

**Solution**:
```bash
# Create/update image pull secret
kubectl create secret docker-registry ghcr-secret \
  --docker-server=ghcr.io \
  --docker-username=williamguerzeder \
  --docker-password=${GITHUB_TOKEN} \
  --namespace=websocket-app-production \
  --dry-run=client -o yaml | kubectl apply -f -
```

#### 2. CrashLoopBackOff

**Cause**: Application crashes on startup

**Solution**:
```bash
# Check logs
kubectl logs <pod-name> -n websocket-app-production

# Check events
kubectl get events -n websocket-app-production --sort-by='.lastTimestamp'
```

#### 3. FluxCD Not Reconciling

**Cause**: FluxCD not detecting changes

**Solution**:
```bash
# Check Flux status
flux get all

# Force reconcile
flux reconcile source git websocket-app --with-source
flux reconcile kustomization websocket-app-production

# Check Flux logs
kubectl logs -n flux-system deploy/source-controller
kubectl logs -n flux-system deploy/kustomize-controller
```

#### 4. Service Not Accessible

**Cause**: Service or endpoints misconfigured

**Solution**:
```bash
# Check service
kubectl get svc -n websocket-app-production

# Check endpoints
kubectl get endpoints -n websocket-app-production

# Verify pod labels match service selector
kubectl get pods -n websocket-app-production --show-labels
```

### Useful Commands

```bash
# Get all resources in namespace
kubectl get all -n websocket-app-production

# Watch resources in real-time
kubectl get pods -n websocket-app-production -w

# Execute command in pod
kubectl exec -it <pod-name> -n websocket-app-production -- /bin/bash

# Port-forward for local testing
kubectl port-forward -n websocket-app-production svc/prod-websocket-server 8080:8080

# Scale deployment
kubectl scale deployment prod-websocket-server -n websocket-app-production --replicas=5

# Restart deployment
kubectl rollout restart deployment/prod-websocket-server -n websocket-app-production
```

## Next Steps

1. **Add Ingress**: Expose service externally
   - Install ingress controller (nginx, traefik)
   - Create Ingress resource
   - Configure TLS certificates

2. **Add Monitoring**: Prometheus and Grafana
   - Deploy Prometheus Operator
   - Add ServiceMonitor
   - Create Grafana dashboards

3. **Add Horizontal Pod Autoscaler**: Auto-scaling
   - Install metrics-server
   - Create HPA resource
   - Configure scaling policies

4. **Add Network Policies**: Security
   - Restrict pod-to-pod traffic
   - Allow only necessary connections

5. **Add PodDisruptionBudget**: High availability
   - Ensure minimum availability during updates
   - Configure disruption policies

## Resources

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Kustomize Documentation](https://kustomize.io/)
- [FluxCD Documentation](https://fluxcd.io/docs/)
- [GitHub Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
