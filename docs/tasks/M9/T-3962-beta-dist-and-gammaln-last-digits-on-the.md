---
id: T-3962
title: "BETA.DIST and GAMMALN last digits on the T-3570, T-3769 and T-3879 bpser probes once T-2122's kernels ship (1,093 cells, Phase 1b)"
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
BETA.DIST and GAMMALN return Excel's bits on the 1,093 last-digit cells of the four bpser probes listed below, once
T-2122's lnGamma kernels ship through T-3042. Otherwise the Outcome names each cell left, with its cause.

## Context
- Gap triage 2026-11-03b (`make eval` on origin/main c2062e5c8, excel-graph evidence). A per-cell ownership run
  (`scripts/exit_owners.py`'s `owned` over every failing cell against every open task, deferred ones included) found
  these cells with no open owner. They are the only unowned failing Excel cells of the run, apart from four pending
  cases and the COSH cell added to T-2744. All four cases are in probes-numeric (ungated). Every listed cell is
  NumericUlp or NumericClose. Six are just past half a unit of the 15th digit (below), and decision 0188 R1(c) leaves
  them out as Phase 1b evidence: Excel and ours are both within 1E-9 of the math.
- By case (BETA.DIST cells in columns A-H of t3570-r5-bpser-lg, GAMMALN in I and J; one column elsewhere):
  - t3570-r5-bpser-lg (T-3570, 523: 368 BETA.DIST, 155 GAMMALN). The GAMMALN columns are Excel's L inputs for the same
    row's a0. Of the 70 rows with a BETA.DIST miss, 66 also miss a GAMMALN cell in the row. The other 4 are rows
    428-431, column H only.
  - t3570-r5-bpser-lngamma-beta (249 BETA.DIST). Of the 249 misses, 229 sit at an a0 where T-3937's
    t3570-r5-bpser-lngamma-gl GAMMALN(a0) also misses.
  - t3879-bpser-split-fresh (T-3879's R4.2 fresh sweep, 319: 177 BETA.DIST, 142 GAMMALN at the 864 L arguments).
    T-3879's Outcome scores its split form at 2,300 of 2,301 exact when L uses Excel's own GAMMALN. With the engine's
    GAMMALN, these 177 BETA.DIST cells miss.
  - t3769-r6-bpser-beta (2 BETA.DIST: A2983, A2998).
  So the main cause is the engine's lnGamma kernels (statistics.md "GAMMALN is Excel's own x87 routine (T-1941,
  T-2128)"), which T-2122 cracks and T-3042 ships. That makes this the same work as T-3937, scored on these four
  probes. Do not tune on these cells: they are hold-out for T-2122's kernels.
- Not explained by GAMMALN(a0) (20 lngamma-beta cells, 4 lg cells). Classify these first after the kernels land:
  - lngamma-beta A1398:A1402 (x 0.0213) and A1558:A1562 (x 0.0371), b = 6.25, a0 from 0.5 + 37·2^-53 to
    0.5 + 41·2^-53: ours is 5 to 10 ulp below Excel, and GAMMALN(a0) is bit-exact there. The run of consecutive a0
    shows a jump at 0.5 + 44·2^-53 in both Excel and ours. Ours drifts about 2.3 ulp per step before it and Excel
    about 1.7, so look at bpser's other lnGamma terms (a0 + b0, algdiv), not GAMMALN(a0). Example: A1561
    `BETA.DIST(0.0371,(8388608*2^-24+5*2^-50),6.25,TRUE)` gives Excel 0.4996152124105067 and ours 0.49961521241050616
    (math 0.49961521241050666: Excel +1 ulp, ours -9).
  - lngamma-beta A11, A12, A16:A18 and A171:A174, A176 (b = 2.5, 1 ulp each); lg H428:H431.
- Visibly off by the strict 15-digit rule, all within Gate A's envelope: lngamma-beta A1561:A1564 (10 to 12 ulp),
  t3879-bpser-split-fresh A1276 (3 ulp) and A2923 `GAMMALN(0.866974768932519)` (4 ulp). Decision 0188 R5 defers
  last-digit BETA.DIST work to Phase 1b. If an eval counts one of them in R1(c), replace `defer:` with a `campaign:`
  stage.
- Code: `crates/rustsheet-engine/src/functions/statistical/bratio/bpser.rs`, `gamma_kernels.rs`, `gammaln.rs`.
  Restart file for the kernels: docs/research/t1941/HANDOFF2.md on agent/research/T-1941-crack-gammaln-gamma-ln.

Cells (gap triage 2026-11-03b; 1,093 cells):
- probes-numeric/t3570-r5-bpser-lg.case.json (523): Calc!A29, A49:A52, A80, A82, A96, A111, A116, A122, A126, A136:A137, A142, A149, A158:A160, A166, A197, A218, A221, A241, A249, A264, A277:A278, A290, A304:A305, A307, A310, A331, A365, A385:A386, A388:A389, A409, A416:A417, A445:A446, A458, A472:A473, A475, A478, B29, B49:B52, B73, B80, B82, B96, B111:B112, B116, B122, B136, B149, B158:B160, B166, B197, B218, B221, B241, B248:B249, B264, B277:B278, B290, B296, B304:B305, B307, B310, B365, B386, B389, B409, B416:B417, B445:B446, B458, B472:B473, B475, B478, C29, C49, C51:C52, C73, C80, C82, C96, C111, C116, C122, C126, C136, C149, C158:C160, C166, C197, C218, C221, C241, C249, C277:C278, C284, C290, C296, C304:C305, C307, C310, C331, C365, C386, C389, C409, C417, C445:C446, C458:C459, C472:C473, C475, C478, D29, D49, D51:D52, D73, D80, D82, D96, D111, D116, D122, D136:D137, D142, D149, D158:D160, D166, D197, D218, D221, D241, D249, D277:D278, D290, D304:D305, D307, D310, D365, D386, D389, D409, D417, D445:D446, D458, D472:D473, D475, E29, E49, E51:E52, E80, E82, E96, E111, E116, E122, E136, E149, E158:E160, E166, E218, E221, E241, E249, E277:E278, E290, E304:E305, E307, E310, E385:E386, E389, E409, E416:E417, E432, E445:E446, E458, E472:E473, E475, E478, F29, F49, F51:F53, F80, F82, F96, F111, F116, F121:F122, F126, F136, F149, F158:F160, F166, F197, F218, F221, F241, F249, F264, F277:F278, F290, F296, F304:F305, F307, F310, F331, F365, F386, F389, F409, F417:F418, F432, F445:F446, F458:F459, F472:F473, F475, F478, G29, G49, G51:G53, G80, G82, G96, G111, G116, G122:G123, G136:G137, G149, G158:G160, G166, G197, G218, G221, G241, G249, G264, G277:G278, G290, G296, G304:G305, G307, G310, G365, G386, G389, G409, G417, G445:G446, G464, G472:G473, G475, G478, H29, H49:H50, H80, H82, H96, H111, H116, H122, H136, H139, H149, H158:H160, H166, H197, H217:H221, H241, H249, H277:H278, H290, H296, H304:H305, H307, H310, H365, H385:H386, H388:H389, H409, H416:H418, H428:H431, H445:H446, H464, H471, I29, I49:I53, I73, I80:I82, I96, I109:I112, I116, I121:I124, I126, I128, I135:I139, I142, I145:I149, I154:I159, I161, I163, I166:I168, I197, I217:I221, I241, I248:I250, I264, I277:I280, I284, I289:I292, I294, I296, I303:I307, I310, I313:I317, I322:I327, I329, I331, I334:I336, I365, I385:I389, I409, I416:I418, I432, I445:I448, I452, I457:I460, I462, I464, I471:I475, I478, I481:I485, I490:I495, I497, I499, I502:I504, J23:J24, J28:J35, J41:J48, J119:J120, J159:J161
- probes-numeric/t3570-r5-bpser-lngamma-beta.case.json (249): Calc!A11:A12, A16:A18, A73, A142, A144:A145, A147, A171:A174, A176, A233, A302, A304:A305, A307, A328, A330, A332, A350, A375:A377, A379, A382, A384, A386, A396:A397, A443:A446, A449, A472:A475, A485, A488, A490, A492, A494, A500:A502, A504, A506, A508, A510, A535, A537, A539, A542, A544, A546, A556:A558, A572:A573, A603:A606, A610:A611, A628:A629, A632:A635, A653, A663, A671, A679:A680, A686, A690, A692, A694, A707, A710, A724, A726, A730, A742, A744, A756, A761, A778, A786, A789, A813, A823, A831, A839:A840, A846, A852, A854, A867, A870, A884, A886, A890, A904, A916, A921, A938, A946, A949, A1033, A1102, A1104:A1107, A1193, A1262, A1264:A1267, A1288, A1290, A1292, A1294, A1300:A1302, A1304, A1306, A1308, A1310, A1335, A1337:A1339, A1343, A1345, A1347, A1349, A1357, A1377, A1398:A1411, A1428, A1432, A1435, A1445, A1448, A1450, A1452, A1454, A1460:A1462, A1464, A1466, A1468:A1470, A1495, A1497:A1499, A1503, A1505, A1507, A1509, A1516:A1517, A1533:A1534, A1536:A1537, A1558:A1571, A1588, A1590, A1593:A1594, A1606, A1621, A1631, A1667, A1670:A1671, A1684, A1687, A1690, A1711, A1714, A1716, A1729, A1737, A1746, A1749, A1757, A1766, A1779, A1781, A1791, A1814, A1827, A1830:A1831, A1844, A1847, A1850, A1871, A1874, A1876, A1889, A1891, A1897, A1906, A1909, A1917
- probes-numeric/t3879-bpser-split-fresh.case.json (319): Calc!A41:A48, A57:A64, A146:A147, A151, A169:A184, A209:A216, A286:A293, A398:A405, A494:A501, A510:A512, A515:A517, A691, A774:A775, A777:A781, A942:A949, A994, A1150:A1157, A1206:A1213, A1270:A1277, A1336, A1479, A1481, A1483:A1484, A1526:A1533, A1614:A1621, A1702:A1703, A1706, A1708:A1709, A1750:A1757, A1766:A1773, A1926:A1933, A2086, A2089:A2092, A2174:A2181, A2547, A2559, A2571, A2601, A2609, A2615, A2620, A2626, A2642, A2682, A2703, A2711, A2728, A2733, A2735, A2737, A2752, A2762, A2766:A2767, A2769, A2777, A2792, A2796, A2800, A2811, A2814, A2819, A2823, A2826, A2830:A2831, A2840, A2843, A2850, A2855:A2856, A2859:A2860, A2869, A2874:A2877, A2879:A2883, A2885:A2895, A2898, A2902:A2904, A2906, A2909, A2912:A2917, A2919:A2923, A2925:A2931, A2933:A2944, A2946:A2947, A2949:A2952, A2955, A2957:A2958, A2960, A2962:A2963, A2966, A2968, A2971:A2973, A2975:A2977, A2979:A2980, A2982:A2987, A2991, A2993, A2995:A2997, A2999, A3001:A3002, A3004:A3006, A3008:A3012, A3016, A3049
- probes-numeric/t3769-r6-bpser-beta.case.json (2): Calc!A2983, A2998

## Acceptance criteria
- [ ] Each listed cell passes bit-exact, or the Outcome lists the ones still open by cause (lnGamma kernel and
      region, the a0 near 1/2 band above, other), with the model's exact rate on each probe.
- [ ] No BETA.DIST, F.DIST, BINOM.DIST, NEGBINOM.DIST, GAMMA, GAMMALN, GAMMALN.PRECISE or LN cell of any suite
      regresses; the known-failure lines in eval/baseline/ only shrink.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    the correctly rounded value, from the fit lab's split, with the margin stated before scoring (these four probes
    are hold-out for T-2122's kernels, never train);
  - a fresh log-uniform sweep over the owned domain, generated and oracle-filled after the model is frozen, with its
    result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, every point outside listed;
  - every fitted constant or cut carries `FITTED(T-3962, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cracking the kernels (T-2122) and shipping them for the T-2464 probes (T-3042). The lg-aux and lngamma-gl GAMMALN,
  GAMMA and LN(GAMMA) cells (T-3937, T-2122).
- bpser's split-z form (T-3879, done) and its L pins (T-3769, T-3771, done).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is within Gate A's envelope and is Phase 1b evidence under decision 0188
R1(c) and R5, so the task waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit,
or when an eval counts one of its cells as visibly off in R1(c) (then give it a `campaign:` stage instead).
