---
id: T-4540
title: "AVERAGEIF numeric, numeric-text and boolean criteria with an average_range: probe the 5 unseen argument pairs, the resized average_range included, and match"
milestone: M9
lane: functions-criteria
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
AVERAGEIF's argument-class pairs reach full pairwise coverage in report.json `argument_coverage` (0.78 today; 5
pairs unseen) with Excel cells ours matches: a criteria of each numeric-like kind together with an `average_range`.

## Context
- Gap triage 2026-11-06 (opus; `make eval` on 257793681, excel-graph evidence; decision 0031). Every failing cell
  of that run calling a criteria function (SUMIF, COUNTIF, COUNTIFS, SUMIFS, AVERAGEIFS, COUNTBLANK, the D-functions)
  fails because of another function nested around it (FORECAST.ETS family: T-2425) or is owned elsewhere, so the
  lane's gap is evidence. AVERAGEIF is the only criteria function with unseen argument pairs:
  `criteria` in {zero, negative, fraction, numeric_text, boolean} x `average_range` = range. Single kinds are all
  covered (single 1.0), so Excel has answered each criteria kind only without a separate average_range.
- Relevant rules: criteria.md "SUMIF resizes sum_range from its top-left cell and never errors" ("AVERAGEIF resizes
  its average_range the same way", evidence gen-m2 SUMIF cells and probes/ifs-shape-mismatch F9-F11) and the
  numeric-criteria rules there (`0`, `"0"`, `"=0"` each match the number 0 and the text "0": probes/
  criteria-tilde-blank-single-cell-edges). Whether a boolean criteria matches TRUE cells only, or also 1, with the
  average taken from a different (and differently shaped) range, has no AVERAGEIF cell.
- T-3129 (research, todo) lists AVERAGEIF.criteria single kinds; it does not ask for an average_range. If it lands
  first with these pairs, close this task as covered by it.
- Ours: `apply_ifs`, `resize_to` in crates/rustsheet-engine/src/functions/statistical/if_ifs.rs; criteria parsing in
  functions/criteria.rs.

## Probe
One case, probes/t4540-averageif-criteria-average-range (excel-probe skill, excel-graph, tag M9). Range A1:A6
holding 0, -1, 0.5, the text "1", TRUE, 1; average_range D1:D6 = 10, 20, 30, 40, 50, 60 (so the average names the
rows matched). Criteria: 0, -1, 0.5, "1", "0.5", TRUE, FALSE, each as a literal and as a cell. Each once with
D1:D6, once with D1 alone and once with the 1x2 range D1:E1 (both resized from their top-left
cell, if Excel follows the SUMIF rule). Record ours in docs/research/t4540 before the fill.

## Acceptance criteria
- [x] probes/t4540-averageif-criteria-average-range is committed with excel-graph oracle values; after `make eval`,
      report.json `argument_coverage` shows no unseen pairs for AVERAGEIF.
- [x] Every cell of it passes; a rule ours did not follow is fixed in if_ifs.rs or criteria.rs (shared with
      SUMIF/COUNTIF, not copied) with a unit test citing the cells, and SUMIF/COUNTIF cells keep passing.
- [x] Any new rule is recorded in docs/excel-semantics/criteria.md citing the cells.
- [x] No new known-failure lines in any eval/baseline/*.txt; `make check-fast` green, CI green; reviewers approve.

## Out of scope
- AVERAGEIFS/MAXIFS/MINIFS criteria kinds (T-3129) and partly lifted ranges (T-2833).

## Outcome
- Probe: probes/t4540-averageif-criteria-average-range (42 cells, excel-graph, 2026-11-06), tagged M9 (AVERAGEIF's
  catalog milestone) rather than the M2 the task's sketch implied by its suite neighbours. Ours was recorded before the
  fill in docs/research/t4540/predictions.md.
- Result: every cell matches Excel with no engine change. The text criterion `"1"` matches the text "1" and the number
  1, `TRUE` matches the TRUE cell and not the number 1, `FALSE` matches nothing (`#DIV/0!`), and D1 and D1:E1 resize
  to D1:D6 for every criteria kind, literal or cell. Recorded in criteria.md "AVERAGEIF's numeric-like criteria pick
  the same rows under a resized average_range" (VERIFIED).
- Coverage: report.json `argument_coverage` for AVERAGEIF is single 1.0, pairwise 1.0, `unseen_pairs: []` (0.78
  before).
- Eval delta: +42 passing cells in one new probes case, 0 new known failures, 0 regressions. No unit test: no rule
  changed (the acceptance criterion asks for one only for a fix).

