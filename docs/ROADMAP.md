# Bayan Roadmap

Phase checklist (done in order; do not start next until current tests pass):

- [ ] P0  SPEC.md, GRAMMAR.md, DECISIONS.md, ROADMAP.md, ERRORS.md; 20 example programs
- [ ] P1  Lexer + parser + pretty-printer; golden tests; fmt idempotency property test
- [ ] P2  IR + lowering (structured → CFG/SSA); IR printer
- [ ] P3  Verifier with full error catalogue; ≥ 60 negative tests (one per error code)
- [ ] P4  Interpreter; run all 20 examples; trap tests for every trap code
- [ ] P5  Binary format + round-trip property tests
- [ ] P6  QBE backend + differential tests vs interpreter
- [ ] P7  CLI polish, `explain`, JSON diagnostics, README with quickstart
- [ ] P8  proptest random program generator → determinism + differential; fix all divergences

Acceptance criteria:
- `cargo test --workspace` passes; `cargo clippy -- -D warnings` clean
- Determinism: any example run 100× → identical output
- No unsafe Rust; no panics on malformed input
- Every opcode has semantics paragraph, edge-case table, interpreter test, codegen test
- Standard demos run: fibonacci, array sum, bubble sort, string length, linked-list on heap, matrix multiply, trap demo, atomic counter, unsafe+asm demo
