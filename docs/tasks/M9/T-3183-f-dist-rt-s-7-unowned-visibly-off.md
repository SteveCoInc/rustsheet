---
id: T-3183
title: "F.DIST.RT's 7 unowned visibly-off cells: 2*F.DIST.RT(1E+12,df1,12) in probes/t2829-ftest-tail-over-b and F.DIST.RT(3,50,100)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The eight F.DIST.RT cells below (seven in the table and gen-m9 60b0b6e69ad35a45 A151) are within half a unit of the 15th significant digit of Excel's value. If no form
gets them there inside the time box (decision 0056), the forms tried and their scores go in
docs/excel-semantics/statistics.md.

## Context
Gap triage 2026-10-30 (`make eval` on main c269641be). `make unowned-off ALL=1` lists these 7 as the only visibly-off
attributed cells with no open owner. T-3007 (done) owned the probe's cells. Its Outcome says they "stay visibly off
(T-2924's ln B k plus fl(a LN x) steps)", but T-2924 (blocked, research) names only the ln-beta grid. The A19 cell
is named only by T-2221, which carries `defer: phase-1b` and so is never picked in Phase 1.

| cell | formula | Excel | ours | rel. diff |
|---|---|---|---|---|
| probes/t2829-ftest-tail-over-b S!A1 | `2*F.DIST.RT(1E+12,17,12)` | 5.654118407649194E-70 | 5.654118407649187E-70 | 1.3E-15 |
| probes/t2829-ftest-tail-over-b S!A3 | `2*F.DIST.RT(1E+12,19,12)` | 4.948640421847892E-70 | 4.948640421847906E-70 | 2.9E-15 |
| probes/t2829-ftest-tail-over-b S!A4 | `2*F.DIST.RT(1E+12,20,12)` | 4.6702655999615336E-70 | 4.67026559996154E-70 | 1.4E-15 |
| probes/t2829-ftest-tail-over-b S!A6 | `2*F.DIST.RT(1E+12,22,12)` | 4.217989084169663E-70 | 4.217989084169657E-70 | 1.4E-15 |
| probes/t2829-ftest-tail-over-b S!A7 | `2*F.DIST.RT(1E+12,23,12)` | 4.032210775559413E-70 | 4.032210775559391E-70 | 5.4E-15 |
| probes/t2829-ftest-tail-over-b S!A9 | `2*F.DIST.RT(1E+12,26,12)` | 3.588797418723195E-70 | 3.588797418723189E-70 | 1.6E-15 |
| probes-numeric/m9-f-dist-rt Calc!A19 | `F.DIST.RT(3,50,100)` | 1.516786664496894E-6 | 1.5167866644968792E-6 | 9.8E-16 |

- The A1-A9 cells are bpser with a = df2/2 = 6 and b = df1/2 from 8.5 to 13, so both shapes are above 1 and one is
  above 8, at y = 12/(12 + df1·1E+12). The case's other cells (df2 = 5, and A2, A5, A8, A10, A11) are within 15 digits.
  statistics.md ("F.TEST / FTEST", T-2829) gives two sources of error. One is Excel's ln B, a stored double k = -3 to +2
  ulps from the accurate ln B (T-2924, probes/t2829-ftest-ln-beta-grid). The other is whole steps of the
  fl(a·LN y) grid. T-2847 found that at A7 the w5 form y = 1/(1 + d1·X/d2) moves fl(a·LN y) by -2.84E-14 relative against
  d2/(d1·X + d2), which is most of A7's gap. Its Outcome lists the y forms per cell.
- Added by gap triage 2026-11-03 (`make eval` on origin/main c2062e5c8, excel-graph; `scripts/exit_owners.py --all-off`
  lists it as visibly off and owned only by deferred T-2221's "Cells added from T-3198"): gen-m9/60b0b6e69ad35a45
  Calc!A151 `F.DIST.RT(1E+10,9,9)`, Excel 6.622462439866629E-44, ours 6.622462439866635E-44. Against 50-digit mpmath
  (6.6224624398666040618E-44) Excel is +3.8E-15 and ours +4.7E-15, the same side, so Excel is the inaccurate one, as at
  A1-A9. It is the same branch: bpser (`BpserLarge`) at a = b = 4.5, y = 9/(9 + 9E+10) = 1E-10, both shapes above 1.
  It is F.DIST.RT's only visibly-off Gate B cell (gen-m9 is a Gate B suite, decision 0188 R1(b)), so it alone keeps
  F.DIST.RT below Gate B's rate. This task takes it out of T-2221's list (T-2221 names this task).
  - gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A151
- A19 is on the ported L140 bup+bpser branch (T-2981 routing note), at a = 50, b = 25, x = 3.
- Code: crates/rustsheet-engine/src/functions/statistical/ (the F tail in `fisher.rs` / `f_test_tail.rs` and the
  `bratio` port, SPEC-0025).
- Approach: first score each cell against a 50-digit reference (mpmath in the tools venv), to learn which side is
  inaccurate. Where Excel is the inaccurate one, try the y form and the ln B pieces per cell (T-2847's w5 form, T-3007's
  TOMS betaln, T-2924's candidate list) and keep the first that reaches 15 digits on all seven without moving another
  F.DIST, F.DIST.RT, FDIST, F.TEST or BETA.DIST cell off. If the reference shows Excel is close to correctly rounded at
  A19, A19 is an accuracy fix on the L140 branch instead.

## Visible-parity stage
`campaign: visible` (decisions 0083, 0114). Excel's ln B is the inaccurate side for A1-A9 (T-2924), so this means
reproducing Excel's form, time-boxed. The task is done when every cell in the table, and gen-m9 60b0b6e69ad35a45 A151, is within half a unit of the 15th
significant digit of Excel's value and no cell is newly visibly off. Bit-level matches are not required. Cells then
within 15 digits but not bit-exact go to T-2221 (`defer: phase-1b`).

## Acceptance criteria
- [ ] These 8 known-failure lines leave eval/baseline (probes.txt, probes-numeric.txt and gen-m9.txt) at visible parity, with no
      new visibly-off cell in any suite. Or, if the time box ends first, statistics.md records each form tried with its
      per-cell score, and a follow-up task names the cells left.
- [ ] `make unowned-off ALL=1` lists none of the 8 cells (each is fixed or named by its follow-up).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The ln-beta grid fit itself (T-2924) and the bit-level F residue (T-2221).
