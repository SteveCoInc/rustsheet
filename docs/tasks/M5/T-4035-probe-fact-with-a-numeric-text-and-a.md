---
id: T-4035
title: "Probe FACT with a numeric-text and a multi-cell-range number argument (the M5 argument kinds T-2862 leaves)"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FACT's `number` has an Excel-backed cell for a numeric text and for a multi-cell range or array constant, the two
argument kinds the report's argument coverage misses that no open task lists. With T-2862's empty text and error,
FACT then reaches `covered == expected`.

## Context
- Gap triage 2026-11-04, second opus pass (`make eval` on main 1fb8465f, excel-graph evidence; report.json
  `argument_coverage`, decision 0031 evidence gaps; decision 0094 wants every argument mode covered). Every failing
  cell of that run has an open owner, and the evidence floor is met. M5 is the earliest milestone with an argument
  kind no open task lists.
- Coverage on that run (kinds of `crates/rustsheet-eval/src/argument_coverage.rs`, `classify_value`): FACT.number
  (mode `value`) has 7 of 11. It has number, zero, negative, fraction, text, boolean and blank, and lacks numeric text,
  empty text, an error and a range. T-2862 lists FACT.number's empty text and error only, so this task takes the
  other two.
- A kind counts only when an Excel-evidence cell passes it as a literal or a single-cell reference. A nested call is
  not classified (docs/progress.md "Argument coverage"). A range counts as a multi-cell reference, a 3-D reference
  or an array constant.

Write one probe case, probes/m5-argument-kind-gaps-fact (excel-probe skill, excel-graph, tag M5):
- numeric text: `=FACT("5")`, `=FACT(" 4 ")`, `=FACT("3.7")`, `=FACT("1e1")`, and `=FACT(A1)` with A1 the text `6`;
- range: `=FACT({3,4,5})` and `=FACT(B1:B3)` with B1:B3 holding 2, 3 and 4. Both spill in dynamic-array Excel, so
  check their whole footprints. Add `=@FACT(B1:B3)` beside them for the implicit-intersection reading.

## Acceptance criteria
- [ ] probes/m5-argument-kind-gaps-fact is committed with excel-graph oracle values and covers both parameter/kind
      pairs above. After `make eval`, report.json `argument_coverage` shows FACT.number's numeric text and range
      kinds as seen.
- [ ] Each cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in FACT's own
      module) or listed in a new functions-math task (campaign: semantics, with a `## Visible-parity stage` section)
      filed with `make new-task` that names the case and cells. The new known failures are recorded with
      `make eval-baseline` and each one has that owner (decision 0245's new-case gate).
- [ ] Any new Excel rule goes into docs/excel-semantics/numbers.md or coercion.md, citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- FACT.number's empty text and error (T-2862), and FACTDOUBLE (T-2862).
- The other M5 parameters T-2862, T-2863, T-2871, T-2872 and T-3965 list.
