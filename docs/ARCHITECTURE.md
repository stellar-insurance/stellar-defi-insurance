# Architecture

## Overview

Stellar DeFi Insurance is a decentralized insurance protocol on the Stellar
blockchain that provides coverage for smart contract risks. Users purchase
policies by paying premiums into a coverage pool. If a covered smart contract
is exploited, policyholders file claims that are reviewed by designated
assessors.

## System Architecture

```
┌──────────────────────────────────────────────────────────┐
│                   Frontend (React)                        │
│  Dashboard · Policies · Purchase · Claims · Admin · Wallet│
└──────────────────────┬───────────────────────────────────┘
                       │ HTTP
┌──────────────────────┼───────────────────────────────────┐
│                Backend API (Axum)                         │
│  /policies · /claims · /pool · /admin · /assessors       │
│  InsuranceStore: in-memory policy/claim/pool management   │
└──────────────────────┬───────────────────────────────────┘
                       │ (production: Soroban RPC)
┌──────────────────────┼───────────────────────────────────┐
│              Stellar Testnet (Soroban)                    │
│  InsuranceContract: purchase_policy · file_claim          │
│  approve_claim · reject_claim · mark_paid · pool mgmt    │
└──────────────────────────────────────────────────────────┘
```

## Smart Contract

**Language:** Rust + Soroban SDK 22.0

### Roles
- **Admin** — set at initialization; manages assessors, premium rates, pause
- **Assessor** — approved by admin; reviews/approves/rejects claims
- **Policyholder** — any address that purchases a policy

### Core Flows
1. **Purchase:** User pays premium → receives policy with coverage & expiry
2. **Claim:** Policyholder files claim with evidence → policy marked as claimed
3. **Review:** Assessor approves or rejects; rejection restores policy to active
4. **Payout:** Admin marks approved claims as paid; pool deducted

### Key Parameters
- Premium rate: 5% default (configurable in basis points)
- Min coverage: 1,000 units · Max: 1,000,000,000
- Max policy duration: 90 days
- Max evidence length: 1,024 chars

### Contract Functions
| Function | Access | Description |
|---|---|---|
| `initialize(admin)` | Once | Set admin |
| `purchase_policy(...)` | Holder | Buy insurance policy |
| `file_claim(...)` | Policyholder | File a claim |
| `approve_claim(id, assessor)` | Assessor | Approve claim |
| `reject_claim(id, assessor)` | Assessor | Reject claim |
| `mark_claim_paid(id)` | Admin | Mark as paid |
| `get_policy(id)` | Public | Fetch policy |
| `get_claim(id)` | Public | Fetch claim |
| `get_pool_balance()` | Public | Pool balance |
| `grant/revoke_assessor(who)` | Admin | Manage assessors |
| `pause()/unpause()` | Admin | Toggle protocol |
| `set_premium_rate(bps)` | Admin | Set premium rate |
| `expire_stale_policies()` | Public | Expire past-due policies |

## Backend API

**Stack:** Rust + Axum 0.7 · Port 3000

| Method | Endpoint | Description |
|---|---|---|
| GET | `/health` | Health check |
| GET | `/api/v1/status` | Service status |
| GET | `/api/v1/policies` | List policies |
| GET | `/api/v1/policies/:id` | Get policy |
| POST | `/api/v1/policies` | Purchase policy |
| GET | `/api/v1/claims` | List claims |
| POST | `/api/v1/claims` | File claim |
| POST | `/api/v1/claims/approve` | Approve claim |
| POST | `/api/v1/claims/reject` | Reject claim |
| POST | `/api/v1/claims/:id/pay` | Mark paid |
| GET | `/api/v1/pool` | Pool info |
| GET | `/api/v1/admin` | Admin address |
| GET | `/api/v1/assessors/:addr` | Check assessor |

## Frontend

6 views: Dashboard (stats + recent), Policies (table), Purchase (form with
premium preview), Claims (file + list), Admin (approve/reject, pause, assessor
management), Wallet. Dark mode + mobile responsive.

## Tests
- Contract: 21 unit tests
- Backend: 10 integration tests
- Total: 31 tests

## Security
- require_auth on all state-changing calls
- Role-based: only assessors review claims, only admin manages protocol
- Policy marked as "Claimed" prevents duplicate claims
- Rejection restores policy to Active
- Pool balance checked before approval
- Input validation: min/max coverage, duration limits, evidence required
