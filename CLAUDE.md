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
- Prefer arrow functions for callbacks and handlers
- Use template literals over string concatenation
- Destructure objects and arrays when accessing multiple properties

### Type Safety
- Never use `any`; use `unknown` and narrow with type guards
- Always annotate function return types explicitly
- Use branded types for domain IDs (e.g., `type UserId = string & { readonly brand: unique symbol }`)
- Prefer `satisfies` operator over type assertions
- Use discriminated unions for state machines and variants
- Enable `noUncheckedIndexedAccess` for safer array/object access

### Imports & Exports
- Use `type` imports for type-only imports: `import type { Foo } from './foo'`
- Group imports: built-ins → external packages → internal modules
- Use named exports; avoid default exports
- Use `.js` extension in imports (required for ESM)

### Error Handling
- Use custom error classes extending `Error`
- Prefer early returns over nested conditionals
- Always handle Promise rejections
- Use Result pattern for expected failures: `type Result<T, E> = { ok: true; value: T } | { ok: false; error: E }`

### Async Patterns
- Always use `async/await` over raw Promises
- Use `Promise.all()` for concurrent independent operations
- Avoid floating promises; always await or void them
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
- Run `bun run typecheck` to verify types
- Run `bun run lint` for ESLint analysis
- All warnings must be resolved before completion
- Verify server starts without errors

### Bun Commands
- `bun run dev` - Start dev server with hot reload
- `bun run build` - Build for production
- `bun run start` - Run production build
- `bun run lint` - Run ESLint
- `bun run typecheck` - Type check without emit
- `bun test` - Run tests

### Project Structure
```
api/
├── src/
│   ├── index.ts          # Entry point
│   ├── config/           # Environment and app config
│   ├── db/               # Database connections
│   ├── middleware/       # Hono middleware
│   ├── routes/           # Route handlers by domain
│   ├── services/         # Business logic
│   ├── types/            # Shared type definitions
│   └── utils/            # Helper functions
├── tests/                # Test files
└── package.json
```

### Development Workflow
- Make one logical change at a time
- Run typecheck and lint after each change
- Prefer small, focused commits
- Never mix refactoring with feature changes
