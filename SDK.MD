# Vortex SDK

A TypeScript SDK for interacting with the Vortex privacy protocol on Sui blockchain. Vortex enables private transactions using zero-knowledge proofs (zkSNARKs) with a UTXO-based model.

## Installation

```bash
npm install @interest-protocol/vortex-sdk
# or
yarn add @interest-protocol/vortex-sdk
# or
pnpm add @interest-protocol/vortex-sdk
```

## Quick Start

```typescript
import {
  vortexSDK,
  VortexKeypair,
  deposit,
  withdraw,
  MerkleTree,
  getUnspentUtxos,
} from '@interest-protocol/vortex-sdk';
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import { SuiClient } from '@mysten/sui/client';

// Create a Vortex keypair from your Sui wallet
const suiKeypair = Ed25519Keypair.generate();
const vortexKeypair = await VortexKeypair.fromSuiWallet(
  suiKeypair.toSuiAddress(),
  (message) => suiKeypair.signPersonalMessage(message)
);

// Use the pre-configured SDK instance
const suiClient = new SuiClient({ url: 'https://fullnode.devnet.sui.io' });

// Fetch commitment events to build merkle tree
const commitmentEvents = await suiClient.queryEvents({
  query: { MoveEventType: vortexSDK.newCommitmentEventType },
  limit: 50,
});

// Get unspent UTXOs
const unspentUtxos = await getUnspentUtxos({
  commitmentEvents,
  vortexKeypair: vortexKeypair,
  vortex: vortexSDK,
});

// Build merkle tree from events
const merkleTree = new MerkleTree(26); // height 26
// Insert commitments into tree...

// Make a deposit
const depositTx = await deposit({
  amount: 1_000_000_000n, // 1 SUI in MIST
  vortex: vortexSDK,
  vortexKeypair,
  merkleTree,
  unspentUtxos,
});

depositTx.setSender(suiKeypair.toSuiAddress());
const result = await suiKeypair.signAndExecuteTransaction({
  transaction: depositTx,
  client: suiClient,
});
```

## Core Concepts

### VortexKeypair

A privacy-preserving keypair that enables private transactions. Each user maintains a VortexKeypair separate from their Sui wallet keypair.

```typescript
// Generate from Sui wallet (recommended)
const vortexKeypair = await VortexKeypair.fromSuiWallet(
  suiAddress,
  signMessageFunction
);

// Generate random keypair
const vortexKeypair = VortexKeypair.generate();

// From Sui private key
const vortexKeypair = VortexKeypair.fromSuiPrivateKey(privateKey);

// From serialized string
const vortexKeypair = VortexKeypair.fromString('0x...');

// Get public key and encryption key
console.log(vortexKeypair.publicKey);
console.log(vortexKeypair.encryptionKey);

// Sign a commitment
const signature = vortexKeypair.sign(commitment, merklePath);

// Decrypt UTXO
const utxo = vortexKeypair.decryptUtxo(encryptedData);
```

### UTXO Model

Vortex uses a UTXO (Unspent Transaction Output) model similar to Bitcoin, where each UTXO represents an unspent amount.

```typescript
import { Utxo } from '@interest-protocol/vortex-sdk';

// Create a new UTXO
const utxo = new Utxo({
  amount: 1_000_000_000n,
  keypair: vortexKeypair,
  index: 0n, // Position in merkle tree
  blinding: 123456789n, // Optional, random by default
});

// Get commitment (public)
const commitment = utxo.commitment();

// Get nullifier (prevents double-spending)
const nullifier = utxo.nullifier();

// Get payload for encryption
const payload = utxo.payload();
```

### Merkle Tree

The Merkle tree stores all commitments and provides membership proofs for transactions.

```typescript
import { MerkleTree, MERKLE_TREE_HEIGHT } from '@interest-protocol/vortex-sdk';

// Create tree with height 26 (supports 2^26 commitments)
const tree = new MerkleTree(MERKLE_TREE_HEIGHT);

// Insert commitments (pairs for better privacy)
tree.insertPair(commitment1, commitment2);

// Or insert single commitment (paired with zero)
tree.insert(commitment);

// Bulk insert
tree.bulkInsert([c1, c2, c3, c4]); // Must be even number

// Get current root
const root = tree.root();

// Generate membership proof
const { pathElements, pathIndices } = tree.path(index);

// Verify proof
const isValid = tree.verify(leaf, pathElements, pathIndices, root);

// Get tree info
console.log(tree.nextIndex); // Next insertion index
console.log(tree.elements()); // All leaves
console.log(tree.isFull()); // Check if full
```

## API Reference

### Vortex Class

The main class for interacting with the Vortex protocol.

#### Constructor

```typescript
import { Vortex } from '@interest-protocol/vortex-sdk';
import { getFullnodeUrl } from '@mysten/sui/client';

// Create custom instance
const vortex = new Vortex({
  packageId: '0x...',
  registry: {
    objectId: '0x...',
    initialSharedVersion: '4',
  },
  vortex: {
    objectId: '0x...',
    initialSharedVersion: '4',
  },
  fullNodeUrl: getFullnodeUrl('devnet'), // Optional, defaults to devnet
});

// Or use pre-configured instance (recommended)
import { vortexSDK } from '@interest-protocol/vortex-sdk';
```

#### Methods

```typescript
// Register encryption key
const { tx } = vortex.register({
  tx: new Transaction(),
  encryptionKey: vortexKeypair.encryptionKey,
});

// Get user's encryption key
const encryptionKey = await vortex.encryptionKey(userAddress);

// Get current merkle root
const root = await vortex.root();

// Get next commitment index
const nextIndex = await vortex.nextIndex();

// Check if nullifier is spent
const isSpent = await vortex.isNullifierSpent(nullifier);

// Batch check nullifiers
const spentStatuses = await vortex.areNullifiersSpent([nullifier1, nullifier2]);

// Get total value locked
const tvl = await vortex.tvl();

// Create ext data for transaction
const { extData, tx } = vortex.newExtData({
  tx: new Transaction(),
  recipient: '0x...',
  value: 1_000_000_000n,
  action: Action.Deposit, // or Action.Withdraw
  relayer: '0x0',
  relayerFee: 0n,
  encryptedOutput0: encryptedUtxo0,
  encryptedOutput1: encryptedUtxo1,
});

// Create proof object
const { proof, tx } = vortex.newProof({
  tx: new Transaction(),
  proofPoints: proofBytes,
  root: merkleRoot,
  publicValue: amount,
  action: Action.Deposit,
  extDataHash: extDataHash,
  inputNullifier0: nullifier0,
  inputNullifier1: nullifier1,
  outputCommitment0: commitment0,
  outputCommitment1: commitment1,
});

// Execute transaction
const { tx } = vortex.transact({
  tx: new Transaction(),
  proof,
  extData,
  deposit: coinObject,
});
```

### Transaction Functions

#### deposit()

Deposit SUI into the privacy pool.

```typescript
import { deposit, Action } from '@interest-protocol/vortex-sdk';

const transaction = await deposit({
  tx: new Transaction(), // Optional
  amount: 1_000_000_000n, // Amount in MIST
  vortex: vortexSDK,
  vortexKeypair: vortexKeypair,
  merkleTree: merkleTree,
  unspentUtxos: [], // Optional, existing UTXOs to consolidate
});

transaction.setSender(suiKeypair.toSuiAddress());
const result = await suiKeypair.signAndExecuteTransaction({
  transaction,
  client: suiClient,
});
```

**Note**: A 0.5% deposit fee (50 basis points) is automatically deducted and sent to the treasury.

#### withdraw()

Withdraw SUI from the privacy pool.

```typescript
import { withdraw } from '@interest-protocol/vortex-sdk';

const transaction = await withdraw({
  tx: new Transaction(), // Optional
  amount: 500_000_000n, // Amount to withdraw in MIST
  vortex: vortexSDK,
  vortexKeypair: vortexKeypair,
  merkleTree: merkleTree,
  unspentUtxos: unspentUtxos, // Must have sufficient balance
  recipient: '0x...', // Recipient address
  relayer: '0x0', // Relayer address (use '0x0' for no relayer)
  relayerFee: 0n, // Fee for relayer
});

transaction.setSender(suiKeypair.toSuiAddress());
const result = await suiKeypair.signAndExecuteTransaction({
  transaction,
  client: suiClient,
});
```

### Utility Functions

#### computeExtDataHash()

Compute the hash of external data for a transaction.

```typescript
import { computeExtDataHash } from '@interest-protocol/vortex-sdk';
import { fromHex } from '@mysten/sui/utils';

const extDataHash = computeExtDataHash({
  recipient: '0x...',
  value: 1_000_000_000n,
  valueSign: true, // true for deposit, false for withdraw
  relayer: '0x0',
  relayerFee: 0n,
  encryptedOutput0: fromHex(encryptedUtxo0),
  encryptedOutput1: fromHex(encryptedUtxo1),
});
```

#### parseNewCommitmentEvent()

Parse commitment events from the blockchain.

```typescript
import { parseNewCommitmentEvent } from '@interest-protocol/vortex-sdk';

const commitments = parseNewCommitmentEvent(paginatedEvents);
// Returns: Array<{ commitment: bigint, index: bigint, encryptedOutput: string }>
```

#### getUnspentUtxos()

Get all unspent UTXOs for a user.

```typescript
import { getUnspentUtxos } from '@interest-protocol/vortex-sdk';

const unspentUtxos = await getUnspentUtxos({
  commitmentEvents: paginatedEvents,
  vortexKeypair: vortexKeypair,
  vortex: vortexSDK,
});
```

### Cryptographic Functions

```typescript
import { poseidon1, poseidon2, poseidon3 } from '@interest-protocol/vortex-sdk';

// Hash single field element
const hash1 = poseidon1(value);

// Hash two field elements
const hash2 = poseidon2(value1, value2);

// Hash three field elements
const hash3 = poseidon3(value1, value2, value3);
```

## Complete Examples

### Example 1: Make a Deposit

```typescript
import {
  vortexSDK,
  VortexKeypair,
  MerkleTree,
  deposit,
  parseNewCommitmentEvent,
  getUnspentUtxos,
  MERKLE_TREE_HEIGHT,
} from '@interest-protocol/vortex-sdk';
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import { SuiClient, getFullnodeUrl } from '@mysten/sui/client';

async function makeDeposit() {
  // Setup
  const suiKeypair = Ed25519Keypair.generate();
  const suiClient = new SuiClient({ url: getFullnodeUrl('devnet') });

  // Create Vortex keypair
  const vortexKeypair = await VortexKeypair.fromSuiWallet(
    suiKeypair.toSuiAddress(),
    (message) => suiKeypair.signPersonalMessage(message)
  );

  // Fetch all commitment events
  const commitmentEvents = await suiClient.queryEvents({
    query: { MoveEventType: vortexSDK.newCommitmentEventType },
    limit: 50,
  });

  // Parse events and build merkle tree
  const commitments = parseNewCommitmentEvent(commitmentEvents);
  const merkleTree = new MerkleTree(MERKLE_TREE_HEIGHT);

  for (let i = 0; i < commitments.length; i += 2) {
    if (i + 1 < commitments.length) {
      merkleTree.insertPair(
        commitments[i].commitment,
        commitments[i + 1].commitment
      );
    }
  }

  // Get unspent UTXOs
  const unspentUtxos = await getUnspentUtxos({
    commitmentEvents,
    vortexKeypair: vortexKeypair,
    vortex: vortexSDK,
  });

  // Create deposit transaction
  const transaction = await deposit({
    amount: 2_000_000_000n, // 2 SUI
    vortex: vortexSDK,
    vortexKeypair,
    merkleTree,
    unspentUtxos,
  });

  // Sign and execute
  transaction.setSender(suiKeypair.toSuiAddress());
  const result = await suiKeypair.signAndExecuteTransaction({
    transaction,
    client: suiClient,
  });

  console.log('Deposit successful:', result.digest);
}
```

### Example 2: Make a Withdrawal

```typescript
import {
  vortexSDK,
  VortexKeypair,
  MerkleTree,
  withdraw,
  parseNewCommitmentEvent,
  getUnspentUtxos,
  MERKLE_TREE_HEIGHT,
} from '@interest-protocol/vortex-sdk';
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import { SuiClient, getFullnodeUrl } from '@mysten/sui/client';

async function makeWithdrawal() {
  // Setup (same as deposit example)
  const suiKeypair = Ed25519Keypair.generate();
  const suiClient = new SuiClient({ url: getFullnodeUrl('devnet') });

  const vortexKeypair = await VortexKeypair.fromSuiWallet(
    suiKeypair.toSuiAddress(),
    (message) => suiKeypair.signPersonalMessage(message)
  );

  // Fetch and build merkle tree (same as deposit)
  const commitmentEvents = await suiClient.queryEvents({
    query: { MoveEventType: vortexSDK.newCommitmentEventType },
    limit: 50,
  });

  const commitments = parseNewCommitmentEvent(commitmentEvents);
  const merkleTree = new MerkleTree(MERKLE_TREE_HEIGHT);

  for (let i = 0; i < commitments.length; i += 2) {
    if (i + 1 < commitments.length) {
      merkleTree.insertPair(
        commitments[i].commitment,
        commitments[i + 1].commitment
      );
    }
  }

  // Get unspent UTXOs
  const unspentUtxos = await getUnspentUtxos({
    commitmentEvents,
    vortexKeypair: vortexKeypair,
    vortex: vortexSDK,
  });

  // Create withdrawal transaction
  const transaction = await withdraw({
    amount: 500_000_000n, // 0.5 SUI
    vortex: vortexSDK,
    vortexKeypair,
    merkleTree,
    unspentUtxos,
    recipient: suiKeypair.toSuiAddress(), // Where to receive funds
    relayer: '0x0', // No relayer
    relayerFee: 0n,
  });

  // Sign and execute
  transaction.setSender(suiKeypair.toSuiAddress());
  const result = await suiKeypair.signAndExecuteTransaction({
    transaction,
    client: suiClient,
  });

  console.log('Withdrawal successful:', result.digest);
}
```

### Example 3: Check Balance

```typescript
import {
  vortexSDK,
  VortexKeypair,
  getUnspentUtxos,
  parseNewCommitmentEvent,
} from '@interest-protocol/vortex-sdk';
import { SuiClient, getFullnodeUrl } from '@mysten/sui/client';
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';

async function checkBalance() {
  const suiKeypair = Ed25519Keypair.generate();
  const suiClient = new SuiClient({ url: getFullnodeUrl('devnet') });

  const vortexKeypair = await VortexKeypair.fromSuiWallet(
    suiKeypair.toSuiAddress(),
    (message) => suiKeypair.signPersonalMessage(message)
  );

  // Fetch commitment events
  const commitmentEvents = await suiClient.queryEvents({
    query: { MoveEventType: vortexSDK.newCommitmentEventType },
    limit: 50,
  });

  // Get unspent UTXOs
  const unspentUtxos = await getUnspentUtxos({
    commitmentEvents,
    vortexKeypair: vortexKeypair,
    vortex: vortexSDK,
  });

  // Calculate total balance
  const totalBalance = unspentUtxos.reduce(
    (sum, utxo) => sum + utxo.amount,
    0n
  );

  console.log(
    `Balance: ${totalBalance} MIST (${totalBalance / 1_000_000_000n} SUI)`
  );
  console.log(`UTXOs: ${unspentUtxos.length}`);

  unspentUtxos.forEach((utxo, i) => {
    console.log(`  UTXO ${i}: ${utxo.amount} MIST at index ${utxo.index}`);
  });
}
```

### Example 4: Paginated Event Fetching

```typescript
import {
  vortexSDK,
  parseNewCommitmentEvent,
} from '@interest-protocol/vortex-sdk';
import { SuiClient, getFullnodeUrl } from '@mysten/sui/client';

async function fetchAllCommitments() {
  const suiClient = new SuiClient({ url: getFullnodeUrl('devnet') });

  let allCommitments = [];
  let cursor = null;
  let hasNextPage = true;

  while (hasNextPage) {
    const result = await suiClient.queryEvents({
      query: { MoveEventType: vortexSDK.newCommitmentEventType },
      cursor,
      limit: 50,
    });

    const commitments = parseNewCommitmentEvent(result);
    allCommitments.push(...commitments);

    hasNextPage = result.hasNextPage;
    cursor = result.nextCursor;
  }

  console.log(`Total commitments: ${allCommitments.length}`);
  return allCommitments;
}
```

## Advanced Usage

### Custom Network Configuration

```typescript
import { Vortex } from '@interest-protocol/vortex-sdk';
import { getFullnodeUrl } from '@mysten/sui/client';

// Testnet configuration
const vortexTestnet = new Vortex({
  packageId: '0x...', // Your testnet package ID
  registry: {
    objectId: '0x...', // Registry object ID
    initialSharedVersion: '1',
  },
  vortex: {
    objectId: '0x...', // Vortex pool object ID
    initialSharedVersion: '1',
  },
  fullNodeUrl: getFullnodeUrl('testnet'),
});

// Mainnet configuration
const vortexMainnet = new Vortex({
  packageId: '0x...', // Your mainnet package ID
  registry: {
    objectId: '0x...',
    initialSharedVersion: '1',
  },
  vortex: {
    objectId: '0x...',
    initialSharedVersion: '1',
  },
  fullNodeUrl: getFullnodeUrl('mainnet'),
});
```

### Encrypting Data for Recipients

```typescript
import { VortexKeypair } from '@interest-protocol/vortex-sdk';

// Sender
const senderKeypair = VortexKeypair.generate();

// Recipient
const recipientKeypair = VortexKeypair.generate();

// Encrypt UTXO data for recipient
const utxoPayload = {
  amount: 1_000_000_000n,
  blinding: 123456789n,
  index: 5n,
};

const encryptedData = VortexKeypair.encryptUtxoFor(
  utxoPayload,
  recipientKeypair.encryptionKey // Recipient's public encryption key
);

// Recipient can decrypt
const decryptedUtxo = recipientKeypair.decryptUtxo(encryptedData);
console.log(decryptedUtxo.amount); // 1_000_000_000n
```

### Working with Serialized Keypairs

```typescript
import { VortexKeypair } from '@interest-protocol/vortex-sdk';

// Serialize keypair for storage
const keypair = VortexKeypair.generate();
const serialized = keypair.toString();
// Format: 0x<64-char-pubkey><64-char-encryptionkey>

// Store in local storage / database
localStorage.setItem('vortex-keypair', serialized);

// Later, deserialize
const restored = VortexKeypair.fromString(
  localStorage.getItem('vortex-keypair')
);

console.log(restored.publicKey === keypair.publicKey); // true
```

## Constants

```typescript
import {
  BN254_FIELD_MODULUS,
  ZERO_VALUE,
  MERKLE_TREE_HEIGHT,
  ROOT_HISTORY_SIZE,
  VORTEX_PACKAGE_ID,
  VORTEX_POOL_OBJECT_ID,
  REGISTRY_OBJECT_ID,
  INITIAL_SHARED_VERSION,
  DEPOSIT_FEE_IN_BASIS_POINTS,
  BASIS_POINTS,
  TREASURY_ADDRESS,
} from '@interest-protocol/vortex-sdk';

// BN254 field modulus for zkSNARK operations
console.log(BN254_FIELD_MODULUS);

// Merkle tree configuration
console.log(MERKLE_TREE_HEIGHT); // 26
console.log(ROOT_HISTORY_SIZE); // 100

// Fee configuration
console.log(DEPOSIT_FEE_IN_BASIS_POINTS); // 50 (0.5%)
console.log(BASIS_POINTS); // 10000
```

## Error Handling

```typescript
import { deposit } from '@interest-protocol/vortex-sdk';

try {
  const transaction = await deposit({
    amount: 1_000_000_000n,
    vortex: vortexSDK,
    vortexKeypair,
    merkleTree,
    unspentUtxos,
  });

  // Execute transaction...
} catch (error) {
  if (error.message.includes('Proof verification failed')) {
    console.error('Invalid proof generated');
  } else if (error.message.includes('Merkle tree is full')) {
    console.error('Privacy pool at capacity');
  } else if (error.message.includes('insufficient balance')) {
    console.error('Not enough funds in UTXOs');
  } else {
    console.error('Transaction failed:', error);
  }
}
```

## Common Error Codes

| Code | Error                         | Description                  |
| ---- | ----------------------------- | ---------------------------- |
| 0    | Invalid allowed deposit value | Deposit amount not valid     |
| 1    | Merkle tree overflow          | Tree at maximum capacity     |
| 2    | Proof root not known          | Root not in history          |
| 3    | Invalid proof                 | ZK proof verification failed |
| 4    | Invalid proof vortex          | Proof format incorrect       |
| 7    | Invalid ext data hash         | External data hash mismatch  |
| 8    | Invalid ext data value        | External data value invalid  |
| 9    | Invalid public value          | Public amount incorrect      |
| 10   | Nullifier already spent       | Double-spend attempt         |
| 11   | Invalid deposit value         | Deposit amount incorrect     |

## Network Configuration

### Devnet (Default)

```typescript
import { vortexSDK } from '@interest-protocol/vortex-sdk';

// Pre-configured for devnet
const root = await vortexSDK.root();
```

### Testnet

Update constants or create custom instance:

```typescript
import { Vortex } from '@interest-protocol/vortex-sdk';
import { getFullnodeUrl } from '@mysten/sui/client';

const vortex = new Vortex({
  packageId:
    '0x12bef57977215e40b6c2efd5a17681fd96d4febec235b88466f6708b3422e796',
  registry: {
    objectId: '0x...', // Testnet registry
    initialSharedVersion: '1',
  },
  vortex: {
    objectId: '0x...', // Testnet pool
    initialSharedVersion: '1',
  },
  fullNodeUrl: getFullnodeUrl('testnet'),
});
```

## Best Practices

1. **Always verify proofs locally** before submitting transactions
2. **Use pagination** when fetching events for large datasets
3. **Keep keypairs secure** - never expose private keys
4. **Consolidate UTXOs** periodically for better performance
5. **Monitor gas costs** - complex proofs require more gas
6. **Test on devnet/testnet** before mainnet deployment
7. **Handle errors gracefully** with proper user feedback
8. **Cache merkle trees** to avoid rebuilding for each transaction
9. **Validate amounts** are within field modulus bounds
10. **Use relayers** for enhanced privacy (optional)

## Security Considerations

- **Private keys**: VortexKeypair private keys are as sensitive as wallet keys
- **Encryption keys**: Store and transmit encryption keys securely
- **UTXOs**: Never reuse nullifiers (prevented by protocol)
- **Amounts**: Always within BN254 field bounds (< 2^248)
- **Merkle proofs**: Generated fresh for each transaction
- **Transaction privacy**: Deposits/withdrawals are unlinkable with proper usage
- **Relayers**: Using relayers enhances privacy but requires trust

## TypeScript Types

```typescript
import type {
  ConstructorArgs,
  SharedObjectData,
  RegisterArgs,
  NewExtDataArgs,
  NewProofArgs,
  TransactArgs,
  DepositArgs,
  WithdrawArgs,
  ExtDataHashArgs,
  Proof,
  Action,
  ParsedCommitmentEvent,
} from '@interest-protocol/vortex-sdk';
```

## License

MIT

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## Support

- Documentation: [https://docs.vortexfi.xyz](https://docs.vortexfi.xyz)
- Discord: [https://discord.gg/vortex](https://discord.gg/vortex)
- GitHub Issues: [https://github.com/interest-protocol/vortex-v2/issues](https://github.com/interest-protocol/vortex-v2/issues)

## Related Packages

- `@mysten/sui` - Sui TypeScript SDK
- `@interest-protocol/sui-core-sdk` - Core utilities for Sui
- `@noble/hashes` - Cryptographic hash functions
- `@noble/ciphers` - Encryption primitives
