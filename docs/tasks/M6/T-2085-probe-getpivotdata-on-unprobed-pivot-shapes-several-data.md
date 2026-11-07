---
id: T-2085
title: "Probe GETPIVOTDATA on unprobed pivot shapes: several data fields, empty average, number and date items, filters (SPEC-0015)"
milestone: M6
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
---

## Goal
Excel evidence for the GETPIVOTDATA cases SPEC-0015 leaves unprobed.

## Context
The numbered unknowns of SPEC-0015 "Excel behavior to match" (that list is the scope; do not copy it here). Extend tools/src/rustsheet_tools/pivot.py only as far as each shape needs (filters/hidden items may need the builder to grow; cut a harness task if so). Record findings in docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table".

## Acceptance criteria
- [x] Probe cases with oracle values for each unknown the builder can express; the rest listed as unprobed with why.
- [x] Findings recorded in web.md; T-2083 updated if a rule changes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh, creation, layout in the engine, export (SPEC-0015 non-goals).

## Review history
Rounds 1-3 on PR #3881: security and design APPROVE (round 1); excel-fidelity REQUEST_CHANGES each round on wording
and evidence, every finding fixed, including two real probe gaps (the error-cell pivot_table rule, re-probed as
Q!E20:E23, and tabular/outline form, re-probed with pivotField attributes). After the review-round limit the task
resumed per work-loop step 5c: main merged, baseline re-recorded, and the full panel re-ran; its fidelity findings
(stale "unprobed" sentences in web.md's engine paragraph, SPEC-0015's error-propagation sentence, PD's cache dates)
are fixed.

## Outcome
Three probe cases cover the SPEC-0015 unknowns the case builder (builder v2, `definition`) can express:
probes/t2085-getpivotdata-data-fields, -items and -layouts (excel-graph, 2026-11-07). No builder change was needed.
The findings are in web.md "GETPIVOTDATA on other pivot shapes (T-2085)", a new section beside "GETPIVOTDATA against a pivot table", which points to it. The headline rule is that GETPIVOTDATA
answers what the pivot shows: a saved render wins over the cache, totals follow the rendered summation order, and a
total or item the pivot does not show is `#REF!`.

Not probed:
- an array `pivot_table`, which no plain formula builds;
- cache items differing only in case, and calculated items, which Excel for the web refuses;
- grouped fields, already T-4228's;
- a cache saved without records (`saveData="0"`), which the case builder cannot write.

A range over two pivots is left UNVERIFIED: the committed fill answered from Left, an uncommitted later fill from
Right, so the case does not ask it; a probe that records several fills is needed.

T-2083 is done, so the rules that differ from today's engine went to new owner tasks instead of an edit to it:
T-4804 (data fields and aggregates), T-4805 (items, records, arguments, arrays) and T-4806 (shown values and
shapes). T-4807 owns the date items' shown text, and T-4174 the calculated-field cells. SPEC-0015's unknowns
paragraph points to the evidence.
