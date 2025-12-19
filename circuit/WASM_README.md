# Vortex WASM Proof Generator

WebAssembly module for generating zero-knowledge proofs for privacy-preserving transactions on Sui blockchain.

## Features

- ✅ **Client-side proof generation** - Generate proofs in browser or Node.js
- ✅ **Full circuit implementation** - 2-input, 2-output transaction circuit
- ✅ **Optimized for production** - Compressed serialization, efficient constraints
- ✅ **Type-safe** - Full TypeScript definitions included
- ✅ **Multiple targets** - Supports Node.js, Web, and bundlers

## Installation

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack

# Add wasm32 target
rustup target add wasm32-unknown-unknown
```

### Build

```bash
# Build all targets
./build-wasm.sh

# Or build specific target
wasm-pack build --target nodejs --out-dir pkg/nodejs --release
wasm-pack build --target web --out-dir pkg/web --release
wasm-pack build --target bundler --out-dir pkg/bundler --release
```

## Usage

### Node.js

```javascript
const { prove, verify, generate_test_keys } = require('./pkg/nodejs');

// Generate test keys (development only - use trusted setup for production)
const keys = JSON.parse(generate_test_keys());
const { proving_key, verifying_key } = keys;

// Prepare circuit inputs
const input = {
  // Public inputs
  root: '12345678901234567890123456789012345678901234567890123456789012345',
  publicAmount: '1000000000', // 1 SUI in MIST
  extDataHash: '0',
  inputNullifier1:
    '11111111111111111111111111111111111111111111111111111111111111111',
  inputNullifier2: '0', // Zero for unused input
  outputCommitment1:
    '22222222222222222222222222222222222222222222222222222222222222222',
  outputCommitment2:
    '33333333333333333333333333333333333333333333333333333333333333333',

  // Private inputs - Input UTXOs
  inPrivateKey1: '12345',
  inPrivateKey2: '0',
  inAmount1: '1000000000',
  inAmount2: '0',
  inBlinding1: '999',
  inBlinding2: '0',
  inPathIndex1: '5',
  inPathIndex2: '0',

  // Merkle paths (26 levels)
  merklePath1: Array(26).fill(['0', '0']),
  merklePath2: Array(26).fill(['0', '0']),

  // Private inputs - Output UTXOs
  outPublicKey1: '54321',
  outPublicKey2: '0',
  outAmount1: '500000000',
  outAmount2: '500000000',
  outBlinding1: '888',
  outBlinding2: '777',
};

// Generate proof
const proofJson = prove(JSON.stringify(input), proving_key);
const proof = JSON.parse(proofJson);

console.log('Proof generated:', {
  proofA: proof.proofA.length + ' bytes',
  proofB: proof.proofB.length + ' bytes',
  proofC: proof.proofC.length + ' bytes',
  publicInputs: proof.publicInputs.length,
});

// Verify proof (optional - chain will verify)
const isValid = verify(proofJson, verifying_key);
console.log('Proof valid:', isValid === 'true');
```

### Web Browser

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Vortex Proof Generator</title>
  </head>
  <body>
    <h1>Generate Privacy Proof</h1>
    <button id="generateProof">Generate Proof</button>
    <pre id="output"></pre>

    <script type="module">
      import init, { prove, generate_test_keys } from './pkg/web/vortex.js';

      async function run() {
        // Initialize WASM module
        await init();

        document.getElementById('generateProof').onclick = async () => {
          const output = document.getElementById('output');
          output.textContent = 'Generating keys...';

          // Generate test keys
          const keys = JSON.parse(generate_test_keys());

          output.textContent = 'Generating proof...';

          // Prepare inputs
          const input = {
            root: '0',
            publicAmount: '1000000000',
            extDataHash: '0',
            inputNullifier1: '0',
            inputNullifier2: '0',
            outputCommitment1: '0',
            outputCommitment2: '0',
            inPrivateKey1: '12345',
            inPrivateKey2: '0',
            inAmount1: '1000000000',
            inAmount2: '0',
            inBlinding1: '999',
            inBlinding2: '0',
            inPathIndex1: '0',
            inPathIndex2: '0',
            merklePath1: Array(26).fill(['0', '0']),
            merklePath2: Array(26).fill(['0', '0']),
            outPublicKey1: '54321',
            outPublicKey2: '0',
            outAmount1: '500000000',
            outAmount2: '500000000',
            outBlinding1: '888',
            outBlinding2: '777',
          };

          try {
            const proofJson = prove(JSON.stringify(input), keys.proving_key);
            const proof = JSON.parse(proofJson);

            output.textContent = JSON.stringify(proof, null, 2);
            console.log('Proof generated successfully!', proof);
          } catch (e) {
            output.textContent = 'Error: ' + e;
            console.error('Proof generation failed:', e);
          }
        };
      }

      run();
    </script>
  </body>
</html>
```

### TypeScript (with bundler)

```typescript
import init, { prove, verify, generate_test_keys } from './pkg/bundler';

interface CircuitInput {
  root: string;
  publicAmount: string;
  extDataHash: string;
  inputNullifier1: string;
  inputNullifier2: string;
  outputCommitment1: string;
  outputCommitment2: string;
  inPrivateKey1: string;
  inPrivateKey2: string;
  inAmount1: string;
  inAmount2: string;
  inBlinding1: string;
  inBlinding2: string;
  inPathIndex1: string;
  inPathIndex2: string;
  merklePath1: [string, string][];
  merklePath2: [string, string][];
  outPublicKey1: string;
  outPublicKey2: string;
  outAmount1: string;
  outAmount2: string;
  outBlinding1: string;
  outBlinding2: string;
}

interface ProofOutput {
  proofA: number[];
  proofB: number[];
  proofC: number[];
  publicInputs: string[];
}

async function generateProof(
  input: CircuitInput,
  provingKey: string
): Promise<ProofOutput> {
  // Initialize WASM (only needed once)
  await init();

  const proofJson = prove(JSON.stringify(input), provingKey);
  return JSON.parse(proofJson);
}

// Usage
const input: CircuitInput = {
  // ... fill in values
};

const keys = JSON.parse(generate_test_keys());
const proof = await generateProof(input, keys.proving_key);
console.log('Proof:', proof);
```

## API Reference

### `prove(input_json: string, proving_key_hex: string): string`

Generates a zero-knowledge proof for a transaction.

**Parameters:**

- `input_json`: JSON string with all circuit inputs (see CircuitInput interface)
- `proving_key_hex`: Hex-encoded proving key from trusted setup

**Returns:** JSON string with ProofOutput structure

**Throws:** Error if inputs are invalid or proof generation fails

### `verify(proof_json: string, verifying_key_hex: string): string`

Verifies a proof (useful for testing before submitting to chain).

**Parameters:**

- `proof_json`: JSON string from `prove()` output
- `verifying_key_hex`: Hex-encoded verifying key

**Returns:** String "true" or "false"

### `generate_test_keys(): string`

Generates deterministic test keys for development. **DO NOT USE IN PRODUCTION**.

**Returns:** JSON string with `proving_key` and `verifying_key` (hex-encoded)

## Circuit Inputs

### Public Inputs (visible on-chain)

- `root`: Merkle tree root (bigint string)
- `publicAmount`: Net value added/removed from pool (bigint string)
- `extDataHash`: Hash of external data (recipient, fees, etc.)
- `inputNullifier1`, `inputNullifier2`: Nullifiers for spent UTXOs
- `outputCommitment1`, `outputCommitment2`: Commitments for new UTXOs

### Private Inputs (never revealed)

**Input UTXOs:**

- `inPrivateKey1`, `inPrivateKey2`: Private keys (bigint strings)
- `inAmount1`, `inAmount2`: Input amounts (bigint strings)
- `inBlinding1`, `inBlinding2`: Blinding factors for privacy
- `inPathIndex1`, `inPathIndex2`: Merkle tree indices
- `merklePath1`, `merklePath2`: Arrays of 26 [left, right] hash pairs

**Output UTXOs:**

- `outPublicKey1`, `outPublicKey2`: Recipient public keys
- `outAmount1`, `outAmount2`: Output amounts
- `outBlinding1`, `outBlinding2`: Blinding factors

## Production Deployment

### Key Generation

**DO NOT use `generate_test_keys()` in production!**

For production, use a multi-party computation (MPC) trusted setup ceremony:

```bash
# Generate secure keys using Hermez Powers of Tau
# See: https://github.com/iden3/snarkjs#7-prepare-phase-2

# 1. Get Powers of Tau from ceremony (pot28_final.ptau)
# 2. Generate circuit-specific keys
snarkjs groth16 setup circuit.r1cs pot28_final.ptau circuit_0000.zkey

# 3. Contribute to phase 2 ceremony
snarkjs zkey contribute circuit_0000.zkey circuit_0001.zkey --name="Contributor 1"

# 4. Export verification key
snarkjs zkey export verificationkey circuit_final.zkey verification_key.json

# 5. Convert to hex for WASM
cat circuit_final.zkey | xxd -p | tr -d '\n' > proving_key.hex
cat verification_key.json | jq -r '.vk_alpha_1 + .vk_beta_2 + ...' > verification_key.hex
```

### Performance Tips

1. **Pre-load keys**: Load proving key once and reuse
2. **Worker threads**: Run proof generation in Web Worker to avoid blocking UI
3. **Batch processing**: Generate multiple proofs in parallel if needed
4. **Memory**: Each proof generation requires ~100-200MB RAM
5. **Time**: Expect 5-15 seconds per proof (varies by device)

### Example Worker Setup

```javascript
// worker.js
importScripts('./pkg/web/vortex.js');

self.onmessage = async (e) => {
  const { input, provingKey } = e.data;

  try {
    await wasm_bindgen('./pkg/web/vortex_bg.wasm');
    const proof = wasm_bindgen.prove(JSON.stringify(input), provingKey);
    self.postMessage({ success: true, proof });
  } catch (error) {
    self.postMessage({ success: false, error: error.message });
  }
};

// main.js
const worker = new Worker('worker.js');
worker.postMessage({ input, provingKey });
worker.onmessage = (e) => {
  if (e.data.success) {
    console.log('Proof:', e.data.proof);
  } else {
    console.error('Error:', e.data.error);
  }
};
```

## Integration with Sui Move

The proof output format is designed for direct use with the Sui Move contract:

```rust
// Move contract usage
public fun transact(
    self: &mut Vortex,
    proof: Proof,
    ext_data: ExtData,
    // ...
) {
    // Proof struct matches WASM output format
    let proof = vortex_proof::new(
        proof_a,           // From proof.proofA
        proof_b,           // From proof.proofB
        proof_c,           // From proof.proofC
        root,              // From proof.publicInputs[0]
        input_nullifier1,  // From proof.publicInputs[3]
        input_nullifier2,  // From proof.publicInputs[4]
        output_commitment1,// From proof.publicInputs[5]
        output_commitment2,// From proof.publicInputs[6]
        public_value,      // From proof.publicInputs[1]
        ext_data_hash,     // From proof.publicInputs[2]
    );
}
```

## Troubleshooting

### "Module not found" error

- Ensure you've run `build-wasm.sh`
- Check the import path matches your target (nodejs/web/bundler)

### "Out of memory" error

- Reduce batch size (generate one proof at a time)
- Increase Node.js heap: `node --max-old-space-size=4096 script.js`

### "Invalid input" error

- All numeric strings must be valid bigints
- Merkle paths must have exactly 26 levels
- Path indices must be < 2^26

### Slow proof generation

- First proof is slower (JIT compilation)
- Use `--release` build for production
- Consider using native Rust for server-side generation

## Security Notes

1. **Proving key security**: Store proving keys securely, they're large (~100MB)
2. **Random number generation**: WASM uses ChaCha20Rng with entropy from the environment
3. **Side-channel attacks**: Browser environments may be vulnerable to timing attacks
4. **Input validation**: Always validate inputs before proof generation
5. **Key ceremony**: Use multi-party trusted setup for production keys

## License

MIT

## Support

For issues and questions:

- GitHub Issues: [vortex-privacy/issues]
- Discord: [vortex-privacy]
- Documentation: [docs.vortex.fi]
