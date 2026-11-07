---
id: T-4875
title: "Probe relative and mixed references in a defined name under column edits, and mixed or half-absolute ranges under row edits (T-4760 follow-up)"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence decides how a defined name's references move under the edits T-4760 left as they were. Then
`Displacement::keep_relative` covers them, or stays narrow, as the evidence shows.

## Context
- T-4760 (PR #3907): probes t4747-name-relative-* and t4760-name-relative-same-sheet-* show that a wholly relative
  `Data!A5` in a name does not move under a row insert or delete. The engine now keeps it (`Displacement::keep_relative`,
  `keeps`, stringify/displacement.rs). reviewer-excel-fidelity blocked widening that past the evidence, so the
  following still displace as before and are unprobed:
  - a relative column under `insert_column` and `delete_column` (the harness can do column ops:
    probes/sheet-ops/t4702-names-insert-column);
  - a mixed reference `$A5` or `A$5` under row and column edits;
  - a range with one absolute and one relative corner (`Data!$A$5:A6`), and a wholly relative range (`Data!A5:A7`)
    under a row insert or delete, including one that spans the edited row;
  - a wholly relative single reference whose own row is deleted (`Data!A5`, `delete_row 5`);
  - the no-edit control for the A1 anchoring.
- Rule and list: docs/excel-semantics/references.md "Relative references in a name".

## Acceptance criteria
- [ ] Probe cases (excel-graph, predictions committed before the fill) cover every form above, each used from another
      sheet and from the edited sheet.
- [ ] The engine matches every probed cell, and references.md states the rule for each form.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Row and column moves (the harness has no move ops; T-4761).
