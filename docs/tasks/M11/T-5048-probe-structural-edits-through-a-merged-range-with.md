---
id: T-5048
title: "Probe structural edits through a merged range with covered content on excel-graph (SPEC-0036 S4)"
milestone: M11
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Excel evidence for the SPEC-0036 rows that excel-graph can answer: spill into a merged range, save of covered content, and row/column deletes through a merge with covered content.

## Context
Spec: docs/specs/SPEC-0036-merged-covered-content.md, rows X3, X4, X6, X8 (rollout S4). The oracle replays `insert_row`,
`delete_row`, `insert_column` and `delete_column` sheet ops (docs/architecture/oracle.md "Sheet ops the excel-graph backend
replays"); spec2xlsx has no merged ranges today (tools/src/rustsheet_tools/spec2xlsx.py, spec_sheet_extras.py), so either add a
`merged` sheet extra or hand-build xlsx fixtures as T-4148 did (fixtures/merged-covered-values). Questions: (X6) delete the anchor
row and the anchor column of a merge whose covered cells hold values, and read the surviving cells and the merge; (X3) a spill whose
range meets an empty covered cell, the anchor, and a covered cell with content; (X4) the saved XML of a covered value and formula;
(X8) whether a Graph Range `unmerge` op is in reach as a new sheet op (decide; adding it is its own task).

## Acceptance criteria
- [ ] Probe cases filled on excel-graph for X3 and X6 (and X4 if the oracle's returned file can be read for it).
- [ ] Findings recorded in docs/excel-semantics/xlsx-storage.md; SPEC-0036's rows updated from UNVERIFIED where evidenced; a task
      filed for any row where Excel differs from the spec's rule, owning the visibly-off cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Paste, autofill, Format Painter and UI merge (no Graph endpoint; SPEC-0036 X7, X9).
