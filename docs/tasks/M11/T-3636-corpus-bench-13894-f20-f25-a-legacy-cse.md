---
id: T-3636
title: "corpus-bench 13894 F20:F25: a legacy CSE array formula's cells past its 6-row result are #N/A (M11 owner of T-0950's corpus cells)"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-0950]
---

## Goal
The six extra cells of corpus-bench/13894/1_13894_input.xlsx's legacy array formula, Sheet1!F20:F25, are `#N/A`
as Excel shows them, so M11 has no unowned exit cell.

## Context
- The CSE array formula over F14:F25 returns a 6-row result (`ucode` = J3:J8). Excel fills the rest of the range
  with `#N/A`; ours gives `#VALUE!` in corpus-bench/13894/1_13894_input.xlsx Sheet1!F20, F21, F22, F23, F24 and F25.
  The formula is `IFERROR(INDEX(ucode,SMALL(IF(MATCH(ucode,ucode,0)=ROW(INDIRECT("1:"&ROWS(ucode))),...)),"")`.
- T-0950 (M9) implements the rule: docs/excel-semantics/arrays.md, "Legacy (CSE) array formula over a larger range".
  It already lists these cells, but as an M9 task it cannot own an M11 exit cell (decision 0096). This task is
  their M11 owner. Once T-0950 lands, confirm they pass and close it; if they still fail, fix what remains here.
- Found by `make exit-owners MILESTONE=M11` on 2026-11-01. M11's exit row is met (corpus-bench is within its
  allowance), so only the owners criterion of the Phase 1 exit waits on these cells.

## Acceptance criteria
- [x] corpus-bench/13894/1_13894_input.xlsx Sheet1!F20:F25 pass, and the case leaves eval/baseline/corpus-bench.txt
      if no other cell fails.
- [x] `make check` green; reviewers approve.

## Out of scope
- The rest of T-0950 (poi/MatrixFormulaEvalTestData.xlsx).

## Outcome
- No code change: T-0950 (#3889) already makes Sheet1!F20:F25 `#N/A`. On origin/main c709e9d20 (2026-11-07, corpus-bench
  fetched), `rustsheet-eval show corpus-bench/13894/1_13894_input.xlsx` reports `status=Pass checked=28 passed=28`, and
  eval/baseline/corpus-bench.txt has no `F` line for the case (only its `C ... 28` count line, which every case has).
  The cells no longer fail, so M11 needs no owner for them.
- Eval delta: 0 (the fix was recorded by T-0950's PR).
- Also filed T-5012 (M15, harness, sonnet): remove the legacy per-case round-trip baseline path after T-4961 (#4007)
  lands, the follow-up of #4007's design and security MINOR findings.
