---
id: T-4230
title: "Stored xlsx numbers read as Excel reads them: the 47 one-ulp probes-numeric/t3658-round-scaling Hits!G cells (=F1 of a stored 0.002877 is 0.0028770000000000002)"
milestone: M2
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4229]
defer: phase-1b
---

## Goal
A constant cell loaded from xlsx holds the double Excel holds for the same `<v>` text, so the 47 one-ulp cells of
probes-numeric/t3658-round-scaling Hits!G pass bit-exact, with the rule backed by T-4229's sweep.

## Context
- Gap triage 2026-11-04 (`make eval` on 726980168). These 47 cells had T-3932 as their only owner, which listed them
  as out of scope and is done, so they were unowned. Failing cells (all NumericUlp, one ulp, within 15 digits, hence
  `defer: phase-1b`): probes-numeric/t3658-round-scaling Hits!G1:G6, G8, G10:G11, G14:G16, G18:G19, G22:G26,
  G29:G41, G45:G47, G49:G58, G60:G61. Examples: G1 `=F1` with F1 stored 0.002877, Excel 0.0028770000000000002,
  ours 0.002877; G8 (4.91E-06) Excel 4.9100000000000004e-6, ours 4.91e-6; G14 Excel 0.00013057999999999999, ours
  0.00013058.
- Excel's value is the literal x87 read (numbers.md "Decimals reach a double through an x87 multiply by 10^-n",
  `x87_text_to_f64` in crates/rustsheet-engine/src/decimal_round.rs) at 46 of 61 Hits!G points with the digits
  as written, 51 with trailing zeros stripped; the misses are G9, G12:G13, G20:G21, G27:G28 and G42:G44, which
  pass today on the nearest double. numbers.md "Stored numbers in an xlsx are not always parsed to the nearest
  double" (T-1778, OPEN) has 7 more misreads at longer texts. So neither the nearest double nor the literal model
  fits every point.
- Needs an oracle sweep: the cached workbooks give 61 points, too few to choose between candidate rules and leave a
  hold-out, so this task depends on T-4229.
- Code: `parse_cell_number` / `parse_constant_number` in crates/rustsheet-xlsx/src/import/util.rs (`str::parse::<f64>`).
  Reuse `x87_text_to_f64` (or whatever T-4229 shows) rather than a second decimal reader (directive 3); the engine
  crate's helper is `pub(crate)`, so exposing it is part of the change. A spill's cached value and a formula's cached
  result are a separate question; keep them on the nearest double unless the sweep covers them.
- Every probe case whose inputs are stored decimals reads through this path, so scan the whole eval for moved cells,
  not only t3658.

## Acceptance criteria
- [ ] The 47 Hits!G cells above pass bit-exact, the 14 Hits!G cells that pass today still pass, and no other cell in
      any suite changes from pass to fail (`make eval`; eval/baseline/probes.txt, ironcalc-calc.txt and the gen
      baselines only shrink).
- [ ] T-4229's held-out case passes at the rate recorded in the Outcome.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), in the Outcome: hold-out counts (train and hold-out
      side by side, bit-exact / within 4 ulp / within 15 digits, for the chosen rule and the nearest double, from
      the split T-4229 fixed before scoring); the fresh sweep result (T-4229's held-out case, generated before this
      fit was frozen); the Gate A envelope result for that sweep, every point outside listed; a
      `FITTED(T-4230, n_cells, range)` tag and range on any fitted piece, with the nearest double outside the range
      (a structural model chosen from ranked candidates owes no tag); the reproducibility floor (no target bit that
      varied between T-4229's two fills of the Hits!F points); no keep-matched branch (no "use the nearest double
      where it already matched" selector).
- [ ] numbers.md "Stored numbers in an xlsx are not always parsed to the nearest double" moves from OPEN to the rule
      found, with Implemented in.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed cell input, VALUE, criteria operands and NUMBERVALUE (T-4074).
- xlsx export of such a value (file a follow-up if Excel's own files write it differently).
