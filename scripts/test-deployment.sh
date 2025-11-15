#!/bin/bash
# Test deployment script

set -e

ENVIRONMENT=${1:-dev}
NAMESPACE="websocket-app-${ENVIRONMENT}"

echo "🧪 Testing WebSocket Server deployment in ${ENVIRONMENT}"
echo "Namespace: ${NAMESPACE}"
echo ""

# Check if deployment exists
echo "1️⃣ Checking deployment..."
if kubectl get deployment ${ENVIRONMENT}-websocket-server -n ${NAMESPACE} &> /dev/null; then
    echo "   ✅ Deployment exists"
else
    echo "   ❌ Deployment not found"
    exit 1
fi

# Check if pods are running
echo "2️⃣ Checking pods..."
READY_PODS=$(kubectl get pods -n ${NAMESPACE} -l app=websocket-server -o json | jq '.items | map(select(.status.phase == "Running")) | length')
TOTAL_PODS=$(kubectl get pods -n ${NAMESPACE} -l app=websocket-server -o json | jq '.items | length')

echo "   Running: ${READY_PODS}/${TOTAL_PODS}"

if [ "${READY_PODS}" -eq 0 ]; then
    echo "   ❌ No pods running"
    kubectl get pods -n ${NAMESPACE} -l app=websocket-server
    exit 1
else
    echo "   ✅ Pods are running"
fi

# Check service
echo "3️⃣ Checking service..."
if kubectl get svc ${ENVIRONMENT}-websocket-server -n ${NAMESPACE} &> /dev/null; then
    echo "   ✅ Service exists"
    kubectl get svc ${ENVIRONMENT}-websocket-server -n ${NAMESPACE}
else
    echo "   ❌ Service not found"
    exit 1
fi

# Test connectivity via port-forward
echo "4️⃣ Testing connectivity..."
echo "   Starting port-forward on localhost:8888..."

# Start port-forward in background
kubectl port-forward -n ${NAMESPACE} svc/${ENVIRONMENT}-websocket-server 8888:8080 &
PF_PID=$!

# Wait for port-forward to be ready
sleep 3

# Test connection
if nc -z localhost 8888 2>/dev/null; then
    echo "   ✅ Port 8080 is accessible"
else
    echo "   ❌ Cannot connect to port 8080"
    kill $PF_PID 2>/dev/null || true
    exit 1
fi

# Kill port-forward
kill $PF_PID 2>/dev/null || true

echo ""
echo "✅ All tests passed!"
echo ""
echo "📊 Deployment details:"
kubectl get deployment ${ENVIRONMENT}-websocket-server -n ${NAMESPACE}
echo ""
kubectl get pods -n ${NAMESPACE} -l app=websocket-server

echo ""
echo "🔌 To connect:"
echo "   kubectl port-forward -n ${NAMESPACE} svc/${ENVIRONMENT}-websocket-server 8080:8080"
echo "   Then run: cargo run --bin client"
