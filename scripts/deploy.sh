#!/usr/bin/env bash
set -euo pipefail
KEY="${1:?Usage: $0 <secret_key>}"
echo "Building contract..."
cd contracts && stellar contract build
WASM=$(ls target/wasm32v1-none/release/*.wasm 2>/dev/null | head -1)
echo "Deploying to testnet..."
CONTRACT_ID=$(stellar contract deploy --network testnet --source "$KEY" --wasm "$WASM" --json | jq -r '.contract_id')
ADMIN=$(stellar keys address "$KEY")
stellar contract invoke --network testnet --source "$KEY" --id "$CONTRACT_ID" -- initialize --admin "$ADMIN"
echo "Contract deployed: $CONTRACT_ID (Admin: $ADMIN)"
