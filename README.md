# Vortex Privacy Protocol v2

A zero-knowledge proof system enabling private SUI token transfers on the Sui blockchain. Vortex implements a UTXO-based privacy pool using zkSNARKs (Groth16), allowing users to deposit, transfer, and withdraw SUI tokens while preserving transaction privacy.

## Overview

Vortex breaks the on-chain link between sender and recipient addresses through cryptographic commitments and nullifiers. Users deposit SUI into a shared pool and can later withdraw or transfer to any address without revealing the connection between deposits and withdrawals.

### Key Features

- **Privacy-Preserving Transfers**: Hide sender, recipient, and amount relationships
- **UTXO Model**: 2-input, 2-output transaction structure for flexibility
- **Cross-User Payments**: Encrypt outputs for other users' public keys
- **Merkle Tree Proofs**: Efficient membership verification (supports 67M+ commitments)
- **Groth16 zkSNARKs**: Compact proofs with on-chain verification
- **Relayer Support**: Optional third-party transaction submission

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Vortex Protocol Stack                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  TypeScript SDK (@interest-protocol/vortex-sdk)             │
│  ├─ VortexKeypair: Key generation & encryption              │
│  ├─ Utxo: Commitment/nullifier computation                  │
│  ├─ MerkleTree: Path generation & verification              │
│  └─ Proof helpers: Circuit input preparation                │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Rust Circuit (rust-circuit)                                │
│  ├─ Transaction2 Circuit: 2-in/2-out zkSNARK                │
│  ├─ Poseidon Hash: Optimized circomlib-compatible           │
│  ├─ Merkle Proofs: 26-level tree verification               │
│  └─ WASM Module: Browser-based proof generation             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Move Smart Contracts (contracts)                           │
│  ├─ vortex.move: Main pool & proof verification             │
│  ├─ vortex_merkle_tree.move: Commitment storage             │
│  ├─ vortex_proof.move: Groth16 verification interface       │
│  └─ vortex_ext_data.move: External data handling            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Privacy Guarantees

### What is Hidden

- **Deposit-Withdrawal Links**: Cannot determine which deposit funded which withdrawal
- **Amounts**: Transaction values are encrypted in output commitments
- **Recipients**: Output ownership is encrypted for recipient's key
- **Sender Identity**: Nullifiers hide which commitment was spent

### What is Public

- **Pool Total**: Overall balance in the privacy pool
- **Commitment Count**: Number of outputs created
- **Nullifiers**: Spent commitment identifiers (unlinkable to commitments)
- **Event Timing**: When deposits/withdrawals occurred

### Mathematical Security

Privacy relies on the **one-wayness of Poseidon hash**:

- **Commitment**: `Poseidon(amount, publicKey, blinding)` → Cannot reverse to find amount/owner
- **Nullifier**: `Poseidon(commitment, index, signature)` → Cannot link to original commitment
- **Signature**: `Poseidon(privateKey, commitment, index)` → Proves ownership without revealing key

## Installation

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Sui CLI 1.59+
- wasm-pack (for browser builds)

### SDK Installation

```bash
npm install @interest-protocol/vortex-sdk
# or
yarn add @interest-protocol/vortex-sdk
```

### Development Setup

```bash
# Clone repository
git clone https://github.com/your-org/vortex-v2
cd vortex-v2

# Install TypeScript dependencies
npm install

# Build Rust circuit WASM module
cd rust-circuit
wasm-pack build --target nodejs --out-dir ../scripts/vortex/pkg/nodejs
wasm-pack build --target web --out-dir ../packages/vortex/pkg/web

# Build Move contracts (optional - using deployed contracts)
cd ../contracts
sui move build
```

## Quick Start

### 1. Generate Vortex Keypair

Derive a privacy keypair from your Sui wallet signature:

```typescript
import { VortexKeypair } from '@interest-protocol/vortex-sdk';
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';

// From Sui wallet (deterministic from signature)
const suiKeypair = Ed25519Keypair.generate();
const vortexKeypair = await VortexKeypair.fromSuiWallet(
  suiKeypair.toSuiAddress(),
  async (message) => suiKeypair.signPersonalMessage(message)
);

console.log('Public Key:', vortexKeypair.publicKey);
console.log('Encryption Key:', vortexKeypair.encryptionKey);
```

### 2. Deposit SUI into Privacy Pool

```typescript
import {
  MerkleTree,
  Utxo,
  computeExtDataHash,
  Action,
} from '@interest-protocol/vortex-sdk';
import { prove } from './pkg/nodejs/vortex'; // WASM module

// Create merkle tree (sync with on-chain state)
const merkleTree = new MerkleTree(26);

// Get next commitment index from chain
const nextIndex = await vortex.nextIndex(); // e.g., 42n

// Create dummy input UTXOs (zero amounts for deposits)
const inputUtxo0 = new Utxo({
  amount: 0n,
  index: nextIndex,
  keypair: vortexKeypair,
});

const inputUtxo1 = new Utxo({
  amount: 0n,
  index: nextIndex + 1n,
  keypair: vortexKeypair,
});

// Create output UTXOs (actual deposit)
const outputUtxo0 = new Utxo({
  amount: 1_000_000_000n, // 1 SUI
  index: 0n, // Not used for outputs
  keypair: vortexKeypair,
});

const outputUtxo1 = new Utxo({
  amount: 0n, // Dummy output
  index: 0n,
  keypair: vortexKeypair,
});

// Encrypt outputs for yourself
const encryptedUtxo0 = VortexKeypair.encryptUtxoFor(
  outputUtxo0.payload(),
  vortexKeypair.encryptionKey
);

const encryptedUtxo1 = VortexKeypair.encryptUtxoFor(
  outputUtxo1.payload(),
  vortexKeypair.encryptionKey
);

// Prepare external data
const extDataHash = computeExtDataHash({
  recipient: suiKeypair.toSuiAddress(),
  value: 1_000_000_000n,
  valueSign: true, // true = deposit
  relayer: '0x0',
  relayerFee: 0n,
  encryptedOutput0: fromHex(encryptedUtxo0),
  encryptedOutput1: fromHex(encryptedUtxo1),
});

// Generate proof
const input = toProveInput({
  merkleTree,
  publicAmount: 1_000_000_000n,
  extDataHash: bytesToBigInt(reverseBytes(extDataHash)),
  nullifier0: inputUtxo0.nullifier(),
  nullifier1: inputUtxo1.nullifier(),
  commitment0: outputUtxo0.commitment(),
  commitment1: outputUtxo1.commitment(),
  vortexKeypair,
  inputUtxo0,
  inputUtxo1,
  outputUtxo0,
  outputUtxo1,
});

const proof = JSON.parse(prove(JSON.stringify(input), provingKey));

// Submit transaction to chain (see scripts/vortex/transact/deposit-tx.ts)
```

### 3. Withdraw SUI from Privacy Pool

```typescript
// Scan events for your commitments
const commitmentEvents = await suiClient.queryEvents({
  query: { MoveEventType: vortex.newCommitmentEventType },
  order: 'ascending',
});

// Decrypt your UTXOs
const utxos = [];
for (const event of parseNewCommitmentEvent(commitmentEvents)) {
  try {
    const utxo = vortexKeypair.decryptUtxo(event.encryptedOutput);
    utxos.push({ ...utxo, index: event.index });
  } catch {
    // Not your UTXO
  }
}

// Select UTXOs to spend (e.g., largest first)
utxos.sort((a, b) => (b.amount > a.amount ? 1 : -1));
const [utxo0, utxo1] = utxos.slice(0, 2);

// Create input UTXOs from stored data
const inputUtxo0 = new Utxo({
  amount: utxo0.amount,
  index: utxo0.index,
  blinding: utxo0.blinding,
  keypair: vortexKeypair,
});

const inputUtxo1 = new Utxo({
  amount: utxo1?.amount || 0n,
  index: utxo1?.index || 0n,
  blinding: utxo1?.blinding,
  keypair: vortexKeypair,
});

// Create change output (if any)
const withdrawAmount = 500_000_000n; // 0.5 SUI
const changeAmount = inputUtxo0.amount + inputUtxo1.amount - withdrawAmount;

const outputUtxo0 = new Utxo({
  amount: changeAmount,
  index: 0n,
  keypair: vortexKeypair,
});

const outputUtxo1 = new Utxo({
  amount: 0n,
  index: 0n,
  keypair: vortexKeypair,
});

// For withdrawals, publicAmount = BN254_FIELD_MODULUS - withdrawAmount
const publicAmount = BN254_FIELD_MODULUS - withdrawAmount;

// Generate proof and submit (similar to deposit)
// See scripts/vortex/transact/withdraw-tx.ts for complete example
```

### 4. Private Transfer to Another User

```typescript
// Get recipient's encryption key (from registry or QR code)
const recipientEncryptionKey = await vortex.encryptionKey(recipientAddress);

// Create output encrypted for recipient
const outputUtxo0 = new Utxo({
  amount: 1_000_000_000n,
  index: 0n,
  keypair: recipientKeypair, // Use recipient's public key
});

const encryptedForRecipient = VortexKeypair.encryptUtxoFor(
  outputUtxo0.payload(),
  recipientEncryptionKey // Recipient can decrypt
);

// Rest of flow similar to withdrawal
// Recipient scans events and decrypts with their private key
```

## SDK Reference

### VortexKeypair

Manages privacy keys and encryption:

```typescript
// Generate from random
const keypair = VortexKeypair.generate();

// Derive from Sui wallet (deterministic)
const keypair = await VortexKeypair.fromSuiWallet(address, signFn);

// From string representation
const keypair = VortexKeypair.fromString('0x...');

// Sign for nullifier generation
const signature = keypair.sign(commitment, merklePath);

// Encrypt UTXO for recipient
const encrypted = VortexKeypair.encryptUtxoFor(payload, recipientEncKey);

// Decrypt received UTXO
const utxo = keypair.decryptUtxo(encryptedData);
```

### Utxo

Represents unspent transaction outputs:

```typescript
const utxo = new Utxo({
  amount: 1_000_000_000n,
  blinding: 123456789n, // Optional, random if not provided
  keypair: vortexKeypair,
  index: 42n,
});

// Compute commitment (public)
const commitment = utxo.commitment(); // Poseidon(amount, pubkey, blinding)

// Compute nullifier (reveals spent UTXO)
const nullifier = utxo.nullifier(); // Poseidon(commitment, index, signature)

// Get payload for encryption
const payload = utxo.payload(); // { amount, blinding, index }
```

### MerkleTree

Tracks commitment tree state:

```typescript
const tree = new MerkleTree(26); // 26 levels = 67M capacity

// Insert commitments (pairs)
tree.insertPair(commitment1, commitment2);

// Bulk insert
tree.bulkInsert([c1, c2, c3, c4]); // Must be even number

// Generate proof
const { pathElements, pathIndices } = tree.path(index);

// Verify proof
const isValid = tree.verify(leaf, pathElements, pathIndices, root);

// Get state
console.log('Root:', tree.root());
console.log('Next index:', tree.nextIndex);
console.log('Capacity:', tree.getCapacity());
```

### Proof Generation

```typescript
import { prove, verify } from './pkg/nodejs/vortex';

// Prepare circuit input
const input = toProveInput({
  merkleTree,
  publicAmount,
  extDataHash,
  nullifier0,
  nullifier1,
  commitment0,
  commitment1,
  vortexKeypair,
  inputUtxo0,
  inputUtxo1,
  outputUtxo0,
  outputUtxo1,
});

// Generate proof (5-15 seconds)
const proofJson = prove(JSON.stringify(input), provingKeyHex);
const proof = JSON.parse(proofJson);

// Verify locally (optional)
const isValid = verify(proofJson, verifyingKeyHex);
```

## Smart Contract Reference

### Deployed Addresses (Devnet)

```
Package ID:     0x346b00f50e9470ca1375248d77ad3e10a0c832865123af6136d94e0f571d6230
Vortex Pool:    0xc1262de72c64e47765b3c07b959a91c46c9c065adf075bb5b1f1ee577ec03d5a
Registry:       0x08c519e23a5671fb8345a515426fcc00c8d63ae797d20ebde3a6b959535e12b2
```

### Key Functions

#### `transact()`

Main transaction function accepting proof and external data:

```move
public fun transact(
    self: &mut Vortex,
    proof: Proof,
    ext_data: ExtData,
    deposit: Coin<SUI>,
    ctx: &mut TxContext,
)
```

- Verifies Groth16 proof on-chain
- Checks nullifiers not already spent
- Processes deposit or withdrawal
- Emits `NewCommitment` events with encrypted outputs

#### `register()`

Register encryption key for receiving private transfers:

```move
public fun register(
    registry: &mut Registry,
    encryption_key: String,
    ctx: &mut TxContext
)
```

#### View Functions

```move
public fun root(self: &Vortex): u256
public fun is_nullifier_spent(self: &Vortex, nullifier: u256): bool
public fun next_index(self: &Vortex): u64
public fun encryption_key(registry: &Registry, address: address): Option<String>
```

## Circuit Details

### Constraints

- **Total**: ~15,000 R1CS constraints
- **Merkle Proofs**: ~6,500 per input (2 inputs)
- **Poseidon Hashes**: ~150 per hash call
- **Range Checks**: ~254 per amount (bit decomposition)

### Public Inputs (7 total)

1. `root` - Merkle tree root
2. `publicAmount` - Value entering/leaving pool
3. `extDataHash` - Hash of external data (recipient, relayer, etc.)
4. `inputNullifier0` - First input nullifier
5. `inputNullifier1` - Second input nullifier
6. `outputCommitment0` - First output commitment
7. `outputCommitment1` - Second output commitment

### Private Inputs (26 total)

**Inputs (per UTXO × 2)**:

- `inPrivateKey` - Owner's private key
- `inAmount` - UTXO amount
- `inBlinding` - Random blinding factor
- `inPathIndex` - Position in Merkle tree
- `merklePath` - Sibling hashes (26 levels × 2 hashes)

**Outputs (per UTXO × 2)**:

- `outPublicKey` - Recipient's public key
- `outAmount` - Output amount
- `outBlinding` - Random blinding factor

### Key Generation

```bash
# Using deterministic test keys (development only)
cd rust-circuit
cargo run --release --bin keygen -- \
  --output-dir ./keys \
  --use-test-seed

# Using trusted setup (production)
# 1. Download Powers of Tau (e.g., Hermez ptau file)
# 2. Run phase 2 ceremony
# 3. Generate final keys
```

## Security Considerations

### ⚠️ Known Limitations

1. **Merkle Tree Capacity**: Fixed at 2^26 (~67M commitments)

   - When full, no new deposits allowed
   - Existing funds always withdrawable (nullifier-only transactions)

2. **Anonymity Set**: Privacy proportional to pool usage

   - Larger anonymity set = better privacy
   - Avoid being sole user in time window

3. **Front-Running**: Relayers can see encrypted outputs

   - Use trusted relayers or self-relay
   - Consider time-locked transactions

4. **Key Management**: Private key compromise reveals all history
   - Store securely (hardware wallet recommended)
   - Derive from Sui wallet for convenience

### Best Practices

✅ **DO**:

- Use production keys with proper trusted setup
- Wait for multiple deposits before withdrawing
- Vary withdrawal amounts and timing
- Register encryption key for receiving transfers
- Verify proofs locally before submission

❌ **DON'T**:

- Reuse nullifiers (enforced on-chain)
- Deposit and immediately withdraw same amount
- Share private keys or seed phrases
- Use deterministic test keys in production
- Link Vortex address to public identity

## Examples

See the `scripts/vortex/transact/` directory for complete examples:

- [`deposit.ts`](https://github.com/interest-protocol/sdk-monorepo/blob/main/scripts/vortex/transact/deposit.ts) - Generate and verify deposit proof
- [`deposit-tx.ts`](https://github.com/interest-protocol/sdk-monorepo/blob/main/scripts/vortex/transact/deposit-tx.ts) - Submit deposit transaction
- [`withdraw-tx.ts`](https://github.com/interest-protocol/sdk-monorepo/blob/main/scripts/vortex/transact/withdraw-tx.ts) - Submit withdrawal transaction
- [`fund.ts`](https://github.com/interest-protocol/sdk-monorepo/blob/main/scripts/vortex/transact/fund.ts) - Fund test accounts

## Development

### Run Tests

```bash
# TypeScript SDK tests
npm test

# Rust circuit tests
cd rust-circuit
cargo test

# Move contract tests (uncomment in vortex_tests.move)
cd contracts
sui move test
```

### Build WASM Module

```bash
cd rust-circuit

# Node.js target
wasm-pack build --target nodejs --out-dir ../scripts/vortex/pkg/nodejs

# Browser target
wasm-pack build --target web --out-dir ../packages/vortex/pkg/web

# With optimizations
wasm-pack build --target web --release --out-dir ../packages/vortex/pkg/web
```

### Deploy Contracts

```bash
cd contracts

# Build
sui move build

# Publish to devnet
sui client publish --gas-budget 500000000

# Update package ID in constants.ts
```

## Troubleshooting

### "Constraints not satisfied" error

- Verify Merkle path is correct for UTXO index
- Check commitment matches stored value
- Ensure amounts don't exceed 2^248 - 1
- Confirm private key matches public key in commitment

### "Nullifier already spent" error

- UTXO was already consumed in previous transaction
- Rescan commitment events for available UTXOs
- Check for transaction replay

### "Proof verification failed" on-chain

- Ensure using correct verifying key
- Verify public inputs match proof generation
- Check proof serialization format (compressed)
- Confirm BN254 field element ordering

### "Cannot decrypt UTXO" error

- Encrypted for different encryption key
- Verify encryption key derivation matches
- Check event index matches UTXO index
- Ensure using correct keypair

## Roadmap

- [ ] Indexer
- [ ] Frontend
- [ ] Alpha Testnet release

## Resources

- **Documentation**: [docs/](docs/)
- **Smart Contracts**: [contracts/sources/](contracts/sources/)
- **TypeScript SDK**: [sdk-monorepo/packages/vortex/](https://github.com/interest-protocol/sdk-monorepo/tree/main/packages/vortex)
- **Rust Circuit**: [rust-circuit/src/](rust-circuit/src/)
- **Examples**: [sdk-monorepo/scripts/vortex/transact/](https://github.com/interest-protocol/sdk-monorepo/tree/main/scripts/vortex/transact)

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

For security-critical changes, please contact the team privately first.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- **Tornado Cash Nova** - Inspiration for paired insertion Merkle tree
- **Circomlib** - Poseidon hash implementation
- **Arkworks** - zkSNARK libraries
- **Sui Foundation** - Native Groth16 verification support

---

**⚠️ Disclaimer**: This software is experimental. Use at your own risk. No warranty provided. Always verify security assumptions before handling real value.
