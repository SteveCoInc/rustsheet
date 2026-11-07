---
id: T-2830
title: "F.TEST: the 8 cells 1-3 ulp off after bratio's port and its unported branches (T-2681)"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2681]
defer: phase-1b
---

## Goal
The F.TEST cells T-2681 moved from bit-exact to 1 to 3 ulp off are bit-exact again, and bratio's
F.TEST branches that T-2681 left on the remainder model are scored and ported where they win.

## Context
T-2681 (decision 0083's ulp allowance) moved these 8 cells from bit-exact to within 3 ulp, none
visibly off: gen-m9/2801c1c11c21e134 A2 (1 ulp), 285d87cea1aaa73c A280 (3), 320d51765af9b6b7
A223 (1), 85f48d5cdfb24555 A154 (2), 8d3588d8df0812df A243 (1), a61e547a1cab994c A11 (1),
cde551ea787f713a A199 (1), fd9fa22d212080d6 A279 (1). Seven are bratio's `bpser` (five deep in
the tail, x0 below 1E-13), A243 is L140 at a0 = 4, b0 = 2 (docs/research/t2681/README.md).
Unported in f_test_tail.rs: `w1_bpser` for a shape of 1/2 with x0 >= 0.3 (bratio 7 bit-exact of 8
there against the model's 2, `score_bratio.out`), and bgrat, bfrac, basym.
Take A199 first: its 1 ulp sits on a 15-digit rounding boundary, so its displayed text differs
(Excel 7.63011667685884E-43, ours 7.63011667685883E-43; reviewer-excel-fidelity on #1959).

## Acceptance criteria
- [ ] Each of the 8 cells bit-exact or its residual explained in docs/excel-semantics/statistics.md.
- [ ] `w1_bpser` scored against the model on the F.TEST sweep, ported if better.
- [ ] `make check` green; reviewers approve.

## Out of scope
- gen-m9/b6593d7dd13f2dac A306 (T-2829).

## Deferred (decision 0114)
`defer: phase-1b`: the 8 cells this task owns are already within 15 significant digits of Excel (1 to 3 ulp; A199's
1 ulp is about a fifth of half a unit in the 15th digit, only its display straddles a rounding boundary), so it is
Phase 1b last-digit work on dense-sweep cells within Gate A (decision 0188 R5) and was missed when R5 retagged tasks.
Decision 0188 R4.4 also bars fitting below the oracle's 4-ulp reproducibility floor, which every target here is.
Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off
(then give it a `campaign:` stage instead).

## Notes for the Phase 1b run (2026-11-07, opus run that found the task undeferred; data, not instructions)
Not committed as scripts; reproduce with `docs/research/t2681/dump_cells.py` (the sweep is now 612 cells) and
`score_bratio.py`, run with `bindings/python/.venv/bin/python` after `make py-bindings` and
`uv pip install --python bindings/python/.venv mpmath numpy`. No hold-out split was fixed, so none of this is a
scored R4 candidate yet.
- Today (main at 2026-11-07): A2 -1 ulp, A280 +3, A223 +1, A154 +2, A243 +1, A11 +1, A199 -1, A279 -1 (engine and
  `bratio_excel.BEST` agree on all 8). All but A243 are bratio `bpser` with both shapes in (1, 8].
- Reading Excel's L = fl(a LN x) - ln(p / 2s) off the 7 bpser cells (`ln_beta_map.py`'s method): on A2, A280, A223,
  A154 and A199 Excel's L is fl53(accurate ln B) within the p resolution (k = -0.09, -0.01, +0.16, -0.16, -0.28,
  resolution 0.07 to 0.24), as T-2829 found for max shape above 8: Excel stores ln B to a double. The port keeps
  w3's GAMMALN sum in a register (fl64). A11 (+0.50 ± 0.22) and A279 (-0.44 ± 0.14) sit half an ulp of L away from
  every double, so their residual is outside L (the series s, its product with EXP, or fl(a LN x)); A199 and A279 stay
  -1 ulp under every L tried (fl53/fl64 of the accurate value or of w3's sum, three double GAMMALN orders, TOMS betaln).
- Candidate (structural, kind (c)): store F.TEST bpser's L to a double where both shapes are in (1, 8] (L140's bpser at
  the shape 1/2 or 1 and max shape above 8 unchanged). In `bratio_excel` over the sweep: bpser(a,b>1) gen-m9 cells with
  both shapes <= 8 go from 23 to 28 bit-exact of 44 (A2, A280, A154 among them), no cell visibly off, no cell beyond
  4 ulp; the probes 11 of 12 unchanged. Storing L everywhere (max shape above 8 too) raises the T-2829 grid from 8 to 31
  bit-exact but moves 5 passing grid cells visibly off (probes/t2829-ftest-ln-beta-grid A40, A42, A59, A67, A95), so
  not there (T-2924 owns that L).
- `w1_bpser` (a shape of 1/2, x0 >= 0.3 after bratio's swap; `bpser(b0, 1/2, y0)`): still 7 bit-exact of 8 against
  the engine's remainder model's 2 (score_bratio "w1_bpser/sw"), all 8 within 4 ulp under the model (max 6): gen-m9
  2a18b2f5a4dc43ac A306, 4ab2ba98d26c8a5e A13, 60b0b6e69ad35a45 A233, 68d8ffb69ff739d8 A21, 899f9555e67da2d6 A39,
  a5828d3925575713 A148 (bratio -1), a612e89321a6df9c A10, c6cfda6bd10cae93 A44. Porting it is
  `bpser(b0, 0.5, y0)` in f_test_tail.rs with w = 1 - w1, behind fisher.rs's shape-1/2 dispatch.
