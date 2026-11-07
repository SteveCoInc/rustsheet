---
id: T-4774
title: "t3444 BINOM.DIST cdf at a large n and the BINOM.DIST.RANGE rows that are its cdf difference: 228 cells no task lists since T-3447's hand-off (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The BINOM.DIST cdf cells of the T-3444 probe listed below match Excel's bits. So do the BINOM.DIST.RANGE rows that
Excel computes as the difference of two of those cdfs. The fix models Excel's cdf at n from 1E4 to 2^31 - 2.
Otherwise the Outcome ranks the forms tried, with their counts.

## Context
- Gap triage 2026-11-07 (`make eval` on origin/main b9ae5216, excel-graph). A per-cell ownership check found 245
  known-failure cells of this probe that no open task lists. It used `scripts/exit_owners.py` `owned` over every open
  task, deferred ones included. T-3447 listed them (Inv!C, Inv!D, Range!A and Range!B), and it is done. Its hand-off
  T-3774 lists only the t3352 cells. T-4773 takes the 17 of them that R1(c) counts, because they have no math value.
  This task takes the other 228.
- Why `defer: phase-1b` and no `campaign:` key. 201 of the 228 are within 15 digits of Excel: 81 within 1 ulp, 54 within
  4 ulp, 18 `digits15`, and 48 `residue13`, which decision 0267 makes Phase 1b residue for the incomplete-beta family.
  The other 27 are visibly off but are R1(c) Phase 1b evidence: Range!A1, A2, A4, A5, A7, A10, A11, A13, A17, A19,
  A25, A31, A32, A35, A49, A50, A53, A61, A67, A68, A71, A91, A115 and Range!B31, B35, B67, B71. For each of them the
  case's math sidecar has a value, Excel and the engine are both within 1E-9 of it, and `--all-off` does not list it
  (decision 0325).
- One root cause. Every Range!A row here has its lower end at or below the mean (s1 - 1 <= n p). On all 49 owned rows
  Excel's Range!A equals its Range!B, `BINOM.DIST(s2,..,TRUE)-BINOM.DIST(s1-1,..,TRUE)`, to the bit. statistics.md
  records this under "BINOM.DIST.RANGE and BINOM.INV at a large n sit on BINOM.DIST's own cdf (T-3444)". Ours equals
  our own Range!B on 45 of them. Inv!C and Inv!D are BINOM.DIST at Excel's own BINOM.INV results, which the engine
  matches. So every owned cell moves with BINOM.DIST's cdf alone.
- What is known about that cdf. statistics.md "BINOM.DIST and NEGBINOM.DIST at a large n: basym and Loader, within
  1E-11 of the math except at small p (T-3352, T-3960)" covers it, and three open tasks work the same kernel on other
  probes:
  - T-3959 (`visible`): near the mean and at a moderate p (basym, bfrac).
  - T-3961 (`visible`): p below 0.01 and a huge n, where Excel is hundreds to tens of thousands of ulp low. At this
    probe's p = 0.001 rows Excel is again the side far from the math. Inv!C36 `BINOM.DIST(95,100000,0.001,TRUE)`:
    Excel -2.6E-13, ours -7.6E-15 (50-digit pmf-recurrence sum, this triage). Inv!D49
    `BINOM.DIST(999,1000000,0.001,TRUE)`: Excel -1.2E-12, ours +9.1E-16.
  - T-4365 (deferred): the pmf last digits.
  Score whatever model those tasks land on these cells first. This probe is a ready-made hold-out for them, at n up to
  2^31 - 2 and p of 0.3, 0.5 and 0.001. Do not fork the cdf code (directive 3).
- Cells:
  - probes/t3444-binom-range-inv-large-n.case.json (66): Inv!C1, C2, C3, C4, C6, C7, C8, C9, C10, C12, C13, C14, C18, C19, C20, C22, C24, C25, C26, C28, C30, C32, C34, C37, C38, C40, C44, C46, C48, C49, C51, C52, C56, C58, C60, C64, C66, C70, C74, C76, C78, C80, C82, C84, C86, C88, C90, C92, C94, C96, C98, C100, C102, C104, C106, C108, C109, C112, C114, C116, C117, C118, C120, C122, C124, C126
  - probes/t3444-binom-range-inv-large-n.case.json (65): Inv!D1, D2, D3, D4, D6, D7, D8, D9, D10, D12, D13, D14, D18, D20, D22, D25, D28, D30, D32, D33, D34, D37, D38, D40, D42, D43, D44, D46, D48, D51, D56, D58, D60, D62, D64, D66, D74, D75, D76, D78, D80, D82, D84, D85, D88, D90, D91, D94, D96, D100, D102, D104, D106, D108, D110, D112, D114, D115, D116, D118, D120, D121, D122, D124, D126
  - probes/t3444-binom-range-inv-large-n.case.json (49): Range!A1, A2, A4, A5, A7, A8, A10, A11, A13, A16, A17, A19, A22, A25, A27, A28, A29, A31, A32, A33, A35, A37, A40, A43, A45, A49, A50, A51, A52, A53, A57, A61, A67, A68, A69, A71, A75, A85, A89, A91, A93, A95, A98, A104, A105, A111, A112, A115, A124
  - probes/t3444-binom-range-inv-large-n.case.json (48): Range!B1, B2, B4, B5, B7, B8, B10, B11, B13, B16, B17, B19, B22, B25, B27, B28, B29, B31, B32, B33, B35, B37, B40, B43, B45, B50, B51, B52, B57, B61, B67, B68, B69, B71, B75, B85, B89, B91, B93, B95, B98, B104, B105, B108, B111, B112, B115, B124
- Not owned here. The 17 counted cells without a math value (Inv!C36, C50, C54, C68, C72, D31, D36, D49, D50, D52,
  D54, D67, D68, D70, D72, Range!B49, B53) are T-4773's. The past-mean Range!A rows (A6, A12, ..., A126) are T-4746's.
- Code: crates/rustsheet-engine/src/functions/statistical/binom.rs (`excel_binom_cdf`, `binom_dist_range_of`) and
  `bratio/{mod,basym,bfrac,bup,bgrat}.rs`. `bratio::dispatch` says which branch each point takes. Method:
  docs/research/excel-kernel-playbook.md.

## Acceptance criteria
- [ ] The Outcome gives, for each candidate form, its bit-exact, within-4-ulp and within-15-digit counts on these 228
      cells. The counts are split by branch (`bratio::dispatch`) and by p (0.3, 0.5, 0.001).
- [ ] Each listed cell that passes leaves eval/baseline/probes.txt (`make eval-baseline`), and no new known-failure
      line appears in any eval/baseline file. No gated BINOM.DIST, NEGBINOM.DIST, BINOM.DIST.RANGE, BINOM.INV,
      BETA.DIST or F.DIST cell regresses (check per cell, main against HEAD).
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome, for any form chosen by scoring
      against oracle cells:
  - hold-out counts: a split of about 30% fixed by a hash before scoring, with train and hold-out side by side
    (bit-exact, within 4 ulp, within 15 digits) for the chosen form and for the correctly rounded value;
  - a fresh sweep over the same n and p, generated and oracle-filled (excel-graph) after the form is frozen, with its
    result recorded here;
  - the Gate A envelope result on the fresh sweep and on every owned cell, with every point outside it listed;
  - every fitted constant or cut tagged `FITTED(T-4774, <n> cells, <range>)` and applied only inside that range, with
    accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a desktop-backed
    reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] What was found is in docs/excel-semantics/statistics.md under the large-n BINOM.DIST entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 17 cells without a math value (T-4773), the p < 0.01 fit on the t3352 and t3960 cases (T-3961), the moderate-p
  cdf near the mean on t3352 (T-3959), the pmf (T-4365, T-3774) and the past-mean BINOM.DIST.RANGE form (T-4746).
