# Bitcoin Script Explorer

A full-stack Bitcoin script interpreter with a visual React frontend and a Rust backend.
Paste any transaction ID, get a complete step-by-step execution trace of its scripts.

## Architecture

```
btc-explorer/
├── backend/          Rust + Axum web server
│   └── src/
│       ├── main.rs       HTTP server, routing, JSON responses
│       ├── fetcher.rs    Fetches transactions from mempool.space
│       ├── opcode.rs     OpCode enum with explanations
│       ├── parser.rs     Raw bytes → Vec<OpCode>
│       ├── crypto.rs     Real SHA256 + RIPEMD160
│       └── engine.rs     Stack machine execution
└── frontend/         React + TypeScript + Vite
    └── src/
        ├── App.tsx
        ├── components/
        │   ├── LandingHero.tsx    Hero with P2PKH diagram
        │   ├── SearchBar.tsx      txid input
        │   ├── TxMeta.tsx         Transaction summary cards
        │   ├── ScriptPanel.tsx    Expandable script card
        │   └── ExecutionTrace.tsx Step-by-step VM trace
        ├── hooks/
        │   └── useTransaction.ts  Data fetching hook
        └── types/
            └── index.ts           TypeScript types
```

## Setup & Running

### 1. Start the Rust backend

```bash
cd backend
cargo run
# Server starts on http://localhost:3001
```

### 2. Start the React frontend

```bash
cd frontend
npm install
npm run dev
# Opens on http://localhost:5173
```

### 3. Open http://localhost:5173 and paste a txid

**Try these transactions:**

| Transaction | txid |
|---|---|
| First BTC transfer (Satoshi → Hal Finney) | `f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16` |
| Bitcoin pizza (10,000 BTC) | `a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d` |
| Genesis coinbase | `4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b` |

## How It Works

The backend fetches the raw transaction JSON from `mempool.space/api/tx/{txid}`,
parses every script's hex bytes opcode-by-opcode, runs them through the stack
machine with real SHA256+RIPEMD160 crypto, and returns a structured JSON trace.

The frontend renders each opcode as a colored chip in a pipeline view, then shows
every execution step with the full stack state and a plain-English explanation of
what that opcode is doing and why it exists in the Bitcoin protocol.

## API

```
GET /api/tx/:txid   → Full transaction with execution traces
GET /api/health     → { "ok": true }
```

## Notes

- `OP_CHECKSIG` is mocked (always returns TRUE). Real secp256k1 verification
  requires the `secp256k1` or `k256` crate and the full transaction sighash.
- SegWit inputs show witness data but don't execute (witness scripts need
  separate handling per BIP 141).
- Data via mempool.space — requires internet connection.
