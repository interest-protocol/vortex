# Vortex

Private transactions on Sui using zero-knowledge proofs.

Vortex is a privacy protocol that breaks the on-chain link between deposit and withdrawal addresses. Users deposit SUI into a shared pool, then withdraw to any address without revealing which deposit they own.

## How it works

1. **Deposit** - Send SUI to the pool. You receive an encrypted note (UTXO) that proves ownership.
2. **Transfer** - Encrypt a note with someone's public key. Only they can decrypt and spend it.
3. **Withdraw** - Generate a ZK proof that you own a valid note, without revealing which one.

The protocol uses a 2-input/2-output UTXO model. Each transaction consumes up to 2 existing notes and creates 2 new ones (one for the recipient, one for change).

## Project structure

```
vortex-v2/
├── contracts/     # Move smart contracts (Sui)
├── circuit/       # Rust zkSNARK circuit (Groth16)
├── indexer/       # Rust indexer for tracking commitments
└── api/           # TypeScript REST API
```

### Contracts

The Move contracts handle:
- Proof verification (native Groth16 on Sui)
- Merkle tree for commitment storage
- Nullifier tracking to prevent double-spends
- Encryption key registry for receiving transfers

### Circuit

The Rust circuit generates Groth16 proofs that verify:
- Input notes exist in the Merkle tree
- Nullifiers are correctly computed
- Output commitments are well-formed
- Amounts balance (inputs = outputs + public delta)

Compiles to WASM for browser-based proof generation.

### Indexer

Rust service that:
- Indexes commitment events from Sui
- Maintains Merkle tree state
- Provides API for building proofs

### API

REST API for:
- Account management
- Transaction building
- Merkle tree queries

## Development

### Prerequisites

- Rust 1.70+
- Sui CLI 1.59+
- Bun
- wasm-pack (for browser builds)

### Building

```bash
# Contracts
cd contracts && sui move build

# Circuit
cd circuit && cargo build --release

# Indexer
cd indexer && cargo build --release

# API
cd api && bun install && bun run build
```

### Testing

```bash
# Contracts
cd contracts && sui move test

# Circuit
cd circuit && cargo test

# Indexer
cd indexer && cargo test

# API
cd api && bun test
```

## SDK

The TypeScript SDK is published separately at [@interest-protocol/vortex-sdk](https://github.com/interest-protocol/sdk-monorepo/tree/main/packages/vortex).

```bash
npm install @interest-protocol/vortex-sdk
```

Basic usage:

```typescript
import { VortexKeypair, deposit, withdraw } from '@interest-protocol/vortex-sdk';

// Create keypair from Sui wallet
const keypair = await VortexKeypair.fromSuiWallet(address, signFn);

// Deposit
const tx = await deposit({ amount: 1_000_000_000n, ... });

// Withdraw
const tx = await withdraw({ amount: 500_000_000n, recipient: '0x...', ... });
```

## Deployed contracts (Testnet)

```
Package:   0xcf81b96e392f82b776ee980108357426b726c4043c838822545a307e12c5ded6
Registry:  0xf2c11c297e0581e0279714f6ba47e26d03d9a70756036fab5882ebc0f1d2b3b1
```

## Privacy considerations

**Hidden**: Which deposit funded a withdrawal. Transaction amounts. Sender/recipient relationship.

**Public**: Total pool balance. Number of commitments. When transactions occurred. Nullifiers (but they can't be linked to commitments).

Privacy depends on the anonymity set. More users = better privacy. Avoid depositing and withdrawing the same amount in quick succession.

## Security

This is experimental software. The circuit uses Groth16 with a trusted setup. Key management is critical - losing your Vortex private key means losing access to your funds.

## License

MIT
