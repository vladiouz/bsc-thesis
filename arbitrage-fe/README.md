# Arbitrage FE

Frontend dashboard for the arbitrage smart contract.

It reads the Devnet contract address from:
`../arbitrage-sc/interactor/state.toml`

The sync runs automatically on `npm run dev` and `npm run build`.

## Features

- network parameter switch: `devnet` or `mainnet`
- contract overview (owner, deploy time, staked token)
- latest transactions with decoded `executeTrades` route
- per-transaction token flow (in/out net for the contract)
- estimated trade profit (from staked token net flow)
- aggregate metrics (total fees, dev winnings, trade count)

## Run

```bash
npm install
npm run dev
```

## Network Selection

Use URL query parameter:

- Devnet: `http://localhost:5173/?network=devnet`
- Mainnet: `http://localhost:5173/?network=mainnet`

Optional contract override:

- `?network=devnet&contract=erd1...`

## Environment Variables

Copy `.env.example` to `.env` and adjust if needed.

- `VITE_DEFAULT_NETWORK=devnet|mainnet`
- `VITE_MAINNET_CONTRACT_ADDRESS=erd1...` (required for mainnet mode)
- `VITE_DEVNET_CONTRACT_ADDRESS=erd1...` (optional override)
