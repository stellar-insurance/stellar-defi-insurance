# 🛡️ Stellar DeFi Insurance

A decentralized insurance protocol on the Stellar blockchain providing coverage
for smart contract risks. Built with Soroban smart contracts, an Axum REST API,
and a React frontend.

![License](https://img.shields.io/badge/license-MIT-blue)
![Network](https://img.shields.io/badge/network-Stellar%20Testnet-7d4cdb)
![Rust](https://img.shields.io/badge/Rust-1.75+-dea584)
![React](https://img.shields.io/badge/React-18-61dafb)

## What It Does

Users purchase insurance policies for smart contracts by paying premiums into a
coverage pool. If a covered contract is exploited, policyholders file claims
with evidence. Designated assessors review and approve/reject claims. Approved
claims are paid out from the coverage pool.

## Key Features

- **Policy management** — Purchase, track, and expire insurance policies
- **Claim lifecycle** — File, review (approve/reject), and payout claims
- **Coverage pool** — Premiums accumulate; admin manages pool withdrawals
- **Assessor governance** — Admin grants/revokes assessor roles
- **Configurable premium rates** — Set in basis points (default 5%)
- **Pause/unpause** — Emergency circuit breaker
- **Policy auto-expiry** — Stale policies can be expired by anyone
- **Dark mode** — Theme toggle persisted to localStorage
- **Mobile responsive** — Works on all screen sizes
- **31 tests** — 21 contract + 10 backend, all passing

## Project Structure
```
stellar-defi-insurance/
├── contracts/          # Soroban smart contract (Rust)
│   ├── src/lib.rs      # Insurance protocol: policies, claims, pool
│   ├── src/test.rs     # 21 unit tests
│   └── Cargo.toml
├── backend/            # REST API (Rust/Axum)
│   ├── src/            # Routes, models, services, state
│   ├── tests/          # 10 integration tests
│   └── Cargo.toml
├── frontend/           # React/TypeScript frontend
│   ├── src/App.tsx     # Insurance app (6 views)
│   ├── src/api.ts      # Backend API client
│   └── package.json
├── docs/               # Architecture, deployment
└── .github/workflows/  # CI/CD
```

## Quick Start
```bash
# Backend (with demo data)
cd backend && cargo run

# Frontend
cd frontend && npm install && npm run dev
```

## Testing
```bash
cd contracts && cargo test   # 21 tests
cd backend && cargo test     # 10 tests
```

## License
MIT
