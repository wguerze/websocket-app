#!/bin/bash
# Manual deployment script for Kubernetes

set -e

ENVIRONMENT=${1:-dev}
NAMESPACE="websocket-app-${ENVIRONMENT}"

echo "🚀 Deploying WebSocket Server to ${ENVIRONMENT} environment"
echo "Namespace: ${NAMESPACE}"
echo ""

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed. Please install it first."
    exit 1
fi

# Check if kustomize is available
if ! kubectl kustomize --help &> /dev/null; then
    echo "❌ kustomize is not available in kubectl. Please upgrade kubectl."
    exit 1
fi

# Create namespace if it doesn't exist
echo "📦 Creating namespace ${NAMESPACE}..."
kubectl create namespace ${NAMESPACE} --dry-run=client -o yaml | kubectl apply -f -

# Deploy using kustomize
echo "🔧 Applying Kustomize overlay for ${ENVIRONMENT}..."
kubectl apply -k k8s/overlays/${ENVIRONMENT}

echo ""
echo "⏳ Waiting for deployment to be ready..."
kubectl wait --for=condition=available --timeout=120s \
    deployment/${ENVIRONMENT}-websocket-server -n ${NAMESPACE} || true

echo ""
echo "✅ Deployment complete!"
echo ""
echo "📊 Current status:"
kubectl get all -n ${NAMESPACE}

echo ""
echo "📝 To view logs:"
echo "   kubectl logs -n ${NAMESPACE} -l app=websocket-server --tail=100 -f"
echo ""
echo "🔌 To port-forward:"
echo "   kubectl port-forward -n ${NAMESPACE} svc/${ENVIRONMENT}-websocket-server 8080:8080"
