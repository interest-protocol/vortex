# Vortex Protocol - Development Guidelines

## Project Overview

Vortex is a privacy protocol for Sui blockchain that breaks the on-chain link between deposit and withdrawal addresses using zero-knowledge proofs. The codebase consists of:

- **`contracts/`** - Sui Move smart contracts (core, swap, test-coins)
- **`circuit/`** - Rust zkSNARK circuit (Groth16 with Arkworks)
- **`indexer/`** - Rust Sui indexer service
- **`api/`** - TypeScript REST API (Bun + Hono)

---

## Universal Rules

### Code Quality Standards

Code must be self-documenting through clear naming. No comments except for complex algorithms or non-obvious business logic. No TODOs, FIXMEs, or placeholder code.

| Forbidden | Required Alternative |
|-----------|---------------------|
| Comments explaining what code does | Clear variable and function names |
| `// TODO: implement later` | Complete implementation or don't commit |
| Magic numbers | Named constants |
| `any` type (TypeScript) | Proper types or `unknown` with narrowing |
| `.unwrap()` in production (Rust) | `?` operator or `.expect()` with context |

### Naming Conventions

| Element | Rust | TypeScript | Move |
|---------|------|------------|------|
| Constants | `UPPER_SNAKE_CASE` | `UPPER_SNAKE_CASE` | `UPPER_SNAKE_CASE` |
| Functions | `snake_case` | `camelCase` | `snake_case` |
| Types/Structs | `PascalCase` | `PascalCase` | `PascalCase` |
| Variables | `snake_case` | `camelCase` | `snake_case` |
| Files | `snake_case.rs` | `kebab-case.ts` | `snake_case.move` |
| Error Constants (Move) | - | - | `EPascalCase` |

### Error Handling

Every error must be handled. No silent failures.

```rust
// Rust: Use ? operator, never unwrap
let value = operation().context("failed to perform operation")?;
```

```typescript
// TypeScript: Use invariant for assertions
import invariant from 'tiny-invariant';
invariant(user, 'User must exist');
```

```move
// Move: Use assert! with error constants
assert!(balance >= amount, EInsufficientBalance);
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

use sui::coin::Coin;
use sui::balance::Balance;

// Error constants (EPascalCase)
const EInsufficientBalance: u64 = 0;
const EInvalidProof: u64 = 1;

// Regular constants (UPPER_SNAKE_CASE)
const MAX_TREE_DEPTH: u64 = 20;

// Struct ordering: OTW → Witness → Cap → Keys → Owned → Shared → Data → Events
public struct POOL has drop {}
public struct AdminCap has key, store { id: UID }
public struct Pool has key { id: UID, balance: Balance<SUI> }
public struct DepositEvent has copy, drop { amount: u64 }
```

### Syntax Preferences

Use dot syntax (method calls) over module function calls:

```move
// Preferred
self.id.delete();
coin.split(amount);
ctx.sender();
vec[0];
b"hello".to_string();

// Avoid
object::delete(self.id);
coin::split(&mut coin, amount);
tx_context::sender(ctx);
vector::borrow(&vec, 0);
string::utf8(b"hello");
```

### Function Ordering

1. `public` functions first (for composability)
2. `public(package)` functions
3. Private functions
4. Test functions

### Parameter Ordering

1. `self` (if method)
2. Shared objects
3. Capabilities
4. Owned structs
5. Primitive values
6. `TxContext` (always last)

### Testing

```move
#[test]
fun deposit_increases_balance() {
    // Test implementation
}

#[test]
#[expected_failure(abort_code = EInsufficientBalance)]
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
- Use simplest solution possible
- Eliminate duplication ruthlessly

### Error Handling

```rust
// Production code: Use ? operator with context
use anyhow::{Context, Result};

fn process_event(event: &Event) -> Result<()> {
    let data = parse_event(event)
        .context("failed to parse event")?;

    db.insert(&data)
        .await
        .context("failed to insert into database")?;

    Ok(())
}

// Only use .expect() for true invariants
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

### Testing

```rust
// Place integration tests in tests/ directory
// crates/indexer/tests/handlers_tests.rs

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
# Fast type checking
cargo check

# Build with optimizations
cargo build --release

# Run tests
cargo test

# Lint (must pass with zero warnings)
cargo clippy -- -D warnings

# Format
cargo fmt

# Security audit
cargo audit
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

// Use branded types for domain IDs
type AccountId = string & { readonly __brand: 'AccountId' };
type PoolId = string & { readonly __brand: 'PoolId' };

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

**Repositories**: Data access only, no business logic
```typescript
export class AccountsRepository {
    constructor(private readonly db: Db) {}

    findByHashedSecret = async (hashedSecret: string): Promise<Account[]> => {
        return this.db
            .collection<AccountDocument>('accounts')
            .find({ hashed_secret: hashedSecret })
            .toArray();
    };
}
```

**Services**: Business logic, depends on repositories
```typescript
export class AccountsService {
    constructor(
        private readonly repository: AccountsRepository,
        private readonly suiService: SuiService,
    ) {}

    createAccount = async (params: CreateAccountParams): Promise<Account> => {
        // Business logic here
    };
}
```

**Handlers**: HTTP concerns only, uses services from context
```typescript
export const getAccounts = async (c: Context<AppBindings>): Promise<Response> => {
    const accountsService = c.get('accountsService');
    const validation = validateQuery(c, GetAccountsSchema);
    if (!validation.success) return validation.response;

    const data = await accountsService.findByHashedSecret(validation.data.hashed_secret);
    return c.json({ success: true, data });
};
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

### Async Patterns

```typescript
// Use async/await over raw Promises
const result = await service.process(data);

// Use Promise.all for concurrent independent operations
const [pools, accounts, commitments] = await Promise.all([
    poolsRepo.findAll(),
    accountsRepo.findByOwner(owner),
    commitmentsRepo.findByPool(poolId),
]);

// Handle fire-and-forget with .catch()
logger.flush().catch(console.error);
```

### Commands

```bash
# Development
bun run dev

# Type check (must pass before commit)
bun run typecheck

# Lint (must pass with zero warnings)
bun run lint

# Format
bun run format

# Run all quality checks
bun run typecheck && bun run lint && bun run format

# Tests
bun test

# Build for production
bun run build
```

---

## Quality Gates

### Pre-Commit Checklist

Before every commit, verify:

| Language | Command | Requirement |
|----------|---------|-------------|
| Rust | `cargo check` | Zero errors |
| Rust | `cargo clippy -- -D warnings` | Zero warnings |
| Rust | `cargo test` | All tests pass |
| Rust | `cargo fmt --check` | Formatted |
| TypeScript | `bun run typecheck` | Zero errors |
| TypeScript | `bun run lint` | Zero warnings |
| TypeScript | `bun run format --check` | Formatted |
| TypeScript | `bun test` | All tests pass |
| Move | `sui move build` | Zero errors |
| Move | `sui move test` | All tests pass |

### Implementation Verification

When implementing features or APIs:
1. Reference official documentation
2. Verify correctness against authoritative sources
3. Write tests that prove the implementation works
4. When fixing bugs, write failing tests first

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
│   ├── accounts.ts
│   ├── commitments.ts
│   ├── pools.ts
│   └── index.ts
├── services/               # Business logic layer
│   ├── accounts.ts
│   ├── health.ts
│   ├── merkle.ts
│   ├── sui.ts
│   └── index.ts
├── routes/v1/              # Route definitions
│   ├── accounts/
│   ├── commitments/
│   ├── merkle/
│   ├── pools/
│   └── transactions/
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
        └── src/
            └── lib.rs
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
│   ├── Move.toml
│   └── sources/
│       └── swap.move
└── test-coins/             # Test tokens
    ├── Move.toml
    └── sources/
        ├── sui.move
        └── usdc.move
```

---

## Anti-Patterns to Avoid

### Code Smells

| Don't | Do |
|-------|-----|
| One-time-use variables | Inline values |
| `Buffer.from(Buffer.from(...))` | Single conversion |
| `arr[arr.length - 1]` | `arr.at(-1)` |
| Sequential awaits for independent operations | `Promise.all()` |
| Dead code, unused exports | Delete completely |
| Backwards-compatibility hacks | Clean removal |
| Over-engineering, premature abstraction | Minimum viable solution |

### Architectural Violations

| Don't | Do |
|-------|-----|
| Business logic in handlers | Use services |
| Raw DB access in services | Use repositories |
| Global state/singletons | Dependency injection |
| Mixing refactoring with features | Separate commits |
| Comments explaining what code does | Self-documenting names |

---

## Security Considerations

### Never Commit

- Private keys, secrets, API keys
- `.env` files with real credentials
- Hardcoded addresses for production

### Input Validation

- Validate all external input at system boundaries
- Use Zod schemas for API requests
- Use Move `assert!` for on-chain validation
- Never trust client-provided data

### Cryptographic Operations

- Use established libraries (Arkworks for circuits)
- Never implement custom cryptography
- Validate proofs on-chain, never skip verification
