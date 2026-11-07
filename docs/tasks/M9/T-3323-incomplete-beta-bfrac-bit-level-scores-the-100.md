---
id: T-3323
title: "Incomplete beta bfrac bit-level scores: the (100, 120) run's 185 of 1,024 and r2-bratio's 146/685 of 1,316 once the large-shape density lands (T-2508 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2508, T-3244]
defer: phase-1b
---

## Goal
The bfrac port reaches the bit-level scores T-2508 carried: at least 185 of 1,024 bit-exact on the (100, 120) run
(probes-numeric t2464-w2f-xruns-1, the BETA.DIST(.,100,120,TRUE) cells) and at least 146 bit-exact and 685 within
15 digits of r2-bratio's 1,316 bfrac cells.

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell this task owns is a BETA.DIST or
  F.DIST density cell or a dense probes-numeric sweep cell, within Gate A's envelope (most agree with Excel to 12 or
  more digits, and Excel and the engine are both close to the math), so R1(c) counts it as Phase 1b evidence. A
  point that Gate A's tables show outside the envelope gets its own task without the key.
- T-2508 (Outcome) ported bfrac with the engine's T-2507 density: 47 of 1,024 on the (100, 120) run (23 before). The
  engine's density is only 33 of 1,024 bit-exact on that run's FALSE cells, and w4's 185 used Excel's own pdf read off
  the sheet, so the score waits on the large-shape density (T-3244, a + b - 2 >= 90). An x87-extended fraction scored
  42, no better than the double one (statistics.md "Incomplete beta L140 and bfrac").
- r2-bratio's scores were not measured on the engine in T-2508: score docs/research/t2464/r2-bratio's bfrac cells
  (`score.py`, cells.tsv) against the engine (`rustsheet-eval run` on a case, or a test over the cells).
- `bratio/tests.rs` `fixture_ported_branches_score` ratchets bfrac at 39 of 82 fixture rows: raise it when this lands.
- Also owns (T-4026, 2026-11-05): the 18 bfrac cells of T-3580's fresh sweep, visibly off only at the 14th or 15th
  digit (0.5 to 13 units of the 15th; Tier C/D, decision 0267). Branch from `bratio::dispatch` on `fisher.rs`
  `f_beta_arguments` at the truncated dfs, all swapped for the right tail:
  - probes-numeric/t3580-f-fresh.case.json (18): Calc!A905, A933, A1009, A1021, A1029, A1053, A1062, A1086, A1098, A1129, A1161, A1176, B918, B949, B1012, B1036, B1087, B1102

## Acceptance criteria
- [ ] The (100, 120) run scores at least 185 of 1,024 bit-exact (`rustsheet-eval run --suite probes-numeric
      --filter t2464-w2f-xruns-1`), or the Outcome states the measured score and which density cells cap it.
- [ ] r2-bratio's bfrac cells are scored on the engine and the numbers recorded in statistics.md.
- [ ] No gated known-failure line is added; `make check` green; reviewers approve.

## Out of scope
- The fraction's own arithmetic (T-2512).
- basym (T-3322).
