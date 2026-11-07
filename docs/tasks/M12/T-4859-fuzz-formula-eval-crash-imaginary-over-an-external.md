---
id: T-4859
title: "Fuzz formula_eval crash: IMAGINARY over an external-looking range trips the scalar-context debug_assert"
milestone: M12
lane: engine
status: done
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

The 33-byte input is the formula `IMAGINARY('[thoerANCRRAY(1'!A1:B2)`. The target enters it in C3 of the fuzz model
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
- [x] A unit test enters the fuzz input in the fuzz model's C3 and evaluates it in a debug build
      without a panic, with the value evidence supports.
- [x] The root cause is fixed where it lives, not by removing the assert.
- [x] Any Excel quirk found is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- An IF or CHOOSE that picks a reference into another workbook as a `LiftArrays` argument
  (`EDATE(IF(TRUE,…!A1:B2),1)`). It is not probed, and the fuzz input does not need it.

## Outcome
**Root cause.** The bug was wider than IMAGINARY and the fuzzer's odd sheet name. Every
`LiftArrays` function (EDATE, the IM* functions, COMPLEX, ISEVEN, and so on) over any multi-cell
reference into another workbook hit it, `[x]Sheet1!A1:B2` included.
- **Static analysis** (`lifted_static_result`) treats such a range as cells, so the argument is
  refused and the call is a scalar.
- **At run time**, `evaluate_as_reference` refused only a `Range` result and a 3-D span. The
  external range (`Node::WrongRangeKind` naming another workbook) was read as a value: a 2x2
  array of `#BLOCKED!` that the call lifted over.

The two disagreed. A debug build hit the scalar-context `debug_assert!`; a release build gave
`#VALUE!` ("array result in scalar context") by accident.

**Probe.** `probes/external-ref-no-link-lift-arrays` (excel-graph, 19 cells) found two rules:
- **Several cells** are refused like a written range: `#VALUE!`, ERROR.TYPE 3, ROWS `#VALUE!`.
- **One cell** reads empty, as the cell INDEX picks: `EDATE(…!A1,1)` is 31, `IMAGINARY` 0,
  `ISEVEN` TRUE, `COMPLEX(…,1)` "i". We gave `#BLOCKED!` there: a second mismatch, also fixed.

Recorded in references.md, "An Analysis ToolPak argument refuses a multi-cell reference into
another workbook".

**Fix.** `evaluate_as_reference` (functions/lift/call.rs) now reads such a reference through the
existing `external_reference` and `external_picked_value` (directive 3):
- one cell is the value INDEX would pick (empty, or the cached value of a linked workbook);
- several cells, or several areas, are refused (`refused_range`);
- the reference's own error is passed on.

A cached linked range is refused too; before, it was lifted over its cached cells, the same
mismatch.

**Tests.** `test_external_links.rs` `lift_arrays_arguments_read_an_external_reference_as_a_reference`:
- the fuzz input in C3 next to a spill, as the fuzz model has it;
- each probed formula;
- a cached linked workbook: one cell reads its cache, several are refused.

The test panicked before the fix with the fuzz run's message. The probe passes 19/19, and 684
engine tests on external references, lifting, engineering, dates and financial functions pass.
