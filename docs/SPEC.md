# Bayan Language Specification

Bayan is a low-level, deterministic intermediate language: "assembly with names".
It is the target of a future natural-language → spec → code pipeline. Every
operation has exactly one defined outcome. There is zero undefined behavior.

## Design decisions (summarized)

- Expression style: **method-style ops** (`x.add.wrap(y)`, `a.lt.s(b)`), chosen
  for readability and LL(1)-friendliness. Prefix form `op(x, y)` is also accepted
  for a small set of ops; see GRAMMAR.md.
- NaNs: every floating-point arithmetic result canonicalizes NaN to a single
  quiet NaN (`0x7FC00000` for f32, `0x7FF8000000000000` for f64), so output is
  deterministic.
- Overflow: explicit opcodes per policy (`add.wrap`, `add.trap`, `add.sat`).
- Division: divide-by-zero and INT_MIN / -1 TRAP (defined, not UB).
- Shifts: `shl.mask` / `shl.trap`, `shr.u.mask` / `shr.u.trap`, `shr.s.mask` /
  `shr.s.trap`.
- Memory: single flat little-endian byte-addressable region; OOB traps;
  uninitialized memory is zero-filled; `load.unaligned` always allowed;
  `load.aligned(n)` traps if misaligned.
- Heap: deterministic first-fit allocator (specified exactly below).
- Stack: `alloca(size, align)` inside functions, freed at return.
- Atomics: syntax reserved; interpreter is single-threaded and treats all
  orderings as sequentially consistent for now.
- Atomics trap codes are included but atomics are not yet executed (deferred).

## 1. Lexical structure

### Comments
- Line comment: `// ...` to end of line.
- Block comment: `/* ... */`, may nest. May contain any Unicode.

### Identifiers
ASCII letters, `_`, digits; cannot start with a digit. `[A-Za-z_][A-Za-z0-9_]*`.
Keywords are reserved and may not be used as identifiers.

### Keywords
`fn extern return if else loop break continue var let unsafe asm true false`
`module import export struct align cc pure noinline inline cold volatile`
`trap goto`

### Literals
- Integer: decimal `123`, hex `0xFF`, binary `0b1010`. Suffix optional: `i8`
  `i16` `i32` `i64`. Default type is `i64` if no suffix.
- Float: `3.14`, `1e-5`. Default `f64` if no suffix (`f32` suffix).
- Bool: `true` `false`.
- String: `"..."` with `\n` `\t` `\\` `\"` escapes; valid UTF-8.
- Character: `'a'`, `'\n'`, `'\0'` (i8).
- Null pointer: `null` (type ptr, value 0).

### Whitespace
Spaces, tabs, newlines, `\r`, form feed.

## 2. Types

| Type | Size | Description |
|------|------|-------------|
| `i8` | 1 | 8-bit integer |
| `i16` | 2 | 16-bit integer |
| `i32` | 4 | 32-bit integer |
| `i64` | 8 | 64-bit integer |
| `bool` | 1 | i1, `true`/`false` |
| `f32` | 4 | IEEE-754 binary32 |
| `f64` | 8 | IEEE-754 binary64 |
| `ptr` | 8 | flat 64-bit address |
| `unit` | 0 | no-return / void |

Aggregate types:
- `struct Name { f1: T1, f2: T2 } align(N)` — explicit layout, fields at
  declared offsets (packed, no implicit padding; `align` is the required
  alignment of accesses, not padding).
- `[T; N]` — fixed array.

Signedness is in the operation, not the type. No implicit conversions. All
casts are explicit ops.

## 3. Expressions

Expression style is method-style: `receiver.op(args)`. Examples:
`x.add.wrap(y)`, `a.lt.s(b)`, `load.unaligned<i32>(ptr)`.

Binary ops take the receiver as the left operand and a parenthesized argument
list as the right: `lhs.op(rhs)`. Unary: `op(x)`.

### Integer arithmetic (overflow policy in opcode)

| Op | Meaning | On overflow |
|----|---------|-------------|
| `add.wrap(a,b)` | a + b | wraps mod 2^N |
| `add.trap(a,b)` | a + b | traps E_TRAP_OVERFLOW |
| `add.sat(a,b)` | a + b | saturates to min/max |
| `sub.wrap/trap/sat`, `mul.wrap/trap/sat` | same policies |

Signedness: arithmetic ops are signed by default; unsigned variants exist where
it matters: `mul.wrap.u` (unsigned multiply wraps identically; provided for
symmetry), `div.s` / `div.u`, `rem.s` / `rem.u`, `lt.s` / `lt.u`, etc.

### Division and remainder

- `div.s(a,b)`, `div.u(a,b)`: trap E_TRAP_DIVZERO if b == 0; trap E_TRAP_DIVOVF
  if a == INT_MIN and b == -1 (signed only).
- `rem.s(a,b)`, `rem.u(a,b)`: same trap conditions; sign of remainder follows
  dividend for signed.

### Shifts

- `shl.mask(a, amt)`: a << (amt mod N)
- `shl.trap(a, amt)`: trap E_TRAP_SHIFT if amt >= N
- `shr.u.mask/shr.u.trap`: logical right shift
- `shr.s.mask/shr.s.trap`: arithmetic right shift

### Bitwise

`and`, `or`, `xor`, `not`, `clz`, `ctz`, `popcnt`, `rotl`, `rotr`, `bswap`.
`clz`/`ctz` of 0 return the bitwidth.

### Comparisons (integers)

`eq`, `ne`, `lt.s`, `lt.u`, `le.s`, `le.u`, `gt.s`, `gt.u`, `ge.s`, `ge.u`.
Result is `bool`.

### Comparisons (floats)

Ordered variants trap on NaN input? No — floats never trap. `f.lt.o` (ordered
less-than: false if either operand NaN), `f.lt.u` (unordered less-than: true if
either NaN). Variants: `eq.o`/`eq.u`, `ne.o`/`ne.u`, `lt.o`/`lt.u`, `le.o`/
`le.u`, `gt.o`/`gt.u`, `ge.o`/`ge.u`.

### Conversions (all explicit)

`zext<T>(x)`, `sext<T>(x)`, `trunc<T>(x)`, `fptosi<T>(x)`, `sitofp<T>(x)`,
`fptou<T>(x)`, `uitofp<T>(x)`, `bitcast<T>(x)`, `ptr2int(x)`, `int2ptr(x)`.

- `fptosi`/`fptou`: out-of-range or NaN → 0 (saturating-to-zero for NaN).
- `sitofp`/`uitofp`: exact or rounded to nearest even.
- `trunc<T>`: drops high bits.
- `bitcast<T>`: reinterprets bits; sizes must match.
- `ptr2int` / `int2ptr`: identity on bits.

## 4. Statements and control flow

Surface syntax is structured: `if/else`, `loop`, `break`, `continue`, `return`,
`switch`. Labeled blocks and `goto label` are also supported. Everything lowers
to a CFG of basic blocks with explicit terminators; SSA uses block parameters
(no phi nodes in text).

Statements:
- `var x: T = init` — mutable local (lowered to SSA).
- `let x: T = init` — immutable local (same lowering, single assignment).
- `x = expr` — assignment to an existing mutable variable.
- `expr;` — expression statement (side effect only).
- `if cond { ... } else { ... }`
- `loop { ... }` with `break [value]` and `continue`.
- `return expr;`
- `switch expr { case v => { ... } ... default => { ... } }`
- `label: { ... }` — labeled block; `goto label` jumps to it.
- `trap(code);` — explicit trap.

## 5. Functions

```
fn name(param: T, ...) -> Ret [attrs] { body }
```

Attributes: `noinline`, `inline`, `pure` (verified side-effect-free), `cold`.
`extern fn` declares an external function (host builtin or foreign).
`export fn` marks a function as exported (visible to linker).

Calling convention: `cc(sysv64)`, `cc(win64)`, `cc(bayan)`. Default `cc(bayan)`.

### Bayan calling convention (cc(bayan))

- Integer/ptr args passed in order: rdi, rsi, rdx, rcx, r8, r9, then stack
  (sysv64-style for simplicity; win64 uses rcx, rdx, r8, r9).
- Return value in rax.
- Callee-saved: rbx, rbp, r12-r15.
- Stack 16-byte aligned at call site.

This is simplified; QBE handles lowering to native.

## 6. Memory model

Single flat little-endian byte-addressable memory per program instance.
Regions: globals, heap, stack. Memory is bounded; out-of-bounds traps
(E_TRAP_OOB).

- `load.unaligned<T>(addr)`: read T bytes at addr; always allowed (no
  alignment check). Traps if any byte OOB.
- `load.aligned<N<T>(addr)`: read T; traps E_TRAP_ALIGN if addr % N != 0;
  traps E_TRAP_OOB if OOB.
- `store.unaligned<T>(addr, val)`: write T bytes; always allowed alignment.
- `store.aligned<N<T>(addr, val)`: align check then write.
- `alloca(size, align)`: allocate on current stack frame; freed at return.
- `alloc(size)`: heap allocate (first-fit). Returns ptr or null on failure.
- `free(ptr)`: free heap allocation. Freeing null is a no-op. Freeing invalid
  ptr is defined: no-op (deterministic allocator).
- `realloc(ptr, size)`: first-fit realloc.

### First-fit heap algorithm (exact)

Heap starts at a fixed base address with a fixed size (default 1 MiB, set by
`--max-memory` or `HEAP_SIZE` env). Each allocation has an 8-byte header:
`[size: u64][free: u8][pad: 7]`. Free list is a singly-linked list ordered by
address. `alloc(n)`:
1. Round n up to 8.
2. Walk free list in address order; first block with size >= n wins.
3. Split if remainder >= 16 (header + min payload); else give whole block.
4. Mark used, return pointer after header.
5. If no block fits, return null.

`free(p)`:
1. If p == null, return.
2. Read header at p-8; if already free or header corrupt, no-op.
3. Mark free, coalesce with adjacent free blocks (prev and next by address).
4. Insert into free list in address order.

`realloc(p, n)`:
1. If p == null, `alloc(n)`.
2. If n == 0, `free(p)`, return null.
3. Read old size. If new size <= old size, keep block (shrink, no split).
4. Else `alloc(n)`, copy min(old,new) bytes, `free(p)`, return new.

Uninitialized memory is zero-filled at allocation and globally.

## 7. Atomics

Syntax: `atomic.load<T>(addr, ordering)`, `atomic.store<T>(addr, val, ordering)`,
`atomic.rmw<T>(addr, op, val, ordering)`, `atomic.cmpxchg<T>(addr, exp, des, ordering)`.
Orderings: `relaxed`, `acquire`, `release`, `seq_cst`.

The reference interpreter is single-threaded; all orderings are treated as
sequentially consistent. Full threading is out of scope for v0 but syntax is
reserved.

## 8. Traps

Trap terminates the program with a deterministic status code and message.

| Code | Name | Meaning |
|------|------|---------|
| 0 | E_TRAP_NONE | no trap (normal exit) |
| 1 | E_TRAP_OOB | out-of-bounds memory access |
| 2 | E_TRAP_ALIGN | alignment violation |
| 3 | E_TRAP_DIVZERO | division by zero |
| 4 | E_TRAP_DIVOVF | INT_MIN / -1 |
| 5 | E_TRAP_OVERFLOW | arithmetic overflow (trap policy) |
| 6 | E_TRAP_SHIFT | shift amount >= bitwidth |
| 7 | E_TRAP_ALLOC | allocation failure |
| 8 | E_TRAP_USER | explicit `trap(code)` |
| 9 | E_TRAP_ASM_UNSUPPORTED | asm without emulation |
| 10 | E_TRAP_UNREACHABLE | reached unreachable block |

Exit status = trap code (non-zero = trap). Message printed to stderr.

## 9. Safety levels and unsafe

Everything above is "safe" (deterministic). Raw hardware access is isolated:

- `unsafe { ... }` blocks permit: `int2ptr` from arbitrary values, raw syscalls
  (deferred), and `asm`.
- `asm volatile("template" : outs : ins : clobbers)` — verifier checks operand
  types; interpreter traps E_TRAP_ASM_UNSUPPORTED unless registered emulation
  exists.
- Target dialect ops `x86.*`, `arm.*` are parseable and type-checked from a
  table; interpreter emulates only a small subset (documented in GRAMMAR.md).

### Asm emulations (demo)
- `rdtsc-fake`: no inputs, one i64 output → returns a deterministic counter
  (incremented per execution).
- `nop`: no effect.
- `x86.add64`: `out = in1 + in2` wrapping i64.

## 10. Text format example

See `docs/GRAMMAR.md` for the full EBNF. The demo program from the project
goal must parse, verify, run, and compile:

```bayan
module demo
extern fn print_i64(v: i64)

fn sum_array(data: ptr, len: i64) -> i64 {
  var total: i64 = 0
  var i: i64 = 0
  loop {
    if i.ge.s(len) { break }
    let x: i32 = load.unaligned<i32>(data + i * 4)
    total = total.add.wrap(sext<i64>(x))
    i = i.add.wrap(1)
  }
  return total
}
```

## 11. Binary format

Magic `BYN\0`, version u32 = 1. Sections: types, globals, functions, code,
debug, source-map. `text -> binary -> text` must round-trip losslessly
(including comments and spans in the debug section).
