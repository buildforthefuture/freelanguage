# Bayan Error Codes

Stable verifier/compiler error codes. Each diagnostic includes:
`file:line:col: E####: message` plus a hint.

## Lexer / parser (E01xx)

| Code | Meaning |
|------|---------|
| E0101 | Invalid character in input |
| E0102 | Unterminated string literal |
| E0103 | Unterminated block comment |
| E0104 | Invalid number literal |
| E0105 | Unexpected token |
| E0106 | Expected token (with list of expected) |
| E0107 | Reserved keyword used as identifier |

## AST / names (E02xx)

| Code | Meaning |
|------|---------|
| E0201 | Duplicate definition (function, global, struct, local) |
| E0202 | Undefined name |
| E0203 | Use before definition (local variable) |
| E0204 | Break/continue outside loop |
| E0205 | Goto to undefined label |
| E0206 | Label defined but not reached / duplicate label |

## Types (E03xx)

| Code | Meaning |
|------|---------|
| E0301 | Type mismatch (expected X, got Y) |
| E0302 | Unknown type |
| E0303 | Invalid cast for given types |
| E0304 | Bitcast size mismatch |
| E0305 | Array index out of bounds (constant) |
| E0306 | Struct field does not exist |
| E0307 | Non-constant expression where constant required |

## Verifier (E04xx)

| Code | Meaning |
|------|---------|
| E0401 | Missing return on a control-flow path (function with non-unit return) |
| E0402 | Unreachable terminator (block ends without terminator) |
| E0403 | Wrong call arity |
| E0404 | ABI / calling convention mismatch on call |
| E0405 | Unsafe operation outside `unsafe` block |
| E0406 | Invalid attribute |
| E0407 | Duplicate attribute |
| E0408 | Stack/size overflow at compile-time-known site |
| E0409 | Phi/SSA violation (internal) |
| E0410 | Switch missing default (required) |

## Interpreter (E05xx / trap codes)

| Code | Meaning |
|------|---------|
| E_TRAP_OOB (1)   | Out-of-bounds memory access |
| E_TRAP_ALIGN (2) | Alignment violation |
| E_TRAP_DIVZERO (3)| Division by zero |
| E_TRAP_DIVOVF (4) | INT_MIN / -1 |
| E_TRAP_OVERFLOW (5)| Arithmetic overflow (trap policy) |
| E_TRAP_SHIFT (6) | Shift amount >= bitwidth |
| E_TRAP_ALLOC (7) | Allocation failure |
| E_TRAP_USER (8)  | Explicit `trap(code)` |
| E_TRAP_ASM_UNSUPPORTED (9) | asm without emulation |
| E_TRAP_UNREACHABLE (10) | Reached unreachable block |
