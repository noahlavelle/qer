#!/usr/bin/env bash

# Forward the development API and queue engine running under `skaffold dev`.
# Usage: tools/dev/portforward.sh [namespace]
set -euo pipefail

namespace="${1:-default}"

pod_name() {
  local app="$1"
  local pod

  pod="$(kubectl get pods \
    --namespace "$namespace" \
    --selector "app=$app" \
    --field-selector=status.phase=Running \
    --output "jsonpath={.items[0].metadata.name}")"

  if [[ -z "$pod" ]]; then
    echo "No running pod found for app=$app in namespace $namespace." >&2
    echo "Start it with 'skaffold dev' and try again." >&2
    exit 1
  fi

  printf '%s' "$pod"
}

api_pod="$(pod_name api)"
engine_pod="$(pod_name engine)"

echo "Forwarding API pod $api_pod: localhost:8080 -> 8080"
kubectl --namespace "$namespace" port-forward "pod/$api_pod" 8080:8080 &
api_pid=$!

echo "Forwarding engine pod $engine_pod: localhost:50051 -> 50051"
kubectl --namespace "$namespace" port-forward "pod/$engine_pod" 50051:50051 &
engine_pid=$!

cleanup() {
  kill "$api_pid" "$engine_pid" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

wait "$api_pid" "$engine_pid"
