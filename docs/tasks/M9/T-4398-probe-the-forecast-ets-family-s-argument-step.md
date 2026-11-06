---
id: T-4398
title: "Probe the FORECAST.ETS family's argument step on excel-graph: references to error cells in values and timeline, and the argument kinds no Excel cell covers in its four members"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
excel-graph cells settle the parts of the FORECAST.ETS family's argument step that the web can observe and no cell
covers yet: whether a one-cell (or multi-cell) reference to an error cell in `values` or `timeline` is reported in
argument order, and what an empty text, an error and the other unseen kinds do in each parameter of FORECAST.ETS,
FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY and FORECAST.ETS.STAT. Every cell where ours differs is fixed in
`statistical/forecast.rs` or owned by a new functions-stat task.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 8465f6f1, excel-graph sidecars). report.json `argument_coverage`
  shows the four members' gaps with no open probe task: T-4008, T-4009, T-3129, T-3369 and T-4223 all leave the
  family out, citing T-2425 (or T-1553), which decision 0266 dropped. Missing kinds, from
  `crates/rustsheet-eval/src/argument_coverage.rs`:
  - FORECAST.ETS: target_date empty text, error; values empty text, boolean, error; timeline empty text, error;
    seasonality numeric text, empty text, error; data_completion numeric text, text, empty text, boolean, error;
    aggregation zero, numeric text, empty text, boolean, error.
  - FORECAST.ETS.CONFINT: target_date empty text, error; values and timeline numeric text, empty text, error;
    confidence_level empty text, error; seasonality numeric text, empty text, boolean, error; data_completion numeric
    text, text, empty text, error, blank, range; aggregation zero, negative, numeric text, empty text, boolean, error,
    range.
  - FORECAST.ETS.SEASONALITY: values zero, empty text, error; timeline empty text, error; data_completion numeric text,
    empty text, error, range; aggregation text, empty text, error, range.
  - FORECAST.ETS.STAT: values empty text, error; timeline numeric text, empty text, error; statistic_type empty text,
    error, blank; seasonality empty text, error; data_completion numeric text, empty text, error, range; aggregation
    numeric text, text, empty text, boolean, error, blank, range.
- Why the web can answer this although it never computes the family (statistics.md "FORECAST.ETS family: excel-graph
  never computes it", re-checked by T-3641): it reads the arguments first, so an error argument, a text scalar
  (`#VALUE!`) and a lifted argument decide before the `#N/A` (statistics.md "FORECAST.ETS family on excel-graph:
  argument errors, text and lifting decide before the #N/A (T-2255)", 967 + 36 + 54 gen-m9 cells). A cell whose web
  value is the plain `#N/A` settles nothing and is not this task's (decision 0266 rule 2).
- The open question in that section: "A reference to an error cell, even a one-cell one, is not a direct error here:
  the argument step passes it, and no oracle cell covers it yet (the gen-m9 Data sheets hold "#N/A" only as text;
  T-2425 probe)." Ours (scratch case, release build of 8465f6f1): `FORECAST.ETS.SEASONALITY(C1,A1,1,NA())` with C1
  `=1/0` is `#N/A`, from the later argument, so a probe where the later error differs from the referenced one
  separates the two orders.
- Ours for empty text in a scalar setting: `FORECAST.ETS.CONFINT(13,B1:B12,A1:A12,"")`,
  `FORECAST.ETS.STAT(B1:B12,A1:A12,"")` and `FORECAST.ETS.SEASONALITY(B1:B12,A1:A12,"")` are `#VALUE!` (UNVERIFIED).

Probe (excel-probe skill, excel-graph, tag M9), probes/t4398-forecast-ets-argument-step, on a 12-point series (A1:A12
= 1..12, B1:B12 a period-4 signal) with error cells C1 `=SQRT(-1)` (`#NUM!`), C2 `=1/0`, C3 `=NA()`, and C4 `=""`:
- error references against a later error: `FORECAST.ETS(13,C1,A1,1/0)`, `FORECAST.ETS(13,B1:B12,C1,1/0)`, the same
  with a 12-cell values range holding C1's error in one cell (a copy of B1:B12 with B6 `=SQRT(-1)`), and the CONFINT,
  SEASONALITY and STAT forms (STAT's later error in statistic_type);
- each scalar parameter of the four members as `""`, as C4, as a literal error (`#DIV/0!`) and as C2, with
  every other argument valid; and the kinds listed above for the setting parameters (`"1"` numeric text, `"x"` text,
  TRUE, a blank cell, a two-cell range);
- wrap every cell in `ERROR.TYPE(..)` as well, so an error kind is scored by number.

## Acceptance criteria
- [ ] probes/t4398-forecast-ets-argument-step is committed with excel-graph oracle values and covers each
      parameter/kind pair above; after `make eval`, report.json `argument_coverage` shows them seen.
- [ ] Each cell where ours differs from a web value other than the plain `#N/A` is fixed in this PR (a small change in
      `forecast_ets_lifted`, statistical/forecast.rs, with a unit test) or listed in a new functions-stat task
      (`campaign: semantics`, with a `## Visible-parity stage` section) filed with `make new-task`, naming the case
      and cells. Plain-`#N/A` cells against a computed engine value join the decision-0266 cells of T-4207.
- [ ] statistics.md's "no oracle cell covers it yet" sentence is replaced by the probed rule, with the cells cited.
- [ ] `make eval-baseline` records no regression; `make check` green; reviewers approve.

## Out of scope
- The forecasting model and its numbers (decision 0221; web `#N/A`, decision 0266).
