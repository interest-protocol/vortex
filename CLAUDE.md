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
- Use meaningful test names describing behavior
- Implement unit tests, integration tests where appropriate
- All tests must pass before commits
- When fixing bugs, write failing tests first

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
