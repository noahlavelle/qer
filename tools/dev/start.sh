#!/usr/bin/env bash

# Build and deploy the dev environment with `skaffold dev`, then forward the
# API and queue engine ports once their pods are running.
# Usage: tools/dev/start.sh [namespace]
set -euo pipefail

namespace="${1:-default}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

skaffold dev --namespace "$namespace" &
skaffold_pid=$!

cleanup() {
  kill "$skaffold_pid" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

pod_ready() {
  local app="$1"

  kubectl get pods \
    --namespace "$namespace" \
    --selector "app=$app" \
    --field-selector=status.phase=Running \
    --output "jsonpath={.items[0].metadata.name}" 2>/dev/null
}

echo "Waiting for api and engine pods to be running..."
while [[ -z "$(pod_ready api)" || -z "$(pod_ready engine)" ]]; do
  if ! kill -0 "$skaffold_pid" 2>/dev/null; then
    echo "skaffold dev exited before pods became ready." >&2
    exit 1
  fi
  sleep 2
done

"$script_dir/portforward.sh" "$namespace"
