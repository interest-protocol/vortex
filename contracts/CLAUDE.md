# Move 2024 Semantic Rules for AI Coding Assistants

Rules enforced by the Move 2024 compiler. Violating these causes compilation errors.

---

## 1. Implicit Imports (DO NOT import these)

Move 2024 auto-imports these - adding explicit imports is redundant.

### Built-in Types

```move
bool, u8, u16, u32, u64, u128, u256, address, vector<T>
```

### Standard Library

| Full Path             | Use As   |
| --------------------- | -------- |
| `std::vector`         | `vector` |
| `std::option`         | `option` |
| `std::option::Option` | `Option` |

### Sui Framework

| Full Path                    | Use As       |
| ---------------------------- | ------------ |
| `sui::object`                | `object`     |
| `sui::transfer`              | `transfer`   |
| `sui::tx_context`            | `tx_context` |
| `sui::object::ID`            | `ID`         |
| `sui::object::UID`           | `UID`        |
| `sui::tx_context::TxContext` | `TxContext`  |

```move
module example::demo;

// WRONG: Redundant imports
use sui::object::UID;
use std::option::Option;

// CORRECT: Just use them directly
public struct MyObject has key {
    id: UID,
    data: Option<u64>,
}
```

---

## 2. Naming Rules (Compiler Enforced)

| Element               | Rule                    | Valid              | Invalid            |
| --------------------- | ----------------------- | ------------------ | ------------------ |
| Variables             | Start with `a-z` or `_` | `count`, `_unused` | `Count`, `123x`    |
| Functions             | Start with `a-z`        | `transfer`         | `_transfer`, `Transfer` |
| Structs/Enums         | Start with `A-Z`        | `Coin`, `NFT`      | `coin`, `_Coin`    |
| Constants             | Start with `A-Z`        | `MAX_SUPPLY`       | `max_supply`       |
| Type Parameters       | Start with `A-Z`        | `T`, `CoinType`    | `t`, `_T`          |
| Macro Parameters      | Start with `$`          | `$x`, `$value`     | `x`, `value`       |

---

## 3. Ability System

Every type has abilities that control usage. Missing abilities cause errors.

| Ability | What It Allows                              | Without It                          |
| ------- | ------------------------------------------- | ----------------------------------- |
| `copy`  | Value can be duplicated                     | Value moves on assignment           |
| `drop`  | Value can be implicitly discarded           | Must be explicitly consumed         |
| `store` | Can be field of `key` struct                | Cannot be stored in objects         |
| `key`   | Can be top-level Sui object                 | Cannot use `transfer::transfer`     |

### Built-in Abilities

| Type                              | Abilities              |
| --------------------------------- | ---------------------- |
| `u8`-`u256`, `bool`, `address`    | `copy`, `drop`, `store`|
| `&T`, `&mut T`                    | `copy`, `drop`         |
| `vector<T>`                       | Same as `T`            |

### Common Errors

```move
public struct NoDrop { value: u64 }  // No abilities declared

fun error_no_drop() {
    let x = NoDrop { value: 1 };
    // ERROR: "x is not used and cannot be dropped"
    // FIX: Either consume x or add `has drop` to struct
}

public struct NoCopy has drop { value: u64 }

fun error_no_copy() {
    let x = NoCopy { value: 1 };
    let y = x;  // x is MOVED
    let z = x;  // ERROR: "x was moved"
    // FIX: Add `has copy, drop` to struct
}
```

---

## 4. Reference Rules

### Borrowing Constraints

| Borrow Type | Allowed Simultaneously | Can Modify |
| ----------- | ---------------------- | ---------- |
| `&T`        | Multiple `&T`          | No         |
| `&mut T`    | Only one, no other borrows | Yes    |

### Compiler Errors

```move
// ERROR: "cannot store references in structs"
public struct Bad { r: &u64 }

// ERROR: "dangling reference" - returning ref to local
fun bad(): &u64 {
    let x = 5;
    &x  // x is dropped when function returns
}

// CORRECT: Return ref from parameter (outlives function)
fun good(x: &u64): &u64 { x }

// ERROR: "cannot borrow as mutable" - variable not declared mut
fun bad_mut() {
    let x = 5;
    let r = &mut x;  // ERROR
}

// CORRECT
fun good_mut() {
    let mut x = 5;
    let r = &mut x;  // OK
}
```

---

## 5. Mutability

Variables are immutable by default. Use `let mut` for mutability.

```move
let x = 5;
x = 6;  // ERROR: "cannot mutate immutable variable"

let mut y = 5;
y = 6;  // OK

// Mutable borrow requires mut variable
let a = 5;
let r = &mut a;  // ERROR

let mut b = 5;
let r = &mut b;  // OK
```

---

## 6. Ownership (Move vs Copy)

```move
public struct Copyable has copy, drop { v: u64 }
public struct MoveOnly has drop { v: u64 }  // No copy

fun ownership() {
    // With copy: value is duplicated
    let a = Copyable { v: 1 };
    let b = a;  // a is copied
    let c = a;  // a still valid

    // Without copy: value is moved
    let x = MoveOnly { v: 1 };
    let y = x;  // x is MOVED to y
    let z = x;  // ERROR: "x was moved"
}
```

---

## 7. Visibility

| Modifier          | Accessible From        |
| ----------------- | ---------------------- |
| `public`          | Any module, any package|
| `public(package)` | Same package only      |
| (none)            | Same module only       |

```move
module pkg::a;
public fun anyone() { }           // Callable from anywhere
public(package) fun pkg_only() { } // Only pkg::* modules
fun private() { }                  // Only this module
```

---

## 8. Type Restrictions

### No Recursive Types

```move
// ERROR: "recursive type"
public struct Node { next: Node }

// OK: Indirection via Option or vector
public struct Node { next: Option<Node> }
```

### Phantom Type Parameters

`phantom` types cannot appear in fields:

```move
public struct Marker<phantom T> {}  // OK
public struct Bad<phantom T> { value: T }  // ERROR: phantom T used in field
```

---

## 9. Constants

### Allowed Types Only

- Primitives: `u8`-`u256`, `bool`, `address`
- `vector<primitive>`
- Byte strings: `b"..."`

```move
const MAX: u64 = 100;                // OK
const BYTES: vector<u8> = b"hello";  // OK
const BAD: MyStruct = MyStruct {};   // ERROR: struct not allowed
```

### No Expressions

Constants cannot contain function calls, control flow, or references.

---

## 10. Pattern Matching

Patterns must be exhaustive:

```move
// ERROR: "non-exhaustive pattern"
match (opt) {
    Option::Some(x) => x,
    // Missing Option::None
}

// CORRECT
match (opt) {
    Option::Some(x) => x,
    Option::None => 0,
}
```

---

## 11. Error Quick Reference

| Error Message                 | Cause                        | Fix                                    |
| ----------------------------- | ---------------------------- | -------------------------------------- |
| "value without drop"          | No `drop`, value unused      | Consume value or add `has drop`        |
| "cannot copy value"           | No `copy`, used after move   | Add `has copy` or restructure          |
| "cannot borrow as mutable"    | Variable not `mut`           | Use `let mut`                          |
| "dangling reference"          | Returning ref to local       | Return owned or borrow from param      |
| "recursive type"              | Direct self-reference        | Use `Option<Self>` indirection         |
| "cannot access private"       | Calling non-public function  | Make it `public` or `public(package)`  |
| "phantom type used in field"  | `phantom T` in struct field  | Remove `phantom` or remove field       |

---

## 12. Sui Object Rules

### UID Must Be First Field

```move
// CORRECT
public struct MyObject has key {
    id: UID,  // MUST be first
    data: u64,
}

// ERROR: "first field must be UID"
public struct Bad has key { data: u64 }

// ERROR: "UID must be first field"
public struct AlsoBad has key { data: u64, id: UID }
```

### Enums Cannot Have `key`

```move
// ERROR: "enums cannot have key ability"
public enum Status has key { Active, Inactive }
```

### Fresh UID Required

```move
// CORRECT: UID from object::new()
let obj = MyObject { id: object::new(ctx), data: 0 };

// ERROR at runtime: Cannot reuse/pass UID
fun bad(uid: UID) {
    let obj = MyObject { id: uid, data: 0 };  // Will fail
}
```

---

## 13. `init` Function Rules

The `init` function is special - called once at publish time.

```move
module example::my_module;

public struct MY_MODULE has drop {}

// Valid signatures:
fun init(ctx: &mut TxContext) { }
fun init(otw: MY_MODULE, ctx: &mut TxContext) { }

// ERRORS:
public fun init(...) { }     // ERROR: must be private
entry fun init(...) { }      // ERROR: cannot be entry
fun init<T>(...) { }         // ERROR: no type parameters
fun init(...): u64 { }       // ERROR: must return ()
fun init(a: u64, b: u64, ctx: &mut TxContext) { }  // ERROR: max 2 params
```

**Rules:**
- Private only (no `public`, no `entry`)
- No type parameters
- Returns `()`
- Last param: `&TxContext` or `&mut TxContext`
- Max 2 params: optional OTW + TxContext

---

## 14. One-Time Witness (OTW)

OTW proves module publisher authority. Created by runtime, not constructable.

```move
module example::my_coin;

// OTW: UPPERCASE module name, only `drop`
public struct MY_COIN has drop {}

fun init(otw: MY_COIN, ctx: &mut TxContext) {
    // otw is created by runtime, cannot be constructed manually
    // Used to prove this code is running at publish time
}
```

**Requirements:**
- Name = UPPERCASE(module_name)
- Only `drop` ability (not `copy`, `store`, `key`)
- No type parameters
- No fields (or single `bool` field)

---

## 15. Public vs Entry Functions

| Modifier       | From PTB | From Move | Restrictions     |
| -------------- | -------- | --------- | ---------------- |
| `public`       | Yes      | Yes       | None             |
| `entry`        | Yes      | No        | Signature rules  |
| `public entry` | Yes      | Yes       | Signature rules  |

### Entry Signature Restrictions

**Valid parameters:**
- Primitives: `u8`-`u256`, `bool`, `address`
- `String`, `ascii::String`, `ID`
- `Option<primitive>`, `vector<T>`
- Objects (types with `key`)
- `Receiving<T>`

**Invalid parameters:**
- `&mut Clock` (use `&Clock`)
- `&mut Random` (use `&Random`)
- Structs without `key`

**Return:** Must have `drop` or be `()`

**Avoid `public entry`** - `public` already works from PTBs, `entry` just adds restrictions.

---

## 16. Transfer Rules

```move
// Private transfers - T must be defined in THIS module:
transfer::transfer(obj, recipient);
transfer::freeze_object(obj);
transfer::share_object(obj);

// Public transfers - T must have `store`:
transfer::public_transfer(obj, recipient);
transfer::public_freeze_object(obj);
transfer::public_share_object(obj);
```

**Rule:** Use private for module-defined types, public for types with `store`.

---

## 17. Event Rules

Events must use types from the current module:

```move
public struct MyEvent has copy, drop { value: u64 }

// CORRECT
event::emit(MyEvent { value: 42 });

// ERROR: "cannot emit external type"
event::emit(other_module::TheirEvent { });
```

---

## 18. Linter Warnings

These are warnings, not errors. Fix them for better code.

| Lint                  | Trigger                                  | Fix                                        |
| --------------------- | ---------------------------------------- | ------------------------------------------ |
| `share_owned`         | `share_object` on parameter/unpacked obj | Create fresh object, share in same function|
| `self_transfer`       | `transfer(obj, ctx.sender())`            | Return object instead                      |
| `custom_state_change` | Private transfer on type with `store`    | Use `public_transfer`                      |
| `coin_field`          | `Coin<T>` as struct field                | Use `Balance<T>` instead                   |
| `freeze_wrapped`      | Freezing struct with nested `key` fields | Don't - nested objects become inaccessible |
| `collection_equality` | `table == other_table`                   | Don't compare collections with `==`        |
| `public_random`       | Public function taking `Random`          | Make private or add access control         |
| `missing_key`         | Struct has `id: UID` but no `key`        | Add `has key`                              |
| `public_entry`        | `public entry fun`                       | Remove `entry` modifier                    |

Suppress with: `#[allow(lint(share_owned))]`

---

## 19. Storage Model

### Object States

| State         | Access              | Consensus | Mutable | Transition From      |
| ------------- | ------------------- | --------- | ------- | -------------------- |
| **Owned**     | Owner only          | No        | Yes     | Fresh                |
| **Shared**    | Anyone              | Yes       | Yes     | Fresh only           |
| **Immutable** | Anyone (read)       | No        | No      | Fresh only           |
| **Wrapped**   | Via parent          | Via parent| Via parent | Fresh             |

### Creating and Transferring

```move
// Owned: transfer to address
let obj = MyObject { id: object::new(ctx), value: 0 };
transfer::transfer(obj, ctx.sender());

// Shared: anyone can access (requires consensus)
let obj = MyObject { id: object::new(ctx), value: 0 };
transfer::share_object(obj);

// Immutable: frozen forever
let obj = MyObject { id: object::new(ctx), value: 0 };
transfer::freeze_object(obj);

// Wrapped: store as field (no longer independently accessible)
public struct Wrapper has key {
    id: UID,
    inner: MyObject,  // wrapped
}
```

### Critical Rules

1. **Share only fresh objects** - sharing from parameter fails at runtime
   ```move
   // WRONG - will abort
   public fun bad(obj: MyObject) {
       transfer::share_object(obj);
   }

   // CORRECT
   public fun good(ctx: &mut TxContext) {
       let obj = MyObject { id: object::new(ctx), value: 0 };
       transfer::share_object(obj);
   }
   ```

2. **State transitions are irreversible**
   - Shared → Cannot transfer or freeze
   - Immutable → Cannot modify or transfer

3. **Wrapped objects lose independent identity**
   - UID exists but object not addressable
   - Access only through parent
   - Can unwrap by extracting and transferring

---

## 20. Quick Reference: Valid Patterns

```move
module example::valid;

// Object with abilities
public struct MyObject has key, store {
    id: UID,      // First field must be UID for key
    value: u64,
}

// Data struct (no UID needed)
public struct Data has copy, drop, store {
    amount: u64,
}

// OTW for init
public struct VALID has drop {}

// Event
public struct ValueChanged has copy, drop {
    old_value: u64,
    new_value: u64,
}

// Private init
fun init(_otw: VALID, ctx: &mut TxContext) {
    let obj = MyObject { id: object::new(ctx), value: 0 };
    transfer::share_object(obj);
}

// Public function (preferred over entry)
public fun update(self: &mut MyObject, new_value: u64) {
    event::emit(ValueChanged {
        old_value: self.value,
        new_value
    });
    self.value = new_value;
}

// Generic with ability constraint
public fun transfer_any<T: key + store>(obj: T, recipient: address) {
    transfer::public_transfer(obj, recipient);
}
```
