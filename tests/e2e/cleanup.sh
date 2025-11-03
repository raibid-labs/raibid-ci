#!/usr/bin/env bash
# E2E Test Cleanup Script
# Cleans up all test artifacts after E2E test execution

set -euo pipefail

NAMESPACE="${RAIBID_NAMESPACE:-raibid-ci}"
REDIS_URL="${TEST_REDIS_URL:-redis://localhost:6379}"

echo "=== E2E Test Cleanup ==="

# Function to clean Redis test data
cleanup_redis() {
    echo "Cleaning Redis test data..."

    if command -v redis-cli &> /dev/null; then
        redis-cli -u "${REDIS_URL}" DEL ci:jobs || true
        redis-cli -u "${REDIS_URL}" DEL "job:*" || true
        redis-cli -u "${REDIS_URL}" DEL "job:*:logs" || true
        echo "✓ Redis cleaned"
    else
        echo "⚠ redis-cli not available, skipping Redis cleanup"
    fi
}

# Function to clean Kubernetes test resources
cleanup_kubernetes() {
    echo "Cleaning Kubernetes test resources..."

    if command -v kubectl &> /dev/null; then
        # Delete test agent pods
        kubectl delete pods -n "${NAMESPACE}" -l "app=raibid-agent,test=e2e" --ignore-not-found || true

        # Delete test jobs
        kubectl delete jobs -n "${NAMESPACE}" -l "test=e2e" --ignore-not-found || true

        echo "✓ Kubernetes resources cleaned"
    else
        echo "⚠ kubectl not available, skipping Kubernetes cleanup"
    fi
}

# Function to clean Docker test images
cleanup_docker() {
    echo "Cleaning Docker test images..."

    if command -v docker &> /dev/null; then
        # Remove test images
        docker images --filter "reference=*/raibid-ci/test-fixture:*" -q | xargs -r docker rmi -f || true

        echo "✓ Docker images cleaned"
    else
        echo "⚠ docker not available, skipping Docker cleanup"
    fi
}

# Function to clean Gitea test repositories
cleanup_gitea() {
    echo "Cleaning Gitea test repositories..."

    # This would require Gitea API calls
    # For MVP, we'll skip this and manually clean if needed

    echo "⚠ Gitea cleanup requires manual intervention"
}

# Main cleanup
main() {
    echo "Starting cleanup for namespace: ${NAMESPACE}"

    cleanup_redis
    cleanup_kubernetes
    cleanup_docker
    cleanup_gitea

    echo ""
    echo "=== Cleanup Complete ==="
}

main "$@"
