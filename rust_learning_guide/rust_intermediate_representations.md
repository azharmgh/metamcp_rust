# Rust Intermediate Representations: HIR, MIR, and LLVM IR

A comprehensive guide to understanding the compilation pipeline in Rust.

## Overview

Rust's compilation pipeline transforms your source code through several intermediate representations before producing the final machine code:

```
Rust Source → AST → HIR → MIR → LLVM IR → Machine Code
```

Each stage serves a specific purpose in ensuring Rust's safety guarantees and performance optimizations.

---

## LLVM IR (Low-Level Virtual Machine Intermediate Representation)

### What is LLVM IR?

**LLVM IR** is a low-level, typed, assembly-like language that serves as the middle layer in LLVM-based compilers. Rust uses LLVM as its backend, making LLVM IR the bridge between Rust's internal representations and the final machine code.

LLVM IR is platform-independent and allows LLVM to perform powerful optimizations before generating the final machine code for your target architecture.

### Example: Rust Code → LLVM IR Mapping

#### Rust Source Code

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn square(x: i32) -> i32 {
    x * x
}
```

#### Corresponding LLVM IR (simplified, with optimizations)

```llvm
; Function: add
define i32 @add(i32 %a, i32 %b) {
entry:
  %result = add i32 %a, %b
  ret i32 %result
}

; Function: square
define i32 @square(i32 %x) {
entry:
  %result = mul i32 %x, %x
  ret i32 %result
}
```

### LLVM IR Syntax Reference

| LLVM IR Element | Meaning |
|-----------------|---------|
| `define` | Declares a function definition |
| `i32` | 32-bit integer type (matches Rust's `i32`) |
| `@add` | Global function name (prefixed with `@`) |
| `%a`, `%b` | Local variables/registers (prefixed with `%`) |
| `entry:` | Basic block label (entry point of function) |
| `add i32 %a, %b` | Integer addition instruction |
| `ret i32 %result` | Return instruction |

### Control Flow Example

#### Rust Code with Control Flow

```rust
fn max(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}
```

#### LLVM IR Output

```llvm
define i32 @max(i32 %a, i32 %b) {
entry:
  %cmp = icmp sgt i32 %a, %b      ; signed greater-than comparison
  br i1 %cmp, label %then, label %else

then:
  br label %merge

else:
  br label %merge

merge:
  %result = phi i32 [ %a, %then ], [ %b, %else ]
  ret i32 %result
}
```

#### Key Control Flow Instructions

| Instruction | Purpose |
|-------------|---------|
| `icmp sgt` | Integer compare, signed greater-than |
| `br i1 %cmp, ...` | Conditional branch based on boolean |
| `phi` | SSA (Static Single Assignment) merge node — selects value based on which block we came from |

---

## HIR (High-level Intermediate Representation)

### What is HIR?

**HIR** is a desugared, simplified version of the AST (Abstract Syntax Tree). It's still close to the original Rust source but with syntactic sugar removed.

### What HIR Does

| Task | Example |
|------|---------|
| **Desugars** syntax | `for` loops → `loop` + `match` on iterators |
| **Resolves** names | Figures out what each identifier refers to |
| **Expands** macros | `println!` → actual function calls |
| **Type checking** | Happens on HIR |

### Example: Desugaring a `for` Loop

**Rust Source:**

```rust
for x in 0..5 {
    println!("{}", x);
}
```

**HIR (conceptually desugared to):**

```rust
{
    let mut iter = IntoIterator::into_iter(0..5);
    loop {
        match Iterator::next(&mut iter) {
            Some(x) => { println!("{}", x); }
            None => break,
        }
    }
}
```

### Common Desugaring Examples

| Rust Syntax | HIR Desugars To |
|-------------|-----------------|
| `x?` | `match x { Ok(v) => v, Err(e) => return Err(e.into()) }` |
| `async fn` | Function returning `impl Future` |
| `if let Some(x) = y` | `match y { Some(x) => ..., _ => () }` |
| `a..b` | `Range { start: a, end: b }` |

---

## MIR (Mid-level Intermediate Representation)

### What is MIR?

**MIR** is a much lower-level, control-flow-graph-based representation. This is where Rust does its most important work: **borrow checking**.

### What MIR Does

| Task | Purpose |
|------|---------|
| **Borrow checking** | Enforces ownership and lifetimes |
| **Optimizations** | Rust-specific opts before LLVM |
| **Monomorphization** | Generates concrete versions of generic functions |
| **Drop elaboration** | Inserts destructor calls |

### MIR Characteristics

- Uses **basic blocks** (like LLVM IR)
- **SSA-like** (variables assigned once per location)
- **Explicit control flow** (no implicit returns)
- **Explicit drops** (destructor calls visible)

### Example: Rust to MIR

**Rust Source:**

```rust
fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}
```

**MIR Output (simplified):**

```
fn max(_1: i32, _2: i32) -> i32 {
    let mut _0: i32;              // return place
    let mut _3: bool;

    bb0: {
        _3 = Gt(_1, _2);          // _3 = a > b
        switchInt(_3) -> [0: bb2, otherwise: bb1];
    }

    bb1: {
        _0 = _1;                  // return = a
        goto -> bb3;
    }

    bb2: {
        _0 = _2;                  // return = b
        goto -> bb3;
    }

    bb3: {
        return;
    }
}
```

### Key MIR Elements

| Element | Meaning |
|---------|---------|
| `_0` | Return place (where result goes) |
| `_1`, `_2` | Function arguments |
| `bb0`, `bb1`... | Basic blocks |
| `switchInt` | Conditional branch |
| `goto` | Unconditional jump |

---

## Comparison: HIR vs MIR vs LLVM IR

| Aspect | HIR | MIR | LLVM IR |
|--------|-----|-----|---------|
| **Level** | High (near source) | Mid (control flow) | Low (near assembly) |
| **Purpose** | Type checking, name resolution | Borrow checking, Rust opts | Platform opts, codegen |
| **Structure** | Tree-like (expressions) | Control flow graph | Control flow graph |
| **Rust-specific** | Yes | Yes | No (generic) |
| **Generics** | Still generic | Monomorphized | Monomorphized |
| **Drops** | Implicit | Explicit | Explicit |

---

## How to Generate These Representations

### Generate LLVM IR

```bash
# Unoptimized (verbose, closer to source)
rustc --emit=llvm-ir example.rs

# Optimized (cleaner, what actually runs)
rustc --emit=llvm-ir -O example.rs
```

### Generate MIR

```bash
# Using rustc directly
rustc --emit=mir example.rs

# Using cargo
RUSTFLAGS="--emit=mir" cargo build
```

### Generate HIR

```bash
# Requires nightly Rust
rustc +nightly -Z unpretty=hir example.rs
```

---

## Visual Summary

```
┌─────────────────────────────────────────────────────────────┐
│  Rust Source                                                │
│  fn add(a: i32, b: i32) -> i32 { a + b }                   │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  HIR  - Desugared, macros expanded, type checked           │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  MIR  - Control flow graph, borrow checked, drops inserted │
│  bb0: { _0 = Add(_1, _2); return; }                        │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  LLVM IR  - Platform-independent, heavily optimized        │
│  define i32 @add(i32 %a, i32 %b) { ... }                   │
└─────────────────────┬───────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  Machine Code  - x86, ARM, etc.                            │
└─────────────────────────────────────────────────────────────┘
```

---

## Summary

| Representation | Key Role |
|----------------|----------|
| **HIR** | Understanding *what* your code means (types, names) |
| **MIR** | Checking *how* it behaves (ownership, control flow) |
| **LLVM IR** | Final optimization and code generation |

LLVM IR lets Rust benefit from decades of LLVM optimization passes (inlining, vectorization, dead code elimination, etc.) without Rust having to implement them directly.

---

## Additional Resources

- [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/)
- [LLVM Language Reference Manual](https://llvm.org/docs/LangRef.html)
- [MIR RFC](https://rust-lang.github.io/rfcs/1211-mir.html)
