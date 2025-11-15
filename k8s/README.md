# Kubernetes Deployment with Kustomize and FluxCD

This directory contains Kubernetes manifests and Kustomize overlays for deploying the WebSocket server to Kubernetes clusters using GitOps with FluxCD.

## Directory Structure

```
k8s/
├── base/                           # Base Kubernetes resources
│   ├── deployment.yaml            # Base deployment configuration
│   ├── service.yaml               # ClusterIP service
│   ├── namespace.yaml             # Namespace definition
│   └── kustomization.yaml         # Base kustomization
│
├── overlays/                      # Environment-specific overlays
│   ├── dev/                       # Development environment
│   │   ├── kustomization.yaml    # Dev kustomization
│   │   ├── replica-patch.yaml    # 1 replica for dev
│   │   ├── resource-patch.yaml   # Minimal resources
│   │   └── image-patch.yaml      # develop tag
│   │
│   ├── staging/                   # Staging environment
│   │   ├── kustomization.yaml    # Staging kustomization
│   │   ├── replica-patch.yaml    # 2 replicas
│   │   └── image-patch.yaml      # staging tag
│   │
│   └── production/                # Production environment
│       ├── kustomization.yaml    # Production kustomization
│       ├── replica-patch.yaml    # 3 replicas
│       ├── resource-patch.yaml   # Production resources
│       └── image-patch.yaml      # latest tag
│
└── README.md                      # This file
```

## Environment Configuration

### Development
- **Namespace**: `websocket-app-dev`
- **Replicas**: 1
- **Resources**: Minimal (32Mi RAM, 50m CPU)
- **Image Tag**: `develop`
- **Log Level**: `debug`

### Staging
- **Namespace**: `websocket-app-staging`
- **Replicas**: 2
- **Resources**: Standard (64Mi RAM, 100m CPU)
- **Image Tag**: `staging`
- **Log Level**: `info`

### Production
- **Namespace**: `websocket-app-production`
- **Replicas**: 3
- **Resources**: High (128Mi RAM, 200m CPU)
- **Image Tag**: `latest`
- **Log Level**: `info`

## Manual Deployment (Without FluxCD)

### Prerequisites

1. **kubectl** installed and configured
2. **kustomize** installed (or use `kubectl apply -k`)
3. Access to Kubernetes cluster
4. GitHub Container Registry authentication

### Deploy to Development

```bash
# Create namespace
kubectl create namespace websocket-app-dev

# Create image pull secret for GHCR
kubectl create secret docker-registry ghcr-secret \
  --docker-server=ghcr.io \
  --docker-username=<your-github-username> \
  --docker-password=<your-github-token> \
  --namespace=websocket-app-dev

# Deploy using kustomize
kubectl apply -k k8s/overlays/dev

# Verify deployment
kubectl get all -n websocket-app-dev
```

### Deploy to Staging

```bash
kubectl create namespace websocket-app-staging
kubectl create secret docker-registry ghcr-secret \
  --docker-server=ghcr.io \
  --docker-username=<your-github-username> \
  --docker-password=<your-github-token> \
  --namespace=websocket-app-staging

kubectl apply -k k8s/overlays/staging
kubectl get all -n websocket-app-staging
```

### Deploy to Production

```bash
kubectl create namespace websocket-app-production
kubectl create secret docker-registry ghcr-secret \
  --docker-server=ghcr.io \
  --docker-username=<your-github-username> \
  --docker-password=<your-github-token> \
  --namespace=websocket-app-production

kubectl apply -k k8s/overlays/production
kubectl get all -n websocket-app-production
```

## GitOps Deployment with FluxCD

### Prerequisites

1. FluxCD installed on your cluster
2. GitHub repository access configured
3. FluxCD has read access to your container registry

### Install FluxCD

```bash
# Install Flux CLI
curl -s https://fluxcd.io/install.sh | sudo bash

# Bootstrap Flux on your cluster
flux bootstrap github \
  --owner=williamguerzeder \
  --repository=websocket-app \
  --branch=main \
  --path=./flux-system \
  --personal
```

### Deploy with FluxCD

FluxCD configurations are in the `flux-system/` directory:

1. **GitRepository** - Monitors this GitHub repository
2. **Kustomizations** - Applies manifests from each environment
3. **ImagePolicy** - Watches for new container images
4. **ImageUpdateAutomation** - Auto-updates image tags in Git

```bash
# Apply FluxCD configurations
kubectl apply -f flux-system/gitrepository.yaml
kubectl apply -f flux-system/kustomization-dev.yaml
kubectl apply -f flux-system/kustomization-staging.yaml
kubectl apply -f flux-system/kustomization-production.yaml

# Optional: Enable automatic image updates
kubectl apply -f flux-system/imagepolicy.yaml
```

### Verify FluxCD Deployment

```bash
# Check GitRepository status
flux get sources git

# Check Kustomization status
flux get kustomizations

# Watch reconciliation
flux logs --follow

# Check specific environment
kubectl get all -n websocket-app-dev
kubectl get all -n websocket-app-staging
kubectl get all -n websocket-app-production
```

## Accessing the Service

### Port Forward (Local Testing)

```bash
# Development
kubectl port-forward -n websocket-app-dev svc/dev-websocket-server 8080:8080

# Staging
kubectl port-forward -n websocket-app-staging svc/staging-websocket-server 8080:8080

# Production
kubectl port-forward -n websocket-app-production svc/prod-websocket-server 8080:8080
```

Then connect with the client:
```bash
cargo run --bin client
> connect
```

### Ingress (Production)

Create an Ingress resource to expose the service:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: websocket-ingress
  namespace: websocket-app-production
  annotations:
    nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "3600"
spec:
  rules:
  - host: ws.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: prod-websocket-server
            port:
              number: 8080
```

## Updating Deployments

### Manual Update

```bash
# Update image tag in overlay
vim k8s/overlays/production/image-patch.yaml

# Commit and push
git add k8s/
git commit -m "Update production image to v1.2.3"
git push

# FluxCD will automatically detect and apply changes
```

### Automatic Image Updates

If `imagepolicy.yaml` is applied, FluxCD will:
1. Monitor GHCR for new images
2. Update image tags in Git automatically
3. Apply the changes to the cluster

## Monitoring

### Check Deployment Status

```bash
kubectl get deployments -n websocket-app-production
kubectl get pods -n websocket-app-production
kubectl get svc -n websocket-app-production
```

### View Logs

```bash
# All pods
kubectl logs -n websocket-app-production -l app=websocket-server --tail=100 -f

# Specific pod
kubectl logs -n websocket-app-production <pod-name> -f
```

### Describe Resources

```bash
kubectl describe deployment prod-websocket-server -n websocket-app-production
kubectl describe pod <pod-name> -n websocket-app-production
```

## Scaling

### Manual Scaling

```bash
kubectl scale deployment prod-websocket-server -n websocket-app-production --replicas=5
```

### Update Overlay

```bash
# Edit replica-patch.yaml
vim k8s/overlays/production/replica-patch.yaml

# Change replicas: 5
# Commit and push - FluxCD will apply
```

## Rollback

### Using kubectl

```bash
kubectl rollout undo deployment/prod-websocket-server -n websocket-app-production
kubectl rollout history deployment/prod-websocket-server -n websocket-app-production
```

### Using Git

```bash
git revert <commit-hash>
git push
# FluxCD will apply the revert
```

## Troubleshooting

### Pods Not Starting

```bash
kubectl describe pod <pod-name> -n websocket-app-production
kubectl logs <pod-name> -n websocket-app-production
```

Common issues:
- Image pull errors: Check GHCR credentials
- Resource limits: Check node capacity
- Health checks failing: Check port 8080 is accessible

### FluxCD Not Reconciling

```bash
# Check Flux status
flux get all

# Force reconciliation
flux reconcile source git websocket-app
flux reconcile kustomization websocket-app-production

# Check Flux logs
kubectl logs -n flux-system deploy/source-controller
kubectl logs -n flux-system deploy/kustomize-controller
```

### Service Not Accessible

```bash
# Check service endpoints
kubectl get endpoints -n websocket-app-production

# Check pod labels match service selector
kubectl get pods -n websocket-app-production --show-labels
```

## Security Considerations

- ✅ Runs as non-root user (UID 1000)
- ✅ Read-only root filesystem
- ✅ Drops all capabilities
- ✅ Resource limits enforced
- ✅ Network policies (can be added)
- ✅ Pod security standards compliant

## Next Steps

1. **Add Monitoring**: Integrate Prometheus metrics
2. **Add Ingress**: Expose service externally
3. **Add HPA**: Horizontal Pod Autoscaling
4. **Add Network Policies**: Restrict traffic
5. **Add PodDisruptionBudget**: Ensure availability during updates
6. **Add RBAC**: Fine-grained access control

## Resources

- [Kustomize Documentation](https://kustomize.io/)
- [FluxCD Documentation](https://fluxcd.io/docs/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
