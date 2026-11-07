---
id: T-4787
title: "BINOM.DIST and BINOM.DIST.RANGE cdf last digits on t3444's Inv and Range sheets: the 245 cells no task names since T-3740 and T-3741 closed (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4786]
defer: phase-1b
---

## Goal
The BINOM.DIST cdf cells of probes/t3444-binom-range-inv-large-n match Excel to the bit, and so do the BINOM.DIST.RANGE
cells built on them, at n from 1E4 to 2^31 - 2. Failing that, the Outcome ranks the forms tried with their bit-exact
counts (decision 0056 time box).

## Context
- Gap triage 2026-11-07 (`make eval` on origin/main c18fafcce, excel-graph). An ownership check ran over every
  failing Excel cell (`scripts/exit_owners.py`'s `owned` rule against the open task files). It found 245 cells of this
  case that no open task names. T-3740 and T-3741 are done. T-4746 keeps its 21 Range rows (A6, A12, ..., A126) and
  the T-3740 sweep.
- What the cells are:
  - Inv C is `BINOM.DIST(A<k>,n,p,TRUE)` and Inv D is `BINOM.DIST(A<k>-1,n,p,TRUE)`, where A<k> is
    `BINOM.INV(n,p,alpha)`. n runs 1E4, 1E5, ..., 1E9, 2^31 - 2; p is 0.3, 0.5 or 0.001; alpha runs from 1E-6 to 0.5.
  - Range A is BINOM.DIST.RANGE. Range B is the matching `BINOM.DIST(s2,...)-BINOM.DIST(s1-1,...)` difference.
- By class: Inv BINOM.DIST 44 ulp4, 39 ulp1, 30 residue13, 18 digits15 and 15 visibly off; Range BINOM.DIST.RANGE
  23 visibly off, 21 ulp1 and 5 ulp4; Range BINOM.DIST 21 ulp1, 18 residue13, 6 visibly off and 5 ulp4.
- Every visibly-off cell is at p = 0.001 and n from 1E5 to 1E7, near the mean or in the lower tail.
  - Where the math exists, Excel is 2.6E-13 to 1.3E-12 below it and the engine is within 1E-15, apart from deep
    lower-tail D cells near a cdf of 1E-6, where both are 1E-13 to 5E-13 off (D52, D70).
  - Examples: Range!A49 `BINOM.DIST.RANGE(1000000,0.001,1000,1100)` has Excel 1.19E-12 off the math and ours 6.6E-16;
    Inv!D67 `BINOM.DIST(A67-1,10000000,0.001,TRUE)` has Excel 0.4986681958001577 and ours 0.49866819580081084.
  - So these are Phase 1b evidence (decision 0325), not campaign cells. 17 of them still lack a math value, which
    T-4786 attaches; this task depends on it so the scoring has the math.
  - This is T-3961's regime: p below 0.01 at a huge n, where Excel is always lower than the math and ours is close.
    Apply T-3961's model of Excel's small-p cdf formation here first if it finds one.
- The p = 0.3 and 0.5 cells are within 15 digits. Examples: Inv!C1 `BINOM.DIST(A1,10000,0.3,TRUE)` has Excel
  0.5049329838192986 and ours 0.5049329838192993 (residue13); Inv!C10 `BINOM.DIST(A10,10000,0.5,TRUE)` has Excel
  1.0127956818449685E-06 and ours 1.0127956818449515E-06. Near the mean these points are on basym (`bratio/basym.rs`).
  T-3774 recorded Excel's basym arithmetic there as not identified, and T-3597 and T-3959 own that question on other
  cases. Start from their findings.
- Semantics: statistics.md "BINOM.DIST.RANGE and BINOM.INV at a large n sit on BINOM.DIST's own cdf" (T-3444) and
  "BINOM.DIST and NEGBINOM.DIST at a large n" (T-3352, T-3447).
- Code: crates/rustsheet-engine/src/functions/statistical/binom.rs (`binom_dist_range_of`, `excel_binom_upper_tail`),
  `bratio/{mod,basym,bfrac,bup,bgrat}.rs`. `bratio::dispatch` names each point's branch. Method:
  docs/research/excel-kernel-playbook.md.
- Needs an oracle sweep if a fit is attempted. The case has only 126 Inv rows over three p values, too few for a
  hold-out at each (n, p). Use T-3960's small-p sweep for p = 0.001, and file a research sweep task for p = 0.3 and
  0.5 near the mean at n of 1E4 and above if no open one covers it.

Cells this task owns (T-4786 owns Inv!C36, C50, C54, C68, C72, D31, D36, D49, D50, D52, D54, D67, D68, D70, D72 and
Range!B49, B53 until their math is attached; they are listed here too):
- probes/t3444-binom-range-inv-large-n.case.json (146): Inv!C1, C2, C3, C4, C6, C7, C8, C9, C10, C12, C13, C14, C18, C19, C20, C22, C24, C25, C26, C28, C30, C32, C34, C36, C37, C38, C40, C44, C46, C48, C49, C50, C51, C52, C54, C56, C58, C60, C64, C66, C68, C70, C72, C74, C76, C78, C80, C82, C84, C86, C88, C90, C92, C94, C96, C98, C100, C102, C104, C106, C108, C109, C112, C114, C116, C117, C118, C120, C122, C124, C126, D1, D2, D3, D4, D6, D7, D8, D9, D10, D12, D13, D14, D18, D20, D22, D25, D28, D30, D31, D32, D33, D34, D36, D37, D38, D40, D42, D43, D44, D46, D48, D49, D50, D51, D52, D54, D56, D58, D60, D62, D64, D66, D67, D68, D70, D72, D74, D75, D76, D78, D80, D82, D84, D85, D88, D90, D91, D94, D96, D100, D102, D104, D106, D108, D110, D112, D114, D115, D116, D118, D120, D121, D122, D124, D126
- probes/t3444-binom-range-inv-large-n.case.json (99): Range!A1, A2, A4, A5, A7, A8, A10, A11, A13, A16, A17, A19, A22, A25, A27, A28, A29, A31, A32, A33, A35, A37, A40, A43, A45, A49, A50, A51, A52, A53, A57, A61, A67, A68, A69, A71, A75, A85, A89, A91, A93, A95, A98, A104, A105, A111, A112, A115, A124, B1, B2, B4, B5, B7, B8, B10, B11, B13, B16, B17, B19, B22, B25, B27, B28, B29, B31, B32, B33, B35, B37, B40, B43, B45, B49, B50, B51, B52, B53, B57, B61, B67, B68, B69, B71, B75, B85, B89, B91, B93, B95, B98, B104, B105, B108, B111, B112, B115, B124

## Acceptance criteria
- [ ] The owned cells' known-failure lines leave eval/baseline/probes.txt (`make eval-baseline`) with no new line in
      any baseline. Otherwise the Outcome lists each form tried with its bit-exact count over the owned cells and the
      hold-out.
- [ ] No gated BINOM.DIST, NEGBINOM.DIST, BETA.DIST or F.DIST cell regresses: they share bratio.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7) for any fitted form, recorded in the Outcome:
  - hold-out counts: a hold-out split fixed before scoring, with its bit-exact count;
  - a fresh sweep drawn after the form is frozen, with its count;
  - the Gate A envelope result for BINOM.DIST, BINOM.DIST.RANGE and BINOM.INV (0 points outside both);
  - a `FITTED` tag in the code with the range it was fitted on;
  - the reproducibility floor;
  - no keep-matched branch (no special case that only keeps already-matching cells).
- [ ] What was found is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-4746's Range rows and the T-3740 sweep; T-3961's and T-3959's t3352 and t3960 cells; T-3774's pmf cells.
- The math sidecar itself (T-4786).

## Deferred (decision 0114)
`defer: phase-1b`: with math attached, every owned cell is within 15 digits of Excel or is Phase 1b evidence (Excel
and the engine both within 1E-9 of the math). It waits until Phase 1 exits. Remove the key only when the steward
records the Phase 1 exit, or when an eval shows an owned cell counted as visibly off (then give it a `campaign:` stage).
