---
id: T-4699
title: "Probe and implement pivot blank rows and collapsed items beyond an outer row field"
milestone: M16
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4168]
---

## Goal
The engine's readings of `insertBlankRow` and collapsed items (`item@sd="0"`) beyond what probes/t4168-pivot-layout-options shows are evidenced, and corrected where Excel differs.

## Context
Split from T-4168, which implemented the probed shapes (crates/rustsheet-engine/src/pivot/layout/lines.rs `blank_line`, `collapsed`; pivots.md "Layout lines"): a blank line after each item of an outer row field (BR tabular, BR2 compact) and a collapsed outer row item standing alone with its values (SD compact, SD2 tabular). Engine readings left UNVERIFIED (pivots.md, T-4699):
- `insertBlankRow` on the innermost row field (the engine adds no line) and on a column field (none);
- `insertBlankRow` after a collapsed item (the engine adds the blank line);
- a collapsed item on a column field (one column line, the item's tuple) and in a field with `subtotalTop="0"` or custom subtotals (the item's line shows its data field's own aggregate);
- a collapsed item in a middle field of three row fields.
- a collapsed item of a field with the Values field inside it (several data fields on rows): the engine gives it one line with no data field chosen, so the first data field's values;
- custom subtotal lines in plain outline form (`outline` on, `compact` off): the engine puts them after the children, as compact form does (t4168 F2).
Write `probes/t4699-...` with the case builder (`collapsed`, `attrs.insertBlankRow`) and fill it with the excel-graph oracle.

## Acceptance criteria
- [x] A filled probe case covers each reading above; pivots.md records the result with the evidence cells.
- [x] The engine's lines and rendered cells pass the probe in unit tests against the sidecar (crates/rustsheet-engine/src/pivot/layout/{lines,render}/tests.rs).
- [x] `make check` green; reviewers approve.

## Out of scope
- Repeat item labels (T-4698).

## Outcome
- New probe probes/t4699-pivot-blank-collapsed (excel-graph, 2026-11-07), 17 pivot sheets (16 for
  the readings, plus CR, all eleven custom subtotals listed in reverse); recorded in pivots.md "Blank rows and collapsed items beyond an outer row field", and the
  "Layout lines" UNVERIFIED list for T-4699 is cleared.
- Confirmed as the engine had them: no blank line for `insertBlankRow` on the innermost row field (BI,
  BI2) or on a column field (BC); a blank line after a collapsed item (BS, BS2); a collapsed item is
  one line on columns (CC), in a middle field (CM, CM2), under `subtotalTop="0"` (CB) and with custom
  subtotals (CU, CU2), showing the data field's own aggregate; custom subtotals in plain outline form
  follow the children (OC).
- Fixed in the engine (lines.rs, test-first against the sidecar):
  - custom subtotal lines come in the functions' fixed order (`Aggregate` now derives `Ord`), not the
    field's: `max` before `count` (Count Numbers) (CU, CU2), all eleven in order when listed in
    reverse (CR);
  - a collapsed item keeps the Values level inside it: one line per data field with the item's
    totals, no subtotal lines, the field levels in between hidden, written `x="1048832"` in tabular
    form and counted as repeated (`r="2"`) in compact form (CV, CV2, CV3, CV4). chart.rs rebuilds such
    a tuple and gives the hidden level an empty label (UNVERIFIED, T-4938).
- Follow-ups: T-4993 (probe a collapsed middle field with Values deeper, and insertBlankRow on a
  field with Values inside); T-4938 gains the chart shapes of a collapsed item with Values inside;
  T-4927 is closed as done here (its cells now pass).
- Eval: gen-pivot known failures 3125 -> 3077 (85 cells fixed, gen-pivot/36996978427905b2 P05, the
  cells T-4927 owns), 0 regressions; the new case's one visibly-off cell, CV4!C3 (the `Values`
  header in tabular form), is T-4914's (owner line added there).

