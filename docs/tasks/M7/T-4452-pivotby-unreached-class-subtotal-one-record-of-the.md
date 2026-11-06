---
id: T-4452
title: "PIVOTBY unreached-class subtotal: one record of the rule after T-4388 (T-4401 overlap, column_records parent sets, T-4388 Outcome)"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4388]
---

## Goal
The unreached-class PIVOTBY column-subtotal rule has one record and one owner once T-4388 (#3430) lands. The docs no
longer say that a satisfied task owns it.

## Context
- These MINOR findings come from the T-4388 orphan-adoption review (PR #3430, head ef1c8cce1, design and
  excel-fidelity reviewers).
- T-4388 adds `Grouping::class_reached`. With it, a column subtotal of a class no record reached holds every record of
  its row line (`Grid::cell_rows`, `Slot::Subtotal` arm).
- The rule also fixes the 9 level-2 cells that T-4401 owned: probes/t4382-pivotby-intransitive-paths Calc!B31, B32,
  B33, B35, B39, B41, C31, C33 and C39. The baseline dropped them in #3430.
- PR #3448 (T-4401) implements the same rule a second way, `Grouping::subtotal_reached`, and adds a hold-out probe
  (probes/t4401-pivotby-level-subtotal-holdout). If #3448 lands too, keep one helper. Its hold-out case is worth
  keeping.
- docs/excel-semantics/coercion.md, the T-4382 section's level-2 bullet, still says T-4401 owns these cells.
- `Grid::column_records` (pivotby.rs, used for the parent sets) says it follows "[`Grid::cell_rows`]'s rule". Its
  `Line::Subtotal` arm still returns `reached_class` only, with no `None` (every record) for an unreached class.
  probes/t4420-pivotby-parent-sets-three-fields has 14 known failures there, owned by T-4428, which "waits on T-4401".
- The Outcome of the T-4388 task file says "Other cases: no change". Its first acceptance box was also left unticked.

## Acceptance criteria
- [ ] coercion.md's T-4382 level-2 bullet names T-4388's rule and its two datasets: t4388 and the 9 t4382 cells.
- [ ] T-4401 is closed, or rescoped to only its hold-out probe. Exactly one helper exists for "class reached"
      (`rg class_reached\|subtotal_reached` finds one definition).
- [ ] T-4428's `depends_on`/Context rests on `Grouping::class_reached`. Otherwise the doc comment of
      `Grid::column_records` says that it does not follow the unreached-class rule yet.
- [ ] The Outcome of T-4388 records the 9 extra t4382 cells as out-of-sample confirmation.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the parent-set rule itself (T-4428).
