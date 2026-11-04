---
id: T-4048
title: "Fuzz bitcode_workbook timeout: decoded iteration settings take any count and delta, so a one-cell cycle sweeps for minutes"
milestone: M12
lane: engine
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
A workbook decoded from bytes cannot ask for more iterative-calculation sweeps than Excel allows:
its iteration count and delta are brought into Excel's range at load by the same rule the xlsx
importer uses, and both nightly `bitcode_workbook` timeouts load and evaluate in milliseconds.

## Context
The nightly fuzz job (`.github/workflows/fuzz.yml`) failed on `bitcode_workbook` on 2026-11-02
(run 41384443922) and 2026-11-03 (run 41573652529). Both are libFuzzer timeouts (more than 30 s on
one input), not panics. The stack is `Model::evaluate` → `iterate_cycles` → `sweep` →
`evaluate_cell`. Directive 5 and decision 0035 make each one a P0.

Both inputs are small workbooks with iteration on, a self-referencing cell (`R[-0]C[0]+1`,
`R[-0]C[0]*2`) and a count decoded straight from the bytes: 33,558,315 and 50,529,030, with a
delta of 0 and 3.7e-294, so the cycle never converges. `iteration_from`
(crates/rustsheet-xlsx/src/import/workbook.rs) limits an xlsx count to 1 to 32,767, but
`Model::from_workbook` took any `Iteration`. The only cap left was `MAX_SWEEP_EVALUATIONS` (2^25
member evaluations). A cheap member like `=A1+1` spends almost none of the 2^32-unit ceiling of
SPEC-0028, so that ceiling did not bind either. One recalculation took 34.4 s and 6.7 s in a plain
release build.

Reproducing: under `cfg(fuzzing)`, bitcode 0.6.9 does not refuse trailing bytes
(`consume.rs`: `if cfg!(not(fuzzing)) && !input.is_empty()`). So the saved artifacts decode only
in a fuzz build. Their first 377 and 388 bytes decode everywhere and are the reproducers in
`test_iteration_settings.rs`.

## Acceptance criteria
- [x] One definition of Excel's iteration range (`Iteration::within_excel_range`), used by the
      xlsx importer and by `Model::from_workbook` (directive 3).
- [x] Both fuzz reproducers load with a count of 100 and evaluate 100 sweeps (unit tests).
- [x] The raw CI artifact no longer times out under `cargo fuzz run bitcode_workbook`.
- [x] Quirk recorded in docs/excel-semantics/recalc.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- An Excel-valid workbook can still be slow: a 1,024-cell cycle that never converges, with a count
  of 32,767, runs the full 2^25 sweep evaluations, 7.5 s per recalculation in a release build.
  Bounding sweep time (a lower `MAX_SWEEP_EVALUATIONS` or a per-run charge against the ceiling) is
  a separate task.
- The steward's daily run does not check nightly fuzz results (a separate task).
- `rustsheet::Workbook::set_iteration` (API callers are trusted).

## Outcome
**Fix.** `Iteration::within_excel_range` (iteration.rs) replaces a count outside 1 to 32,767
(`Iteration::MAX_ITERATIONS`) with 100, and a delta that is not finite and non-negative with
0.001. Each setting falls back on its own. `Model::from_workbook_with_clock` applies it right after
the index check (T-2913). `iteration_from` now parses each attribute and calls the same function,
so the importer has no copy of the range. Its behavior is unchanged.

**Tests.**
- `test_iteration_settings.rs`:
  - both reproducers load with a count of 100, and their cycles end where 100 sweeps leave them
    (night two's `=A2+1` is 102);
  - each setting's fallback on its own, both ends of the count's range kept;
  - a save and reload keeps in-range settings, and iteration off stays off.
- `import/workbook.rs` gains a unit test of `iteration_from`: out-of-range, unparsable and absent
  attributes.

**Fuzz replay.** The raw night-two artifact (`timeout-03955d2d…`) and the night-one input, run
through `cargo fuzz run bitcode_workbook` with ASan, finish without a timeout.
