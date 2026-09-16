# Deployment Guide

## Prerequisites
- Rust (stable), Node.js 20+, Stellar CLI

## 1. Deploy Smart Contract
```bash
cd contracts
stellar contract build
stellar contract deploy --network testnet --source <KEY> --wasm target/wasm32v1-none/release/*.wasm
stellar contract invoke --network testnet --id <ID> -- initialize --admin <ADMIN>
```

## 2. Run Backend
```bash
cd backend
export CONTRACT_ID=<ID>
cargo run --release
```

## 3. Run Frontend
```bash
cd frontend
npm install && npm run dev
```

## Environment Variables
| Var | Default | Description |
|---|---|---|
| PORT | 3000 | Backend port |
| CONTRACT_ID | (empty) | Soroban contract ID |
| SOROBAN_RPC_URL | https://soroban-testnet.stellar.org:443 | RPC |
| HORIZON_URL | https://horizon-testnet.stellar.org | Horizon |
| NETWORK_PASSPHRASE | Test SDF Network ; September 2015 | Passphrase |
| SKIP_SEED | (unset) | Set 1 to skip demo data |
