#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SCRIPT_DIR/../.env.sepolia"

if [[ ! -f "$ENV_FILE" ]]; then
    echo "Error: $ENV_FILE not found"
    exit 1
fi

DEPLOY_RPC_URL=$(grep '^DEPLOY_RPC_URL=' "$ENV_FILE" | cut -d'=' -f2-)

if [[ -z "$DEPLOY_RPC_URL" ]]; then
    echo "Error: DEPLOY_RPC_URL not found in $ENV_FILE"
    exit 1
fi

echo "Starting starknet-devnet forking $DEPLOY_RPC_URL ..."
starknet-devnet --fork-network "$DEPLOY_RPC_URL" 1> /dev/null &
DEVNET_PID=$!
trap "kill $DEVNET_PID 2>/dev/null" EXIT

until curl -s http://127.0.0.1:5050/is_alive > /dev/null 2>&1; do sleep 0.2; done
echo "starknet-devnet ready (pid $DEVNET_PID)"

snforge test e2e "$@"
