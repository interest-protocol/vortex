# Move 2024 Semantic Rules

A guide to the semantic restrictions enforced by the Move 2024 compiler.

---

## 1. Implicit Imports and Aliases

Move 2024 automatically imports common modules and types.

### Built-in Types (always available)

```move
bool, u8, u16, u32, u64, u128, u256, address, vector<T>
```

### Standard Library (auto-imported)

| Module/Type           | Available As |
| --------------------- | ------------ |
| `std::vector`         | `vector`     |
| `std::option`         | `option`     |
| `std::option::Option` | `Option`     |

### Sui Framework (auto-imported)

| Module/Type                  | Available As |
| ---------------------------- | ------------ |
| `sui::object`                | `object`     |
| `sui::transfer`              | `transfer`   |
| `sui::tx_context`            | `tx_context` |
| `sui::object::ID`            | `ID`         |
| `sui::object::UID`           | `UID`        |
| `sui::tx_context::TxContext` | `TxContext`  |

### Module-level Aliases

- `Self` - alias for the current module
- All defined structs, enums, functions, and constants by their name

```move
module example::demo;

// No imports needed - these are implicit in Sui mode:
public struct MyObject has key {
    id: UID,           // UID is implicit
    data: Option<u64>, // Option is implicit
}

public fun create(ctx: &mut TxContext): MyObject {
    MyObject {
        id: object::new(ctx),    // object module is implicit
        data: option::none(),    // option module is implicit
    }
}
```

---

## 2. Naming Rules

| Element               | Rule                             | Example                         |
| --------------------- | -------------------------------- | ------------------------------- |
| Variables             | Start with `a-z` or `_`          | `let count = 0;`                |
| Functions             | Start with `a-z`, no leading `_` | `public fun transfer()`         |
| Structs/Enums         | Start with `A-Z`                 | `public struct Coin { }`        |
| Constants             | Start with `A-Z`                 | `const MAX_SUPPLY: u64 = 1000;` |
| Type Parameters       | Start with `A-Z`                 | `public struct Box<T> { }`      |
| Macro Parameters      | Must start with `$`              | `macro fun m($x: u64)`          |
| Macro Type Parameters | Must start with `$`              | `macro fun m<$T>(x: $T)`        |

---

## 3. Ability System

| Ability | Meaning                                        |
| ------- | ---------------------------------------------- |
| `copy`  | Value can be copied                            |
| `drop`  | Value can be implicitly discarded              |
| `store` | Value can be used as a field of a `key` struct |
| `key`   | Value can be a top-level storage object (Sui)  |

**Built-in type abilities:**

- Primitives (`u8`-`u256`, `bool`, `address`): `copy`, `drop`, `store`
- References (`&T`, `&mut T`): `copy`, `drop` only
- `vector<T>`: inherits from `T`

**Rules:**

- Types without `drop` MUST be explicitly consumed
- Types without `copy` are moved on use (single ownership)

```move
public struct NoDrop { value: u64 }  // No abilities

fun bad() {
    let x = NoDrop { value: 1 };
    // ERROR: x is not used and cannot be dropped
}

fun good() {
    let x = NoDrop { value: 1 };
    consume(x);  // x is explicitly consumed
}
```

---

## 4. Reference and Borrow Rules

**Immutable borrows (`&T`):**

- Multiple immutable borrows allowed simultaneously
- Cannot modify through immutable reference

**Mutable borrows (`&mut T`):**

- Only ONE mutable borrow at a time
- No other borrows can coexist
- Variable must be declared `mut`

**Critical restrictions:**

- Cannot return references to local variables (dangling)
- Struct fields CANNOT contain references
- References have `copy` and `drop` but NOT `store`

```move
// WRONG: Cannot store references in structs
public struct Bad { r: &u64 }

// WRONG: Dangling reference
fun bad(): &u64 {
    let x = 5;
    &x  // ERROR: x is dropped, reference dangles
}

// CORRECT: Borrow from parameter
fun good(x: &u64): &u64 { x }
```

---

## 5. Mutability Rules

Variables are immutable by default:

```move
let x = 5;
x = 6;  // ERROR: cannot mutate immutable variable

let mut y = 5;
y = 6;  // OK
```

To take a mutable borrow, the variable must be `mut`:

```move
let x = 5;
let r = &mut x;  // ERROR

let mut y = 5;
let r = &mut y;  // OK
```

---

## 6. Move vs Copy Semantics

```move
public struct Coin has copy, drop { value: u64 }
public struct NFT has drop { id: u64 }  // No copy

fun example() {
    let coin = Coin { value: 100 };
    let c2 = coin;      // Copied (has copy)
    let c3 = coin;      // Still valid

    let nft = NFT { id: 1 };
    let n2 = nft;       // Moved (no copy)
    let n3 = nft;       // ERROR: nft was moved
}
```

---

## 7. Visibility Rules

| Visibility        | Accessible From   |
| ----------------- | ----------------- |
| `public`          | Anywhere          |
| `public(package)` | Same package only |
| (none)            | Same module only  |

---

## 8. Type Restrictions

**Recursive types are forbidden:**

```move
// ERROR: Recursive type
public struct Node { next: Node }

// OK: Use Option for indirection
public struct Node { next: Option<Node> }
```

**Phantom type parameters:**

```move
public struct Marker<phantom T> {}  // OK: T not used in fields
public struct Bad<phantom T> { value: T }  // ERROR: phantom T in field
```

---

## 9. Constant Restrictions

Constants can only have these types:

- Primitives: `u8`, `u16`, `u32`, `u64`, `u128`, `u256`, `bool`, `address`
- `vector` of primitives
- Byte strings

```move
const MAX: u64 = 100;               // OK
const BYTES: vector<u8> = b"hello"; // OK
const BAD: Coin = Coin { };         // ERROR: struct not allowed
```

Constant expressions cannot contain function calls, control flow, references, or non-constant values.

---

## 10. Pattern Matching Rules

Patterns must be exhaustive:

```move
// ERROR: Non-exhaustive
match (opt) {
    Option::Some(x) => x,
    // Missing None case
}

// CORRECT
match (opt) {
    Option::Some(x) => x,
    Option::None => 0,
}
```

---

## 11. Common Errors and Fixes

| Error                | Cause                           | Fix                                         |
| -------------------- | ------------------------------- | ------------------------------------------- |
| "value without drop" | Type lacks `drop`, not consumed | Explicitly use or destroy the value         |
| "cannot copy"        | Type lacks `copy`               | Use `move` or add `copy` ability            |
| "invalid borrow"     | Borrowing moved value           | Borrow before move, or copy first           |
| "cannot mutate"      | Variable not `mut`              | Add `mut` to declaration                    |
| "dangling reference" | Returning local ref             | Return owned value or borrow param          |
| "recursive type"     | Self-referential struct         | Use `Option` or `vector` indirection        |
| "visibility"         | Calling private function        | Make function `public` or `public(package)` |

---

## 12. Sui-Specific Rules

### Object Rules

**Objects must have `id: UID` as first field:**

```move
// CORRECT
public struct MyObject has key {
    id: UID,
    data: u64,
}

// ERROR: Missing UID or wrong position
public struct Bad has key { data: u64 }
public struct AlsoBad has key { data: u64, id: UID }
```

**Enums cannot have `key` ability.**

**Fresh UID required for object creation:**

```move
// CORRECT
let obj = MyObject { id: object::new(ctx), data: 0 };

// ERROR: Reusing UID from elsewhere
let obj = MyObject { id: some_other_uid, data: 0 };
```

### `init` Function Rules

```move
module example::my_module;

public struct MY_MODULE has drop {}

// CORRECT init signatures:
fun init(ctx: &mut TxContext) { }
fun init(otw: MY_MODULE, ctx: &mut TxContext) { }
```

**Rules:**

- Must be private (no visibility modifier)
- Cannot be `entry`
- No type parameters allowed
- Must return `()`
- Last parameter must be `&TxContext` or `&mut TxContext`
- Maximum 2 parameters (OTW + TxContext)
- Cannot be called directly (only at publish time)

### One-Time Witness (OTW)

```move
module example::my_coin;

public struct MY_COIN has drop {}

fun init(otw: MY_COIN, ctx: &mut TxContext) {
    // otw can only be received here, never constructed
}
```

**OTW Requirements:**

- Name must be uppercase version of module name
- Only `drop` ability (no `copy`, `store`, `key`)
- No type parameters
- No fields, or single `bool` field
- Cannot be manually constructed

### Public vs Entry Functions

| Modifier       | Callable From            | Restrictions       |
| -------------- | ------------------------ | ------------------ |
| `public fun`   | PTBs + other modules     | None               |
| `entry fun`    | PTBs only                | Signature rules    |
| `public entry` | PTBs + other modules     | Signature rules    |

**`entry` signature restrictions:**

- Valid params: primitives, strings, `ID`, `Option<primitive>`, `vector<T>`, objects (`key` types), `Receiving<T>`
- Invalid params: `&mut Clock`, `&mut Random`, non-object structs
- Return type must have `drop` (or be `()`)

**Avoid `public entry`** - adds restrictions without benefit.

### Transfer Rules

```move
// These require T defined in SAME module:
transfer::transfer(obj, recipient);
transfer::freeze_object(obj);
transfer::share_object(obj);

// For types with `store`, use public versions:
transfer::public_transfer(obj, recipient);
transfer::public_freeze_object(obj);
transfer::public_share_object(obj);
```

### Event Rules

Events must use types defined in the current module:

```move
// CORRECT
event::emit(MyEvent { value: 42 });

// ERROR: Cannot emit external types
event::emit(other_module::TheirEvent { });
```

### Sui Linter Warnings

| Filter                | Issue                                    | Fix                                     |
| --------------------- | ---------------------------------------- | --------------------------------------- |
| `share_owned`         | Sharing object from parameter            | Create fresh and share in same function |
| `self_transfer`       | Transferring to `tx_context::sender()`   | Return object for composability         |
| `custom_state_change` | Custom transfer on types with `store`    | Use `public_transfer`                   |
| `coin_field`          | `Coin` in struct field                   | Use `Balance` instead                   |
| `freeze_wrapped`      | Freezing struct with nested `key` types  | Nested objects become inaccessible      |
| `collection_equality` | Comparing `Table`, `Bag` with `==`       | Structural equality not checked         |
| `public_random`       | Public function taking `Random`          | Make private or add access control      |
| `missing_key`         | Struct has `id: UID` but no `key`        | Add `key` ability                       |
| `public_entry`        | `public entry fun`                       | Remove `entry` or make non-public       |

---

## 13. Sui Storage Model

### Object Ownership States

| State         | Owner          | Consensus | Mutable    | Use Case                   |
| ------------- | -------------- | --------- | ---------- | -------------------------- |
| **Owned**     | Single address | No        | Yes        | User assets, personal data |
| **Shared**    | None (global)  | Yes       | Yes        | AMM pools, registries      |
| **Immutable** | None (frozen)  | No        | No         | Config, constants          |
| **Wrapped**   | Parent object  | Via parent| Via parent | Composition, bundling      |

### Owned Objects

```move
let obj = MyObject { id: object::new(ctx), value: 0 };
transfer::transfer(obj, ctx.sender());
```

- Fast path execution (no consensus)
- Can be transferred, shared, or frozen

### Shared Objects

```move
// CORRECT: Create and share in same function
let obj = MyObject { id: object::new(ctx), value: 0 };
transfer::share_object(obj);

// WRONG: Sharing object from parameter
public fun bad(obj: MyObject) {
    transfer::share_object(obj);  // Lint: share_owned
}
```

- Requires consensus ordering (slower)
- Cannot be transferred once shared
- **Must share freshly created objects**

### Immutable Objects

```move
transfer::freeze_object(obj);
```

- Read-only forever
- No consensus needed (fast reads)

### Wrapped Objects

```move
public struct Wrapper has key {
    id: UID,
    inner: InnerObject,  // wrapped - not in global storage
}
```

- Not directly accessible
- Can be unwrapped by extracting and transferring

### State Transitions

- Fresh → Shared: `share_object()` (irreversible)
- Fresh → Immutable: `freeze_object()` (irreversible)
- Fresh → Wrapped: Store as field
- Wrapped → Owned: Extract and `transfer()`
- Shared/Immutable: Cannot change state
