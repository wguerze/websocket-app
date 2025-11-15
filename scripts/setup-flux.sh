#!/bin/bash
# FluxCD setup script

set -e

GITHUB_USER=${GITHUB_USER:-williamguerzeder}
GITHUB_REPO=${GITHUB_REPO:-websocket-app}
GITHUB_BRANCH=${GITHUB_BRANCH:-main}

echo "🌊 Setting up FluxCD for GitOps deployment"
echo ""
echo "GitHub User: ${GITHUB_USER}"
echo "GitHub Repo: ${GITHUB_REPO}"
echo "Branch: ${GITHUB_BRANCH}"
echo ""

# Check if flux CLI is installed
if ! command -v flux &> /dev/null; then
    echo "📥 FluxCD CLI not found. Installing..."
    curl -s https://fluxcd.io/install.sh | sudo bash
    echo "✅ FluxCD CLI installed"
else
    echo "✅ FluxCD CLI already installed"
fi

# Check if kubectl is configured
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ kubectl is not configured. Please configure access to your cluster first."
    exit 1
fi

echo ""
echo "🔍 Checking cluster prerequisites..."
flux check --pre

echo ""
read -p "Do you want to bootstrap FluxCD on this cluster? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Aborted"
    exit 1
fi

# Check for GitHub token
if [ -z "${GITHUB_TOKEN}" ]; then
    echo "❌ GITHUB_TOKEN environment variable is not set"
    echo "Please create a GitHub Personal Access Token with 'repo' permissions and export it:"
    echo "   export GITHUB_TOKEN=<your-token>"
    exit 1
fi

echo ""
echo "🚀 Bootstrapping FluxCD..."
flux bootstrap github \
  --owner=${GITHUB_USER} \
  --repository=${GITHUB_REPO} \
  --branch=${GITHUB_BRANCH} \
  --path=./flux-system \
  --personal

echo ""
echo "⏳ Waiting for FluxCD to be ready..."
kubectl wait --for=condition=ready --timeout=5m \
    pod -l app=source-controller -n flux-system

echo ""
echo "✅ FluxCD bootstrap complete!"
echo ""
echo "📋 Applying GitOps configurations..."

# Apply GitRepository
kubectl apply -f flux-system/gitrepository.yaml

# Apply Kustomizations for each environment
kubectl apply -f flux-system/kustomization-dev.yaml
kubectl apply -f flux-system/kustomization-staging.yaml
kubectl apply -f flux-system/kustomization-production.yaml

echo ""
echo "⏳ Waiting for reconciliation..."
sleep 10

echo ""
echo "📊 FluxCD Status:"
flux get sources git
echo ""
flux get kustomizations

echo ""
echo "✅ FluxCD setup complete!"
echo ""
echo "🔍 Monitor deployments with:"
echo "   flux logs --follow"
echo "   flux get all"
echo ""
echo "🔄 Force reconciliation:"
echo "   flux reconcile source git websocket-app"
echo "   flux reconcile kustomization websocket-app-dev"
