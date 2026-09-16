# Frontend — Stellar DeFi Insurance

React + TypeScript + Vite frontend for the DeFi Insurance protocol.

## Views
- Dashboard — Pool stats, recent policies & claims
- Policies — Full policy table
- Purchase — Buy policy with premium preview
- Claims — File claims + view all claims
- Admin — Approve/reject claims, manage assessors, pause protocol
- Wallet — Freighter wallet integration

## Development
```bash
npm install
npm run dev    # :5174 (proxies /api to backend :3000)
npm run build
```

## Features
- Dark mode (localStorage persistence)
- Mobile responsive
- Toast notifications
- Premium calculation preview
