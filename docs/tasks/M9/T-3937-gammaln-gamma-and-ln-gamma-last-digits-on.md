---
id: T-3937
title: "GAMMALN, GAMMA and LN(GAMMA) last digits on T-3570's lg-aux and lngamma-gl probes once T-2122's kernels ship (719 cells, Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2122, T-3042]
defer: phase-1b
---

## Goal
GAMMALN, GAMMA and LN(GAMMA(x)) return Excel's bits on the 719 last-digit cells of T-3570's two lnGamma probes listed
below, by shipping the K1, K3 and (1.5, 2] kernels T-2122 cracks; or the Outcome names each cell left with its kernel.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): measured on its own probe t3570-r5-bpser-lg-aux: 0 of 2520 cells changed over three fills; see docs/research/t3992/README.md.
- Gap triage 2026-11-03m (`make eval` on origin/main ea84dadb9, excel-graph evidence; per-cell `owned` run over every
  failing cell against every open task, deferred ones included, and every unmerged branch's task files). T-3570
  (done) filled both probes as research side columns; neither case is named by an open task except T-2122's 45
  visibly-off LN(GAMMA) cells of t3570-r5-bpser-lg-aux column E (decision 0245), which stay T-2122's and are not
  listed here. T-3042 ships T-2122's kernels for the two T-2464 GAMMALN probes; this task is the same shipping
  scored on the T-3570 probes, so it depends on both.
- Both cases are in probes-numeric (ungated) and have no math sidecar. Every listed cell is NumericUlp or
  NumericClose and within 15 digits (none is visibly off).
- By column and kernel region of the GAMMA/GAMMALN argument:
  - t3570-r5-bpser-lg-aux (594): A `=GAMMALN(v)`: 102 in [0.7, 1.5) (K3), 183 in [1.5, 2]; B `=GAMMA(v)`: 57 below 0.7,
    27 in [0.7, 1.5); C `=LN(GAMMA(v))`: 51 below 0.7, 27 in [0.7, 1.5); D `=GAMMA(v)`: 24 in [0.7, 1.5), 72 in
    [1.5, 2] (K1's [1.8, 2) dip); E `=LN(GAMMA(v))`: 24 and 27.
  - t3570-r5-bpser-lngamma-gl (125): A `=GAMMALN(a0)` at a0 stepping by single ulps near 0.3, 0.5 and 0.7: 85 below
    0.7 (inheriting K3 through fl(GAMMALN(x + 1) - LN(x))), 38 in [0.7, 1.5), 2 above 2.
- Examples (Excel vs ours), `make eval-show CASE=probes-numeric/<case>`:
  - lg-aux Calc!A100 `=GAMMALN((107374182*2^-26+26843541*2^-52))`: Excel -0.11259176569675594, ours ...595.
  - lg-aux Calc!B110 `=GAMMA((20132659*2^-25+13421773*2^-51))`: Excel 1.489192248812817, ours 1.4891922488128169.
  - lg-aux Calc!D109 `=GAMMA(1.6)`: Excel 0.8935153492876903, ours ...902.
  - lngamma-gl Calc!A73 `=GAMMALN((80530636*2^-28+53687083*2^-54))`: Excel 1.095797994818077, ours
    1.0957979948180772.
- Model: docs/excel-semantics/statistics.md "GAMMALN is Excel's own x87 routine (T-1941, T-2128)"; code
  `crates/rustsheet-engine/src/functions/statistical/gamma_kernels.rs` and `gammaln.rs`. T-2122's restart file is
  docs/research/t1941/HANDOFF2.md on agent/research/T-1941-crack-gammaln-gamma-ln.
- Fitting: every miss is 1-3 ulp, below decision 0112's 4-ulp reproducibility floor, so a coefficient set chosen on
  these cells needs a desktop-backed reproducibility check or an owner ruling (R4.4). These probes are a natural
  hold-out for T-2122's kernels: do not tune on them.

Cells (gap triage 2026-11-03m; 719 cells):
- probes-numeric/t3570-r5-bpser-lg-aux.case.json (594): Calc!A25:A31, A40:A44, A48:A53, A62:A74, A80:A82, A87, A91:A94, A96:A107, A109:A116, A120:A128, A130:A132, A134:A140, A142:A144, A148:A152, A156, A160:A163, A165:A168, A193:A199, A208:A212, A216:A221, A230:A242, A248:A250, A255, A259:A262, A264:A275, A277:A284, A288:A296, A298:A300, A302:A308, A310:A312, A316:A320, A324, A328:A331, A333:A336, A361:A367, A376:A380, A384:A389, A398:A410, A416:A418, A423, A427:A430, A432:A443, A445:A452, A456:A464, A466:A468, A470:A476, A478:A480, A484:A488, A492, A496:A499, A501:A504, B45:B47, B75:B79, B110:B112, B116:B120, B130:B132, B148:B155, B157, B213:B215, B243:B247, B278:B280, B284:B288, B298:B300, B316:B323, B325, B381:B383, B411:B415, B446:B448, B452:B456, B466:B468, B484:B491, B493, C46, C75:C79, C110:C112, C116:C120, C130:C132, C148:C155, C157, C214, C243:C247, C278:C280, C284:C288, C298:C300, C316:C323, C325, C382, C411:C415, C446:C448, C452:C456, C466:C468, C484:C491, C493, D45:D47, D75:D79, D109:D112, D116:D120, D130:D132, D148:D155, D157:D159, D168, D213:D215, D243:D247, D277:D280, D284:D288, D298:D300, D316:D323, D325:D327, D336, D381:D383, D411:D415, D445:D448, D452:D456, D466:D468, D484:D491, D493:D495, D504, E45:E47, E75:E79, E109:E112, E116:E120, E213:E215, E243:E247, E277:E280, E284:E288, E381:E383, E411:E415, E445:E448, E452:E456
- probes-numeric/t3570-r5-bpser-lngamma-gl.case.json (125): Calc!A73, A142, A144:A147, A165, A168, A170, A172, A174, A180:A182, A184, A186, A188:A190, A215:A220, A222:A227, A229, A236:A238, A252:A254, A256:A258, A283:A291, A308:A310, A312:A315, A326:A328, A333, A339, A341:A343, A351, A357:A360, A366:A368, A370:A372, A374, A376, A387, A389:A392, A394, A396, A403:A407, A410, A414, A419:A425, A428:A429, A431:A432, A434, A436, A441:A442, A448:A451, A457:A458, A461:A463, A465:A466, A469:A470, A477:A478, A480, A483, A594

## Acceptance criteria
- [ ] Each listed cell passes bit-exact, or the Outcome lists the ones still open by kernel, with the model's
      exact rate on them.
- [ ] No GAMMA, GAMMALN, GAMMALN.PRECISE or LN cell of any suite regresses; the known-failure lines in
      eval/baseline/ only shrink.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    the correctly rounded value, from the fit lab's split, with the margin stated before scoring;
  - a fresh log-uniform sweep over the owned domain, generated and oracle-filled after the model is frozen, with its
    result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, every point outside listed;
  - every fitted constant or cut carries `FITTED(T-3937, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cracking the kernels (T-2122) and the T-2464 probes (T-3042); bpser's L (T-3769, T-3771, done).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
