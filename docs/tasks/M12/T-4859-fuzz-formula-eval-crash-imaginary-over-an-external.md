---
id: T-4859
title: "Fuzz formula_eval crash: IMAGINARY over an external-looking range trips the scalar-context debug_assert"
milestone: M12
lane: engine
status: in-progress
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
`=IMAGINARY('[thoerANCRRAY(1'!A1:B2)` evaluates without a panic in debug builds too, and the cell
takes the value evidence supports.

## Context
Found by the nightly `formula_eval` fuzz run of 2026-11-05 (run 42019157715, job "fuzz
(formula_eval)", artifact `crash-294a86cb835463717474ce6b843b1d95131660e6`). It ran before T-4490's
issue route merged, so no issue was opened and no task was filed until the owner asked.

The 33-byte input is the formula `IMAGINARY('[thoerANCRRAY(1'!A1:B2)` (base64
`SU1BR0lOQVJZKCdbdGhvZXJBTkNSUkFZKDEnIUExOkIy`). The target enters it in C3 of the fuzz model
(fuzz/fuzz_targets/formula_eval.rs). It panics at the `debug_assert!` in `Model::evaluate_cell`
(crates/rustsheet-engine/src/model.rs): "Larger-than-1x1 array reached scalar-context cell
(sheet=0, row=3, column=3, 2x2)". T-3467 hit the same assert by another path.

On current main (57c4e875f), in a debug build through the facade:
- `IMAGINARY('[thoerANCRRAY(1'!A1:B2)` panics;
- `'[x'!A1:B2` and `SIN('[x'!A1:B2)` are `#BLOCKED!`;
- `IMAGINARY(A1:B2)` is `#VALUE!`;
- `IMAGINARY(Nope!A1:B2)` is `#REF!`.

Release builds give `#VALUE!` ("Array result in scalar context") for the crashing formula.

## Acceptance criteria
- [ ] A unit test enters the fuzz input in the fuzz model's C3 and evaluates it in a debug build
      without a panic, with the value evidence supports.
- [ ] The root cause is fixed where it lives, not by removing the assert.
- [ ] Any Excel quirk found is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
