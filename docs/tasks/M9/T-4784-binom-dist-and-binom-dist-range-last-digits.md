---
id: T-4784
title: "BINOM.DIST and BINOM.DIST.RANGE last digits on probes/t3444-binom-range-inv-large-n: the 245 cells T-3447 listed and no open task took over (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3961, T-4782, T-4783]
defer: phase-1b
---

## Goal
The BINOM.DIST cdf cells of probes/t3444-binom-range-inv-large-n (Inv!C, Inv!D, Range!B) and its BINOM.DIST.RANGE cells
outside T-4746's rows match Excel bit for bit. Otherwise the Outcome ranks the models tried against them, after T-3961
has a model for Excel's small-p cdf.

## Context
- Gap triage 2026-11-07 (`make eval` on origin/main b9ae5216, every suite fetched, excel-graph). The failing cells of
  this case below have no open owner. T-3447 (done 2026-11-01) listed them, and its Outcome moved "the cells and the
  second attempt" to T-3774. T-3774 (blocked, time-boxed) lists only probes/t3352-binom-negbinom-large-n, so these were
  left without an owner.
- Why deferred (decision 0114): every visibly-off cell here is, or becomes once its math is attached, Phase 1b
  evidence. Excel and the engine are both within 1e-9 of the 50-digit math. Of the 245 cells, 44 differ from Excel at
  15 digits (Inv!C 5, Inv!D 10, Range!A 23, Range!B 6). R1(c) counts 17 of them today only because they have no math
  value: T-4782 (Range!B49, B53) and T-4783 (the 15 Inv cells) name those and attach it. The rest are already Phase 1b
  evidence.
- The error is Excel's own, the T-3961 regime. At p = 0.001 and n 1E5 to 1E7 Excel's cdf is 2.5E-13 to 1.3E-12 relative
  below the math, and the engine is within about 1E-14. For example Inv!C36 `BINOM.DIST(95,100000,0.001,TRUE)` is Excel
  0.3311016442530485, engine 0.33110164425313326, math 0.33110164425313579. Inv!D67
  `BINOM.DIST(9999,10000000,0.001,TRUE)` is Excel 0.4986681958001577, engine 0.49866819580081084, math
  0.49866819580081081. At the median rows (C31, C49, C67, alpha 0.5) Excel is within 2E-15. Matching these cells means
  reproducing Excel's inaccuracy. Use T-3961's model when it lands, and re-score here, without fitting this case again.
  The Inv cells at p = 0.3 and 0.5 (n 1E4 to 2^31 - 2) are within 15 digits, 1 to a few ulp: basym and Loader residue,
  as in T-3774. The 23 visibly-off Range!A cells span all three p, for example A1 `BINOM.DIST.RANGE(10000,0.3,3000,3010)`
  (Excel 0.09492270540059111, engine 0.09492270540058889), A13 `(10000,0.001,10,20)` and A91
  `(1000000000,0.3,300000000,300000100)`. They are a cdf difference, so it amplifies the last-digit residue of both
  cdfs. T-4746's candidates (statistics.md "BINOM.DIST.RANGE and BINOM.INV at a large n sit on BINOM.DIST's own cdf")
  apply to them too.
- Cells (probes/t3444-binom-range-inv-large-n.case.json):
  - Inv!C (71): C1, C2, C3, C4, C6, C7, C8, C9, C10, C12, C13, C14, C18, C19, C20, C22, C24, C25, C26, C28, C30, C32,
    C34, C36, C37, C38, C40, C44, C46, C48, C49, C50, C51, C52, C54, C56, C58, C60, C64, C66, C68, C70, C72, C74, C76,
    C78, C80, C82, C84, C86, C88, C90, C92, C94, C96, C98, C100, C102, C104, C106, C108, C109, C112, C114, C116, C117,
    C118, C120, C122, C124, C126
  - Inv!D (75): D1, D2, D3, D4, D6, D7, D8, D9, D10, D12, D13, D14, D18, D20, D22, D25, D28, D30, D31, D32, D33, D34,
    D36, D37, D38, D40, D42, D43, D44, D46, D48, D49, D50, D51, D52, D54, D56, D58, D60, D62, D64, D66, D67, D68, D70,
    D72, D74, D75, D76, D78, D80, D82, D84, D85, D88, D90, D91, D94, D96, D100, D102, D104, D106, D108, D110, D112,
    D114, D115, D116, D118, D120, D121, D122, D124, D126
  - Range!A (49), BINOM.DIST.RANGE outside T-4746's 21 rows: A1, A2, A4, A5, A7, A8, A10, A11, A13, A16, A17, A19, A22,
    A25, A27, A28, A29, A31, A32, A33, A35, A37, A40, A43, A45, A49, A50, A51, A52, A53, A57, A61, A67, A68, A69, A71,
    A75, A85, A89, A91, A93, A95, A98, A104, A105, A111, A112, A115, A124
  - Range!B (50): B1, B2, B4, B5, B7, B8, B10, B11, B13, B16, B17, B19, B22, B25, B27, B28, B29, B31, B32, B33, B35, B37,
    B40, B43, B45, B49, B50, B51, B52, B53, B57, B61, B67, B68, B69, B71, B75, B85, B89, B91, B93, B95, B98, B104, B105,
    B108, B111, B112, B115, B124
- Shared cells: T-4782 names Range!B49 and B53, and T-4783 names the 15 visibly-off Inv cells, for their math only.
  This task owns their bit-level parity.
- Code: `crates/rustsheet-engine/src/functions/statistical/binom.rs` (`binom_dist_of`, `binom_dist_range_of`),
  `bratio/{basym,bfrac}.rs`. Semantics: statistics.md "BINOM.DIST and NEGBINOM.DIST at a large n" and "BINOM.DIST.RANGE
  and BINOM.INV at a large n sit on BINOM.DIST's own cdf".

## Acceptance criteria
- [ ] The Outcome gives the bit-exact / within-4-ulp / within-15-digit counts of the cells above on main and after the
      change. Every cell that passes leaves eval/baseline/probes.txt (`make eval-baseline`). No new known-failure line
      appears in any eval/baseline file, and no gated BINOM.DIST, NEGBINOM.DIST, BINOM.DIST.RANGE, BINOM.INV or
      BETA.DIST cell regresses.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome, for any form or cut chosen by scoring
      these cells:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    for the correctly rounded value, with the split and margin stated before scoring;
  - a fresh sweep over the regime, generated and oracle-filled (excel-graph) after the model is frozen, with its
    result;
  - the Gate A envelope result on the fresh sweep and on every owned cell, with every point outside it listed;
  - every fitted constant or cut carries `FITTED(T-4784, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] What was found is in docs/excel-semantics/statistics.md, "BINOM.DIST and NEGBINOM.DIST at a large n".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's small-p cdf model itself (T-3961), the t3352 cells (T-3774, T-3959), and BINOM.DIST.RANGE past the mean
  (T-4746).
- Attaching math values (T-4782, T-4783).
