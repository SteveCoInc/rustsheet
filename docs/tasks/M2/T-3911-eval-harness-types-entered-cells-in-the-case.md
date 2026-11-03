---
id: T-3911
title: "Eval harness types entered cells in the case file's key order, as the oracle does"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3898]
---

## Goal
The eval harness types a case's entered cells in the order the oracle types them, so results that depend on the typing
order match Excel.

## Context
- Follow-up of T-3898 (PR #2950), from its Excel-fidelity review (MINOR finding). Since T-3898 each entered cell is its own
  edit (`type_entries`, crates/rustsheet-eval/src/engine.rs), so the order matters: a cycle typed one cell at a time keeps the
  first cell's value (probes/t3898-typed-circular-readers H17:H18; docs/excel-semantics/recalc.md, "A circular formula
  typed after the load leaves its readers their values from before").
- The oracle types entries in spec order, the JSON key order of each sheet's `cells`. `spec2xlsx.build` walks
  `sheet["cells"].items()` and records `rustsheet.entered.<n>`, and the Graph and COM backends loop over
  `typed_entries(xlsx)` in that order (tools/src/rustsheet_tools/oracle/backends.py). The harness reads `cells` into a
  `BTreeMap` and sorts the entries row-major (`row_major`).
- Today only 2 cases list entries out of row-major order (probes/t2786-whitespace-structured-refs-runs,
  probes/date-text-single-letter-ap), and neither has a typed cycle, so no result differs yet.
- Options: keep spec order in the harness (`CellInput` map order), or make spec2xlsx record entries in row-major order and
  have the case loader refuse entries listed out of that order. Also check whether the oracle types sheet by sheet.
  Then fix the `type_entries` doc comment and recalc.md's "Ours:" line, which both say row-major.

## Acceptance criteria
- [ ] The harness and the oracle type entered cells in the same order, and a test pins that order (a cycle typed bottom-up).
- [ ] The `type_entries` doc comment and recalc.md's "Ours:" line say the order truthfully.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine's circular-value rule.
