---
id: T-4804
title: "GETPIVOTDATA: several data fields, the Values field, blank and empty data_field, one- and no-record aggregates, overflow (T-2085 evidence)"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: M
tier: opus
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
depends_on: []
---

## Goal
GETPIVOTDATA matches Excel on pivots with several data fields, on blank and empty `data_field` arguments, on aggregates over one record or none, and on totals that overflow.

## Context
T-2085 probed SPEC-0015 unknowns 1, 2, 8, 15 and 16 (docs/excel-semantics/web.md "GETPIVOTDATA on other pivot shapes (T-2085)", the "Several data fields", "Blank and empty
data_field", "Aggregates over one record or none" and "Overflow" paragraphs). The engine
(`functions/lookup_and_reference/getpivotdata.rs`, `pivot_index.rs`) answers `#REF!` for most of these, and the grand
total for a blank cell outside the pivot, where Excel gives `#REF!`.

Owner of these visibly-off cells (decision 0245):
- probes/t2085-getpivotdata-data-fields Q!A1:A5, Q!A8, Q!A12, Q!A13, Q!A15, Q!A17, Q!C6, Q!F6, Q!G5, Q!G6, Q!H6,
  Q!I5, Q!I6, Q!J6, Q!K1:K3, Q!L1:L3

The calculated-field cells probes/t2085-getpivotdata-layouts Q!G1:G3, moved here from T-4174 after this task's PR
was written, moved on to T-5039 (adopted PR #3950's merge of main).

## Acceptance criteria
- [x] The cells above pass, and no other case regresses (`make eval-baseline`).
- [x] SPEC-0015's resolution rules and web.md's "Ours" line say what the engine now does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Item kinds and argument errors (T-4805), and pivot shapes and shown values (T-4806).

## Outcome
GETPIVOTDATA now answers pivots with several data fields (the Values field on an axis), with the Values field as a
pair whose only item is the chosen data field's caption, and a source name shared by two data fields picks the first
(`getpivotdata.rs` `pivot_data_field`, `pivot_index.rs` `supported`/`values_on_axis`). A blank-cell data_field picks
the only data field only when the cell is inside the pivot's location. Every aggregate over no records is 0, and Var and
StdDev over one record are `#DIV/0!` (`pivot/layout.rs` `aggregate`, with a unit test). Overflow is `#NUM!` on the
two-data-field pivot as on one. Which data field wins when one's caption is another's source name is unprobed; the
caption does (SPEC-0015 GETPIVOTDATA step 2, web.md "Ours").

Eval delta: probes 24 fixed (every owned cell of probes/t2085-getpivotdata-data-fields, which now passes 365/365),
0 regressions in every suite; `eval/baseline/probes.txt` re-recorded with `make eval-baseline`.
