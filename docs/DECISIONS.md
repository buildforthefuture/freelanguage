# Bayan Design Decisions

All deviations from, or clarifications of, docs/SPEC.md are recorded here with
the reason. This is the single source of truth for implementation choices.

## D001 — Expression style: method-style ops
Chosen: `x.add.wrap(y)`, `a.lt.s(b)`.
Reason: reads naturally, uniform grammar, easy for LLMs to generate, LL(1)-friendly.
Prefix `op(x, y)` is NOT supported (kept simple).

## D002 — NaN canonicalization
Every float arithmetic result that is NaN becomes a single canonical quiet NaN:
f32 `0x7FC00000`, f64 `0x7FF8000000000000`.
Reason: determinism — same input must produce byte-identical output.

## D003 — Default literal types
Integer literal without suffix → `i64`. Float literal without suffix → `f64`.
Reason: simplest, matches common usage.

## D004 — Struct layout
`align(N)` is the required alignment of accesses, NOT implicit padding. Fields
are laid out at declared offsets in declaration order, packed. Programmer is
responsible for padding.
Reason: explicit, low-level, no hidden layout decisions.

## D005 — Division/remainder traps
div/rem by zero → E_TRAP_DIVZERO. INT_MIN / -1 → E_TRAP_DIVOVF (signed only).
Reason: defined behavior, no UB, matches spec.

## D006 — Shift modes
`shl.mask` = amount mod bitwidth; `shl.trap` = trap if amount >= bitwidth.
Same for shr.u / shr.s.
Reason: explicit policy, no UB.

## D007 — Heap allocator
Deterministic first-fit, 8-byte header `[size:u64][free:u8][pad:7]`, free list
ordered by address, coalescing on free, split if remainder >= 16.
Reason: reproducible, simple, specifiable exactly.

## D008 — Stack alloca
`alloca(size, align)` inside a function, freed at return. Modeled as a stack
pointer bump in the interpreter.
Reason: simple, deterministic.

## D009 — Uninitialized memory
Zero-filled everywhere (globals, heap, stack).
Reason: deterministic.

## D010 — Atomics
Single-threaded interpreter; all orderings treated as sequentially consistent.
Full threading out of scope for v0; syntax reserved.
Reason: v0 scope.

## D011 — Asm
Interpreter traps E_TRAP_ASM_UNSUPPORTED unless registered emulation exists.
Emulations: `rdtsc-fake` (deterministic counter), `nop`, `x86.add64`.
Target dialect ops `x86.*`, `arm.*` parseable + type-checked; small subset
emulated.
Reason: safe isolation of raw hardware access.

## D012 — Binary format
Magic `BYN\0`, version 1. Sections: types, globals, functions, code, debug,
source-map. Round-trip text→binary→text lossless including comments/spans in
debug section.
Reason: deterministic, inspectable.

## D013 — No implicit conversions
All casts explicit ops. Signedness in operation, not type.
Reason: explicit side effects, easy to verify.

## D014 — Errors
Every verifier error has a stable code E0001+ documented in docs/ERRORS.md.
Diagnostics include file:line:col, code, hint.

## D015 — Implementation language
Rust stable, no unsafe in compiler. External crates: `clap`, `proptest` only.
Reason: safety, simplicity.
