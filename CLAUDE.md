# Instructions for Claude

## Rust Code Guidelines

### Code Style
- Do not add comments to the code
- Use Rust best practices and production-ready code
- Eliminate duplication ruthlessly
- Express intent through naming and structure
- Keep methods small with single responsibility
- Minimize state and side effects
- Use simplest solutions possible

### Quality Checks
- Double check that things build and there are no type errors
- Always verify builds with `cargo check` or `cargo build` before completing tasks
- Run `cargo clippy` for lint analysis
- Run `cargo test` to ensure all tests pass
- All compiler/linter warnings must be resolved before completion

### Implementation Verification
- Try to find official documentation and implementation to make sure ours is correct
- When implementing features or APIs, reference authoritative sources to ensure accuracy

### Cargo Commands
- Batch cargo operations when possible for efficiency
- `cargo check` - Fast type checking
- `cargo build --release` - Optimized compilation
- `cargo test` - Execute test suites
- `cargo clippy` - Lint analysis
- `cargo fmt` - Code formatting
- `cargo audit` - Security vulnerability scanning

### Testing Standards
- Write tests that define small functionality increments
- Use meaningful test names describing behavior (without `test_` prefix for clarity)
- Place integration tests in `tests/` directory at crate root (e.g., `crates/indexer/tests/`)
- Keep unit tests minimal; prefer integration tests in separate files
- Organize test files by module: `tests/handlers_tests.rs`, `tests/lib_tests.rs`
- All tests must pass before commits
- When fixing bugs, write failing tests first
- Use `assert_eq!` for equality checks when possible

### Development Workflow
- Make one logical change at a time
- Ensure tests pass after each change
- Prefer small, frequent commits over large ones
- Never mix structural changes (renaming, refactoring) with behavioral changes (new features, bug fixes) in the same commit

## Sui Move Code Guidelines

### Tool Calling
- `sui move build` compiles packages (run in Move.toml directory)
- `sui move test` runs tests
- Use `--skip-fetch-latest-git-deps` flag after initial successful builds
- Format modified files with `bunx prettier-move -c *.move --write`

### Package Manifest
- Requires Move 2024 Edition (`2024.beta` or `2024`)
- Sui, Bridge, MoveStdlib, and SuiSystem are implicitly imported (Sui 1.45+)
- Prefix named addresses with project names to avoid conflicts

### Module & Imports
- Use module label syntax: `module my_package::my_module;` (not block syntax)
- Avoid standalone `{Self}` in use statements
- Group imports with Self: `use path::{Self, Member};`
- Error constants use `EPascalCase`; regular constants use `ALL_CAPS`

### Structs
- Capability structs end with `Cap` suffix
- Avoid "Potato" in type names (implicit for zero-ability structs)
- Events use past tense naming
- Dynamic field keys are positional with `Key` suffix

### Functions
- Use `public` over `public entry` for composability
- Objects appear first in parameters (except Clock)
- Capabilities come second
- Getter methods match field names; mutable versions add `_mut`
- Prefer struct methods over module functions

### Common Patterns
- Use `coin.split()` method instead of `coin::split()` function
- Access vectors with index syntax: `vec[0]` not `vector::borrow()`
- Use `ctx.sender()` not `tx_context::sender(ctx)`
- Avoid importing `std::string::utf8`; use `b"".to_string()` instead
- Call `id.delete()` as method, not `object::delete()`

### Testing Standards
- Combine `#[test]` and `#[expected_failure(...)]` attributes
- Don't clean up expected_failure tests
- Omit `test_` prefix in testing modules
- Use `assert_eq!` when possible
- Implement "black hole" destroy functions for cleanup

### Code Style
- Comment only functions, struct fields, and complex logic
- Public functions first, then `public(package)`, then private
- Prefer macros over constants
- Only import necessary items

## TypeScript/JavaScript Guidelines (API)

### Tech Stack
- Runtime: Bun
- Framework: Hono
- Database: MongoDB (native driver)
- Cache: Redis (ioredis)
- Validation: Zod
- Logging: Pino

### Code Style
- Do not add comments unless logic is complex
- Use strict TypeScript (`strict: true` in tsconfig)
- Prefer `type` over `interface` unless extending
- Use `const` by default; `let` only when reassignment needed
- Always use arrow functions (not `function` declarations)
- Functions with more than 2 parameters must use an object parameter with named properties
- Use template literals over string concatenation
- Destructure objects and arrays when accessing multiple properties
- Eliminate duplicate code ruthlessly - extract to shared utils (e.g., `src/utils/schemas.ts`)
- Use kebab-case for file names (e.g., `rate-limit.ts`, not `rateLimit.ts`)

### Code Readability
- Code is read by humans - make it readable, easy to understand and reason with
- Add blank lines between all top-level declarations (constants, types, functions, exports)
- Separate logical groups of code with blank lines for visual clarity

### Type Safety
- Never use `any`; use `unknown` and narrow with type guards
- Always annotate function return types explicitly
- Use branded types for domain IDs (e.g., `type UserId = string & { readonly brand: unique symbol }`)
- Prefer `satisfies` operator over type assertions
- Use discriminated unions for state machines and variants
- Enable `noUncheckedIndexedAccess` for safer array/object access

### Imports & Exports
- Use `@/` path alias for internal imports: `import { env } from '@/config/env.js'`
- Use `type` imports for type-only imports: `import type { Foo } from '@/types/index.js'`
- Group imports: built-ins → external packages → internal modules (`@/`)
- Use named exports; avoid default exports
- Use `.js` extension in imports (required for ESM)

### Error Handling
- Use `invariant` from `tiny-invariant` for assertions instead of `if (!x) throw new Error()`
- Use custom error classes extending `Error` for domain errors
- Prefer early returns over nested conditionals
- Always handle Promise rejections
- Use Result pattern for expected failures: `type Result<T, E> = { ok: true; value: T } | { ok: false; error: E }`

### Async Patterns
- Always use `async/await` over raw Promises
- Use `Promise.all()` for concurrent independent operations
- Never use `void` operator; use `.catch()` for fire-and-forget promises
- Use `AbortController` for cancellable operations

### API Design (Hono)
- Group routes by domain in separate files under `src/routes/`
- Use Zod schemas for request validation
- Return consistent response shapes: `{ success: boolean; data?: T; error?: string }`
- Use proper HTTP status codes (200, 201, 400, 401, 404, 500)
- Inject dependencies via Hono context, not globals

### Database (MongoDB)
- Use typed collections: `db.collection<User>('users')`
- Create indexes for frequently queried fields
- Use projection to limit returned fields
- Prefer bulk operations for batch writes
- Always handle connection errors gracefully

### Testing Standards
- Place tests in `tests/` directory
- Use descriptive test names: `'returns 404 when user not found'`
- Test behavior, not implementation
- Mock external services (DB, Redis, APIs)
- Run `bun test` before commits

### Quality Checks
- After EVERY code change, run: `bun run typecheck && bun run lint && bun run format`
- All type errors must be resolved before proceeding
- All lint warnings must be resolved before completion
- Verify server starts without errors
- Never commit code that fails typecheck or lint

### Bun Commands
- `bun run dev` - Start dev server with hot reload
- `bun run build` - Build for production
- `bun run start` - Run production build
- `bun run typecheck` - Type check without emit
- `bun run lint` - Run ESLint
- `bun run format` - Format code with Prettier
- `bun test` - Run tests

### Project Structure
```
api/
├── src/
│   ├── index.ts          # Entry point
│   ├── config/           # Environment and app config
│   ├── constants/        # Shared constants (pagination, etc.)
│   ├── db/               # Database connections
│   │   └── collections/  # Collection types and constants
│   ├── middleware/       # Hono middleware (DI injection)
│   ├── repositories/     # Data access layer (MongoDB queries)
│   │   ├── accounts.ts   # AccountsRepository
│   │   ├── commitments.ts # CommitmentsRepository
│   │   ├── pools.ts      # PoolsRepository
│   │   └── index.ts      # Re-exports all repositories
│   ├── routes/           # Route definitions
│   │   ├── v1/           # API version 1
│   │   │   └── {domain}/ # Domain route folder (pools, accounts, etc.)
│   │   │       ├── index.ts     # Route definitions only
│   │   │       ├── handlers.ts  # Handler functions
│   │   │       ├── schema.ts    # Zod validation schemas
│   │   │       ├── types.ts     # API response types only
│   │   │       └── mappers.ts   # DB → API transformations
│   │   └── health.ts     # Health check route
│   ├── services/         # Business logic layer
│   │   ├── accounts.ts   # AccountsService (Sui transactions)
│   │   ├── health.ts     # HealthService (connectivity checks)
│   │   ├── merkle.ts     # MerkleService (tree operations)
│   │   ├── sui.ts        # Low-level Sui client
│   │   └── index.ts      # Re-exports all services
│   ├── types/            # Shared type definitions (AppBindings, etc.)
│   └── utils/            # Helper functions (validation.ts, logger.ts)
├── tests/                # Test files
└── package.json
```

### Architecture Pattern (Dependency Injection)
- **Repositories**: Handle data access (MongoDB queries). No business logic.
- **Services**: Handle business logic. Depend on repositories, not raw db/redis.
- **Handlers**: Handle HTTP. Use services via `c.get('serviceName')`.
- **Middleware**: Creates and injects all dependencies into Hono context.

Example handler pattern:
```typescript
export const getAccounts = async (c: Context<AppBindings>) => {
    const accountsService = c.get('accountsService');
    const validation = validateQuery(c, schema);
    if (!validation.success) return validation.response;
    const data = await accountsService.findByHashedSecret(validation.data.hashed_secret);
    return c.json({ success: true, data });
};
```

### Code Simplicity Rules
- No unnecessary one-time-use variables - inline values when used only once
- Avoid redundant operations (e.g., `Buffer.from(Buffer.from(...))`)
- Use `.at(-1)` instead of `arr[arr.length - 1]` for safe last element access
- Use nullish coalescing (`??`) and optional chaining (`?.`) for cleaner null handling
- Prefer method chaining over multiple assignments
- Use `Promise.all()` for parallel operations instead of sequential awaits
- Write succinct, readable code - fewer lines when clarity is maintained
- Never leave dead code in the codebase - remove unused exports, functions, and variables

### Route Structure Pattern
Each route domain folder follows this structure:
- `index.ts` - Route definitions only (keep minimal, just wire handlers to routes)
- `handlers.ts` - Handler functions containing the business logic
- `schema.ts` - Zod validation schemas for request body/query
- `types.ts` - Domain-specific types and filter types
- `mappers.ts` - Transform DB documents to API response shapes

### Development Workflow
- Make one logical change at a time
- Run typecheck and lint after each change
- Prefer small, focused commits
- Never mix refactoring with feature changes

### Git Commits
- Use conventional commits format: `emoji type(scope): subject`
- ALWAYS use emoji at the start of commit messages
- Do NOT add "Generated with Claude" or "Co-Authored-By: Claude" to commits
- Keep commit messages concise and descriptive

Commit types with required emojis:
- `✨ feat` - New feature
- `🐛 fix` - Bug fix
- `📝 docs` - Documentation only
- `🎨 style` - Code style (formatting, semicolons, etc.)
- `♻️ refactor` - Code change that neither fixes a bug nor adds a feature
- `⚡ perf` - Performance improvement
- `✅ test` - Adding or updating tests
- `📦 build` - Build system or external dependencies
- `👷 ci` - CI configuration
- `🔧 chore` - Other changes (updating dependencies, etc.)
- `⏪ revert` - Revert a previous commit

Examples:
- `✨ feat(api): add user authentication`
- `🐛 fix(indexer): handle null checkpoint`
- `♻️ refactor(api): implement dependency injection`

Rules:
- Subject must be lowercase
- Subject cannot be empty
- Type cannot be empty
