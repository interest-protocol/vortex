# Vortex Protocol - Development Guidelines

## Project Overview

Vortex is a privacy protocol for Sui blockchain that breaks the on-chain link between deposit and withdrawal addresses using zero-knowledge proofs.

**Directory Structure:**
- `contracts/` - Sui Move smart contracts (core, swap, test-coins)
- `circuit/` - Rust zkSNARK circuit (Groth16 with Arkworks)
- `indexer/` - Rust Sui indexer service
- `api/` - TypeScript REST API (Bun + Hono)

---

## Agent Behavior

### Task Execution

1. **Understand first** - Read relevant files before making changes
2. **Verify assumptions** - Check existing patterns in the codebase
3. **Incremental changes** - Small, testable changes over large refactors
4. **Validate after changes** - Run build and tests before completing

### Decision Framework

**Ask for clarification when:**
- Requirements are ambiguous
- Multiple valid approaches exist with significant tradeoffs
- Changes affect public APIs or interfaces
- Unsure about business logic intent

**Proceed autonomously when:**
- Task is well-defined
- Following established codebase patterns
- Changes are easily reversible
- Standard refactoring or bug fixes

### Change Principles

1. **Minimize blast radius** - Touch only what's necessary
2. **Preserve patterns** - Match existing code style and architecture
3. **No surprise dependencies** - Discuss before adding new packages
4. **Single responsibility** - One logical change per commit

---

## Universal Rules

### Code Quality Standards

Code must be self-documenting through clear naming. No comments except for complex algorithms or non-obvious business logic.

| Forbidden | Required Alternative |
|-----------|---------------------|
| Comments explaining what code does | Clear variable and function names |
| `// TODO: implement later` | Complete implementation or don't commit |
| Magic numbers | Named constants |
| `any` type (TypeScript) | Proper types or `unknown` with narrowing |
| `.unwrap()` in production (Rust) | `?` operator or `.expect()` with context |
| Commented-out code | Delete it, git has history |
| Re-exports at end of file | Export inline on declarations |

### Naming Conventions

| Element | Rust | TypeScript | Move |
|---------|------|------------|------|
| Constants | `UPPER_SNAKE_CASE` | `UPPER_SNAKE_CASE` | `UPPER_SNAKE_CASE` |
| Functions | `snake_case` | `camelCase` | `snake_case` |
| Types/Structs | `PascalCase` | `PascalCase` | `PascalCase` |
| Variables | `snake_case` | `camelCase` | `snake_case` |
| Files | `snake_case.rs` | `kebab-case.ts` | `snake_case.move` |
| Error Macros (Move) | - | - | `snake_case!()` |

### Error Handling

Every error must be handled. No silent failures.

```rust
// Rust: Use ? operator with context
let value = operation().context("failed to perform operation")?;
```

```typescript
// TypeScript: Use invariant for assertions
import invariant from 'tiny-invariant';
invariant(user, 'User must exist');
```

```move
// Move: Use assert! with error macros
assert!(bytes.length() == 32, vortex::errors::invalid_length!());
```

### Implementation Standards

Write optimal code on the first attempt. Use canonical solutions.

1. **Parallel when independent** - Don't await sequentially if operations are independent
   ```typescript
   // DO: parallel fetches
   const [a, b, c] = await Promise.all([fetchA(), fetchB(), fetchC()]);

   // DON'T: sequential when independent
   const a = await fetchA();
   const b = await fetchB();
   ```

2. **Actionable error messages** - Include what failed AND what's expected
   ```typescript
   // DO: actionable
   invariant(bytes.length === 32, `Address must be 32 bytes, got ${bytes.length}`);

   // DON'T: vague
   invariant(bytes.length === 32, 'Invalid address');
   ```

3. **Fail fast** - Validate inputs at function entry
   ```typescript
   const processOrder = (order: Order) => {
     invariant(order.items.length > 0, 'Order must have items');
     invariant(order.total > 0, 'Total must be positive');
     // ... rest of logic
   };
   ```

4. **Pre-compute constants** - Move computation out of hot paths
   ```typescript
   // DO: compute once at module load
   const BCS_BYTES = bcs.vector(bcs.u8());
   const fn = (data: Uint8Array) => tx.pure(BCS_BYTES.serialize(data));

   // DON'T: recompute on every call
   const fn = (data: Uint8Array) => tx.pure(bcs.vector(bcs.u8()).serialize(data));
   ```

---

## Sui Move Guidelines

### Package Configuration

- Use Move 2024 Edition (`edition = "2024.beta"`)
- Sui, Bridge, MoveStdlib, SuiSystem are implicitly imported (Sui 1.45+)
- Prefix named addresses with project name to avoid conflicts

### Module Structure

```move
module vortex::pool;

// === Imports ===
use vortex::errors;

// === Constants ===
const MAX_TREE_DEPTH: u64 = 20;

// === Structs (OTW → Witness → Cap → Keys → Owned → Shared → Data → Events) ===
public struct POOL has drop {}
public struct AdminCap has key, store { id: UID }
public struct MerkleTreeKey() has copy, drop, store;
public struct Pool<phantom CoinType> has key { id: UID, balance: Balance<CoinType> }
public struct DepositEvent<phantom CoinType> has copy, drop { amount: u64 }

// === Initialization ===
fun init(otw: POOL, ctx: &mut TxContext) { ... }

// === Public Functions ===

// === Package Functions ===

// === Private Functions ===

// === Macros ===

// === Test Only ===

// === Aliases ===
```

### Error Handling Pattern

Errors are defined as macros in a dedicated errors module:

```move
// In errors.move
#[test_only] const EInvalidLength: u64 = 0;
public(package) macro fun invalid_length(): u64 { 0 }

#[test_only] const EPoolAlreadyExists: u64 = 1;
public(package) macro fun pool_already_exists(): u64 { 1 }

// Usage in other modules - fully qualified, no import needed
assert!(bytes.length() == 32, vortex::errors::invalid_length!());
```

### Struct Conventions

```move
// One-time witness (OTW): ALL_CAPS, matches module name
public struct VORTEX has drop {}

// Capability: ends with Cap
public struct AdminCap has key, store { id: UID }

// Dynamic field key: positional with Key suffix
public struct MerkleTreeKey() has copy, drop, store;

// Shared object with phantom type: has key
public struct Vortex<phantom CoinType> has key { id: UID, ... }

// Event: past tense naming, phantom for coin types
public struct NewCommitment<phantom CoinType> has copy, drop { ... }
```

### Syntax Preferences

Use dot syntax (method calls) over module function calls:

```move
// DO
self.id.delete();
coin.split(amount, ctx);
ctx.sender();
vec[0];
b"hello".to_string();

// DON'T
object::delete(self.id);
coin::split(&mut coin, amount, ctx);
tx_context::sender(ctx);
vector::borrow(&vec, 0);
string::utf8(b"hello");
```

### Function Ordering

1. `init` (if present)
2. `public` functions (for composability)
3. `public(package)` functions
4. Private functions
5. Macros
6. Test-only functions
7. Use fun aliases (at end)

### Parameter Ordering

1. `self` (if method)
2. Shared objects (mut before immut)
3. Capabilities
4. Owned structs
5. Primitive values
6. `TxContext` (always last, even if unused - for upgrade compatibility)

### Testing

```move
#[test]
fun deposit_increases_balance() {
    // Test implementation
}

#[test, expected_failure(abort_code = vortex::errors::EInsufficientBalance)]
fun withdraw_fails_with_insufficient_balance() {
    // Test that triggers expected failure
}
```

### Commands

```bash
# Build (run in Move.toml directory)
sui move build

# After initial build, skip fetching deps
sui move build --skip-fetch-latest-git-deps

# Run tests
sui move test

# Format
bunx prettier-move -c *.move --write
```

---

## Rust Guidelines

### Code Style

- Express intent through naming and structure
- Keep methods small with single responsibility
- Minimize state and side effects
- Eliminate duplication ruthlessly

### Error Handling

```rust
use anyhow::{Context, Result};

// Production: Use ? operator with context
fn process_event(event: &Event) -> Result<()> {
    let data = parse_event(event)
        .context("failed to parse event")?;

    db.insert(&data)
        .await
        .context("failed to insert into database")?;

    Ok(())
}

// Only use .expect() for true invariants at startup
let config = Config::from_env()
    .expect("CONFIG must be set at startup");
```

### Preferred Crates

| Purpose | Crate |
|---------|-------|
| Async runtime | `tokio` |
| HTTP server | `axum` |
| Serialization | `serde`, `serde_json` |
| Error handling | `anyhow`, `thiserror` |
| Logging | `tracing` |
| Database | `mongodb` |
| CLI parsing | `clap` |
| ZK circuits | `ark-*` (Arkworks) |

### Indexer Patterns

Handler pattern with `Processor` trait:

```rust
pub struct NewCommitmentHandler;

#[async_trait]
impl Processor for NewCommitmentHandler {
    const NAME: &'static str = "new_commitment";

    async fn process(&self, checkpoint: Arc<Checkpoint>) -> Result<()> {
        let events = process_vortex_events::<NewCommitmentEvent>(
            &checkpoint,
            &self.package_id,
            "NewCommitment",
        )?;
        // ... handle events
        Ok(())
    }
}
```

Use the `impl_mongo_handler!` macro for MongoDB persistence:
```rust
impl_mongo_handler!(NewCommitmentHandler, NewCommitmentDocument, "new_commitments");
```

### Testing

```rust
// Integration tests in tests/ directory
#[tokio::test]
async fn processes_deposit_event() {
    let db = setup_test_db().await;
    let event = create_test_event();

    let result = handler.process(&event).await;

    assert!(result.is_ok());
    assert_eq!(db.count().await, 1);
}
```

### Commands

```bash
cargo check                      # Fast type checking
cargo build --release            # Optimized build
cargo test                       # Run tests
cargo clippy -- -D warnings      # Lint (must pass with zero warnings)
cargo fmt                        # Format
cargo audit                      # Security audit
```

---

## TypeScript Guidelines (API)

### Tech Stack

| Component | Technology |
|-----------|------------|
| Runtime | Bun |
| Framework | Hono |
| Database | MongoDB (native driver) |
| Cache | Redis (ioredis) |
| Validation | Zod |
| Logging | Pino |
| Sui SDK | @mysten/sui |

### Code Style

```typescript
// Use arrow functions exclusively
export const getUser = async (id: string): Promise<User> => {
    // implementation
};

// Functions with >2 params use object parameter
export const createAccount = async ({
    hashedSecret,
    owner,
    txDigest,
}: CreateAccountParams): Promise<Account> => {
    // implementation
};

// Use const by default, let only when reassignment needed
const users = await repository.findAll();
let retryCount = 0;

// Destructure when accessing multiple properties
const { id, name, email } = user;
```

### Type Safety

```typescript
// Never use 'any' - use 'unknown' and narrow
const parseJson = (input: unknown): User => {
    const parsed = JSON.parse(input as string);
    return UserSchema.parse(parsed);
};

// Always annotate return types
export const findById = async (id: string): Promise<User | null> => {
    return db.collection<User>('users').findOne({ _id: id });
};

// Use discriminated unions for state
type TransactionStatus =
    | { status: 'pending' }
    | { status: 'confirmed'; digest: string }
    | { status: 'failed'; error: string };
```

### Imports

```typescript
// Order: built-ins → external → internal (@/)
import { randomBytes } from 'node:crypto';

import { Hono } from 'hono';
import { z } from 'zod';

import { env } from '@/config/env.ts';
import type { AppBindings } from '@/types/index.ts';
import { logger } from '@/utils/logger.ts';
```

### Architecture Pattern

```
Request → Middleware → Handler → Service → Repository → Database
```

**Factory Functions** (preferred over classes):

```typescript
// Repository: data access only
export const createAccountsRepository = (db: Db): AccountsRepository => ({
    findByHashedSecret: async (hashedSecret: string) =>
        db.collection<AccountDocument>('accounts')
            .find({ hashed_secret: hashedSecret })
            .toArray(),

    insert: async (doc: AccountDocument) =>
        db.collection<AccountDocument>('accounts').insertOne(doc),
});

// Service: business logic
export const createAccountsService = (
    repository: AccountsRepository,
    suiService: SuiService,
): AccountsService => ({
    createAccount: async (params: CreateAccountParams) => {
        // Business logic here
        const result = await suiService.sponsorAndExecute(tx);
        return repository.insert(mapToDocument(result));
    },
});
```

**Handlers**: HTTP concerns only, wrapped with error handler
```typescript
export const getAccounts = withErrorHandler(
    async (c: Context<AppBindings>): Promise<Response> => {
        const accountsService = c.get('accountsService');
        const validation = validateQuery(c, GetAccountsSchema);
        if (!validation.success) return validation.response;

        const data = await accountsService.findByHashedSecret(validation.data.hashed_secret);
        return c.json({ success: true, data });
    },
    'Failed to get accounts',
);
```

### Route Structure

Each domain folder follows this pattern:
```
routes/v1/accounts/
├── index.ts      # Route definitions (minimal, wires handlers to routes)
├── handlers.ts   # Handler functions
├── schema.ts     # Zod validation schemas
├── types.ts      # API response types
└── mappers.ts    # DB → API transformations
```

### Validation Pattern

```typescript
// In handlers
const validation = validateQuery(c, GetAccountsSchema);
if (!validation.success) return validation.response;

// validation.data is now typed correctly
const data = await service.find(validation.data.hashed_secret);
```

### Commands

```bash
bun run dev                    # Development
bun run typecheck              # Type check (must pass)
bun run lint                   # Lint (zero warnings)
bun run format                 # Format code
bun test                       # Run tests
bun run build                  # Production build
```

---

## Quality Gates

### Pre-Commit Checklist

| Language | Command | Requirement |
|----------|---------|-------------|
| Rust | `cargo check` | Zero errors |
| Rust | `cargo clippy -- -D warnings` | Zero warnings |
| Rust | `cargo test` | All pass |
| Rust | `cargo fmt --check` | Formatted |
| TypeScript | `bun run typecheck` | Zero errors |
| TypeScript | `bun run lint` | Zero warnings |
| TypeScript | `bun run format --check` | Formatted |
| TypeScript | `bun test` | All pass |
| Move | `sui move build` | Zero errors |
| Move | `sui move test` | All pass |

---

## Git Workflow

### Commit Format

```
emoji type(scope): subject
```

Always use emoji at the start. Subject must be lowercase.

| Type | Emoji | Description |
|------|-------|-------------|
| `feat` | ✨ | New feature |
| `fix` | 🐛 | Bug fix |
| `docs` | 📝 | Documentation only |
| `style` | 🎨 | Formatting, no code change |
| `refactor` | ♻️ | Code change without feature/fix |
| `perf` | ⚡ | Performance improvement |
| `test` | ✅ | Adding or updating tests |
| `build` | 📦 | Build system or dependencies |
| `ci` | 👷 | CI configuration |
| `chore` | 🔧 | Other changes |
| `revert` | ⏪ | Revert previous commit |

**Examples:**
```
✨ feat(api): add merkle proof endpoint
🐛 fix(indexer): handle null checkpoint gracefully
♻️ refactor(contracts): extract commitment validation
⚡ perf(api): add database indexes for queries
```

**Rules:**
- Do NOT add "Generated with Claude" or "Co-Authored-By: Claude"
- Never mix refactoring with feature/bug work in same commit
- One logical change per commit
- Prefer small, frequent commits over large ones

---

## Project Structure Reference

### API Directory Layout

```
api/src/
├── index.ts                # Entry point, middleware chain
├── config/
│   └── env.ts              # Zod environment validation
├── constants/
│   └── index.ts            # Pagination, defaults
├── db/
│   ├── mongodb.ts          # Connection, index creation
│   ├── redis.ts            # Redis client
│   └── collections/        # Collection types
├── middleware/
│   ├── database.ts         # DI: injects repos and services
│   ├── rate-limit.ts       # Redis/memory rate limiter
│   ├── api-key.ts          # API key validation
│   ├── cors.ts             # CORS handling
│   └── error.ts            # Global error handler
├── repositories/           # Data access layer
├── services/               # Business logic layer
├── routes/v1/              # Route definitions
├── types/
│   └── index.ts            # AppBindings, shared types
└── utils/
    ├── validation.ts       # Request validation helpers
    ├── handler.ts          # Error wrapping
    ├── schemas.ts          # Shared Zod schemas
    ├── logger.ts           # Pino logger
    └── hex.ts              # Hex utilities
```

### Indexer Workspace Layout

```
indexer/
├── Cargo.toml              # Workspace root
└── crates/
    ├── indexer/            # Main binary
    │   ├── src/
    │   │   ├── main.rs
    │   │   ├── handlers/   # Event handlers
    │   │   └── db/         # MongoDB operations
    │   └── tests/          # Integration tests
    └── schema/             # Shared data models
```

### Contracts Layout

```
contracts/
├── core/                   # Main protocol
│   ├── Move.toml
│   └── sources/
│       ├── vortex.move     # Pool and proof verification
│       ├── account.move    # User accounts
│       ├── merkle_tree.move
│       ├── proof.move
│       ├── events.move
│       ├── constants.move
│       └── errors.move
├── swap/                   # Swap functionality
└── test-coins/             # Test tokens
```

---

## Anti-Patterns

### Code Smells

| Don't | Do |
|-------|-----|
| One-time-use variables | Inline values |
| `arr[arr.length - 1]` | `arr.at(-1)` |
| Sequential awaits for independent ops | `Promise.all()` |
| Dead code, unused exports | Delete completely |
| Backwards-compatibility hacks | Clean removal |
| Over-engineering | Minimum viable solution |

### Architectural Violations

| Don't | Do |
|-------|-----|
| Business logic in handlers | Use services |
| Raw DB access in services | Use repositories |
| Global state/singletons | Dependency injection (factory functions) |
| Mixing refactoring with features | Separate commits |
| Comments explaining what code does | Self-documenting names |
| Classes for stateless operations | Factory functions |

---

## Security

### Never Commit

- Private keys, secrets, API keys
- `.env` files with real credentials
- Hardcoded production addresses

### Input Validation

- Validate all external input at system boundaries
- Use Zod schemas for API requests
- Use Move `assert!` with error macros for on-chain validation
- Never trust client-provided data

### Cryptographic Operations

- Use established libraries (Arkworks for circuits)
- Never implement custom cryptography
- Validate proofs on-chain, never skip verification
