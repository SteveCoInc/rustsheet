---
id: T-4720
title: "FIXED and DOLLAR of a negative half-unit tie (FIXED(-0.005,2)) and TEXT of 1E+100 under 0.0E+0: probe the two UNVERIFIED number-to-text notes and match"
milestone: M5
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells settle the two number-to-text notes the semantics docs leave UNVERIFIED: whether FIXED (and DOLLAR)
keep the minus sign on a negative value that rounds away from zero to the last kept decimal, and how TEXT prints a
3-digit exponent under a single exponent placeholder. The engine matches every probe cell, and the two entries state
the observed results.

## Context
- Gap triage 2026-11-06 (opus worker run, `make eval` on origin/main 4def66f7 with calipers, corpus-bench and poi
  fetched). `scripts/exit_owners.py --all-off` reports 0 visibly-off cells with no open owner, so UNVERIFIED notes
  with no open task are the gap left. FIXED is M5 in eval/catalog/functions.yaml (prior 0.2); DOLLAR and TEXT share
  the same number-to-text code.
- docs/excel-semantics/text.md "FIXED drops the sign of a value that rounds to zero" (T-1257): VERIFIED for
  `FIXED(-2,-1,TRUE)` = "0", `FIXED(-0.004,2)` and `FIXED(-0.4,0)` (gen-m6/f94b5b8236742618 A24,
  probes/dollar-fixed-unichar-edges A49, A50), but "`FIXED(-0.005,2)` keeping its sign ("-0.01") is UNVERIFIED".
  The double -0.005 lies just above -0.005 in magnitude (-0.005000000000000000104), so both a half-away and a
  correctly rounded reading give "-0.01"; the probe should also take ties whose double lies below the tie in
  magnitude (-0.015, -0.045 and -1.005 are 0.01499999999999999944, 0.04499999999999999833 and
  1.00499999999999989 in magnitude) and one more above it (-0.025, 0.025000000000000001387), and the DOLLAR forms
  of each. Ours: `fn_fixed` and `fn_dollar` in
  crates/rustsheet-engine/src/functions/text/string_format.rs drop the sign only when the rounded text has no
  non-zero digit.
- docs/excel-semantics/numbers.md "TEXT of DBL_MAX with a scientific format prints the full 3-digit exponent"
  (T-2106): VERIFIED for `TEXT(2^1023*(2-2^-52),"0.00E+00")` = "1.80E+308" (probes/dbl-max-text-b A1), but "the
  3-digit single-placeholder case (`1E+100` under `0.0E+0` as `1.0E+100`) is UNVERIFIED (probe pending), assumed from
  the 2-digit cells". Ours: formatter/format.rs with decimal_round.rs `round_scientific`.

## Acceptance criteria
- [ ] A probe case `probes/t4720-fixed-sign-and-exponent-digits.case.json` is committed with excel-graph oracle values
      and holds at least: `FIXED(x,2)` and `DOLLAR(x,2)` for x in -0.005, -0.015, -0.025, -0.045, -0.0050000001,
      -0.0049999999 and -1.005, and `FIXED(-0.5,0)`, `FIXED(-0.05,1)`, `FIXED(-5,-1)`; and `TEXT(x,f)` for x in 1E+100,
      9.96E+99, 1E-100, -1E+100 and 1.5E+308 with f in `0.0E+0`, `0E+0`, `0.0E-0` and `##0.0E+0`.
- [ ] Every probe cell passes, or each cell that does not is listed in a new functions-text task (`campaign:
      semantics` with a `## Visible-parity stage` section) filed with `make new-task` that names the case and its
      cells, and those known failures are recorded with `make eval-baseline`.
- [ ] text.md and numbers.md replace the two UNVERIFIED sentences with the observed results and cite the probe cells.
- [ ] Any engine change has a unit test that pins a probed result.
- [ ] `make check-fast` green; no new known-failure line in eval/baseline/ other than those owned as above; reviewers
      approve.

## Out of scope
- FIXED's no_commas argument and DOLLAR's currency symbol under other locales.
- Engineering formats (`##0.0E+0`) beyond the exponent-digit count; the engineering-format tie is T-4542's.

## Outcome
Probe `probes/t4720-fixed-sign-and-exponent-digits` (37 cells, excel-graph, 2026-11-06) is committed with its oracle
sidecar; all 37 cells pass with no engine change. FIXED/DOLLAR keep the minus sign of a negative value that rounds away
from zero, ties included; TEXT prints a full 3-digit exponent under a single placeholder. text.md and numbers.md now
state the observed results. The case writes `1.5*10^308` for the 1.5E+308 row: the Graph workbook service rejects the
literal `1.5E+308` in a formula as an unsupported workbook (value is identical). No Gate A definition covers FIXED,
DOLLAR or TEXT, so there is no math sidecar. Eval delta: +37 passing probe cells, no new known-failure lines (the probes baseline gains only the generated coverage line).
