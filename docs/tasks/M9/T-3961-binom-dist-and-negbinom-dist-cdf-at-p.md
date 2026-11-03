---
id: T-3961
title: "BINOM.DIST and NEGBINOM.DIST cdf at p below 0.01 and a huge n: Excel is 269 to 64,789 ulp off the math where ours is close (11 visibly-off t3352 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3960]
campaign: visible
---

## Goal
The 11 cumulative BINOM.DIST and NEGBINOM.DIST cells below (p below 0.01, n or the failure count from 1E5 to 1E8)
are within 15 significant digits of Excel, from a model of how Excel forms or evaluates its cdf there. If no model is
found, the cells are an UNVERIFIED statistics.md entry with the hypotheses ruled out.

## Context
- Gap triage 2026-11-03b (`make eval` on origin/main c2062e5c8, excel-graph). Split from T-3774, which is blocked
  (time-boxed on the pmf; its cdf cells were never attempted), together with T-3959. T-3959 takes the other cdf
  cells. These 11 are a separate regime. Ours minus Excel is hundreds to tens of thousands of ulp, always positive
  (Excel is lower). Where a math value exists, Excel is far off it and ours is close:

  | cell | formula | Excel | ours | ours - Excel (ulp) | Excel - math, ours - math (ulp) |
  |---|---|---|---|---|---|
  | large-n A34 | `BINOM.DIST(70,100000,0.001,TRUE)` | 0.0009667927349360346 | ...61112 | +706 | -961, -255 |
  | large-n A52 | `BINOM.DIST(526,600000,0.001,TRUE)` | 0.001107528039888131 | ...886653 | +2464 | |
  | large-n A70 | `BINOM.DIST(905,1000000,0.001,TRUE)` | 0.0012086874997822498 | ...26675 | +1926 | |
  | large-n A88 | `BINOM.DIST(9700,10000000,0.001,TRUE)` | 0.0013064181746120147 | ...2302 | +1325 | |
  | sweep A132 | `BINOM.DIST(2461,16194261,0.000154374041567456,TRUE)` | 0.22119048320161896 | 0.22119048320341644 | +64761 | -64789, -28 |
  | sweep A147 | `BINOM.DIST(5887,623666,0.00947288369372644,TRUE)` | 0.39554725622481485 | ...86125 | +836 | |
  | sweep A156 | `BINOM.DIST(8259,868228,0.00961856313687781,TRUE)` | 0.15689278088052766 | ...53513 | +269 | |
  | sweep A171 | `NEGBINOM.DIST(106931620,1903,1.81058846428433e-05,TRUE)` | 0.7771607360384137 | 0.7771607360422212 | +34295 | -34302, -7 |
  | sweep A184 | `NEGBINOM.DIST(1041120,4940,0.00472551518826802,TRUE)` | 0.5199426990783227 | ...85869 | +2380 | |
  | sweep A190 | `NEGBINOM.DIST(544583,45,0.000105461509284258,TRUE)` | 0.9603825121576379 | ...76842 | +417 | -417, 0 |
  | sweep A208 | `NEGBINOM.DIST(3827391,6017,0.00159874266118577,TRUE)` | 0.924466231105322 | ...5395 | +658 | |

  Math values are the report's math sidecar (50 digits) and, for A34, mpmath `betainc` at 60 digits. Excel's worst
  relative error is 8.1E-12 (A132). That is far above basym's 100·eps stop and bfrac's 15·eps, so Excel's own branch
  arithmetic does not explain it. Something larger is at work: argument formation, or a different route at small p.
- Lead (UNVERIFIED, gap triage): the error grows with n·p·(the cdf's sensitivity to p), the size an error in the low
  bits of p would give. One test rebuilt p from the complement, p' = 1 - fl(1 - p), and took the math at p':
  - Excel moved closer to the math on large-n A34 (-961 to -714 ulp), A52 (-2780 to -2425), A70 (-2383 to -1884) and
    sweep A190 (-417 to -247);
  - it moved much further on NEGBINOM.DIST at small r (large-n A146 -9 to +373; sweep A171 -34302 to +287014).
  So "Excel uses p' throughout" is ruled out. A mixed formation (x from fl(1 - p) and y from p, inside λ or the
  exponent of basym or bgrat) is untested.
- Needs an oracle sweep. The cached workbooks have only these 11 points in the regime (p < 0.01 with n·p or r·(1-p)/p
  in the hundreds or more), too few for a model with a hold-out. T-3960 fills a dense train case and a separate hold-out
  first; this task depends on it.
- Code: `crates/rustsheet-engine/src/functions/statistical/binom.rs`, `bratio/{mod,basym,bfrac,bup,bgrat}.rs`.
  `bratio::dispatch` tells which branch each point takes. Method: docs/research/excel-kernel-playbook.md.

Cells this task owns (T-3774 and T-3959 own the other cells of both cases):
- probes/t3352-binom-negbinom-large-n.case.json (4): Calc!A34, A52, A70, A88
- probes-numeric/t3352-binom-negbinom-sweep.case.json (7): Calc!A132, A147, A156, A171, A184, A190, A208
- probes-numeric/t3960-binom-negbinom-small-p-cdf.case.json (393; T-3960, every cell `rustsheet-eval show` lists as failing on 2026-11-03):
  Calc!A1, Calc!A2, Calc!A3, Calc!A4, Calc!A5, Calc!A6, Calc!A7, Calc!A8, Calc!A9, Calc!A10, Calc!A11, Calc!A13,
  Calc!A14, Calc!A15, Calc!A16, Calc!A17, Calc!A18, Calc!A20, Calc!A22, Calc!A24, Calc!A25, Calc!A26, Calc!A27,
  Calc!A29, Calc!A30, Calc!A31, Calc!A32, Calc!A34, Calc!A35, Calc!A36, Calc!A37, Calc!A38, Calc!A39, Calc!A40,
  Calc!A42, Calc!A44, Calc!A45, Calc!A47, Calc!A49, Calc!A50, Calc!A51, Calc!A52, Calc!A53, Calc!A54, Calc!A55,
  Calc!A57, Calc!A59, Calc!A62, Calc!A63, Calc!A64, Calc!A65, Calc!A66, Calc!A67, Calc!A68, Calc!A69, Calc!A70,
  Calc!A71, Calc!A72, Calc!A73, Calc!A74, Calc!A75, Calc!A76, Calc!A78, Calc!A80, Calc!A81, Calc!A83, Calc!A85,
  Calc!A87, Calc!A88, Calc!A90, Calc!A91, Calc!A93, Calc!A94, Calc!A95, Calc!A96, Calc!A97, Calc!A98, Calc!A99,
  Calc!A100, Calc!A101, Calc!A102, Calc!A104, Calc!A105, Calc!A106, Calc!A107, Calc!A109, Calc!A110, Calc!A111,
  Calc!A113, Calc!A114, Calc!A115, Calc!A116, Calc!A117, Calc!A118, Calc!A122, Calc!A123, Calc!A126, Calc!A127,
  Calc!A128, Calc!A130, Calc!A132, Calc!A133, Calc!A135, Calc!A136, Calc!A137, Calc!A138, Calc!A139, Calc!A140,
  Calc!A141, Calc!A143, Calc!A144, Calc!A145, Calc!A146, Calc!A147, Calc!A148, Calc!A149, Calc!A150, Calc!A151,
  Calc!A152, Calc!A153, Calc!A154, Calc!A155, Calc!A157, Calc!A158, Calc!A159, Calc!A162, Calc!A163, Calc!A164,
  Calc!A165, Calc!A166, Calc!A168, Calc!A169, Calc!A170, Calc!A171, Calc!A172, Calc!A173, Calc!A176, Calc!A177,
  Calc!A178, Calc!A179, Calc!A181, Calc!A182, Calc!A183, Calc!A184, Calc!A185, Calc!A186, Calc!A187, Calc!A188,
  Calc!A189, Calc!A190, Calc!A191, Calc!A192, Calc!A193, Calc!A195, Calc!A197, Calc!A198, Calc!A200, Calc!A201,
  Calc!A202, Calc!A203, Calc!A204, Calc!A205, Calc!A206, Calc!A207, Calc!A208, Calc!A209, Calc!A210, Calc!A211,
  Calc!A212, Calc!A213, Calc!A214, Calc!A215, Calc!A216, Calc!A217, Calc!A218, Calc!A219, Calc!A220, Calc!A221,
  Calc!A222, Calc!A223, Calc!A224, Calc!A225, Calc!A226, Calc!A227, Calc!A228, Calc!A229, Calc!A230, Calc!A231,
  Calc!A232, Calc!A233, Calc!A234, Calc!A235, Calc!A236, Calc!A237, Calc!A238, Calc!A239, Calc!A240, Calc!A241,
  Calc!A242, Calc!A243, Calc!A244, Calc!A245, Calc!A246, Calc!A247, Calc!A248, Calc!A249, Calc!A250, Calc!A251,
  Calc!A252, Calc!A253, Calc!A254, Calc!A255, Calc!A256, Calc!A257, Calc!A258, Calc!A259, Calc!A260, Calc!A261,
  Calc!A262, Calc!A263, Calc!A264, Calc!A265, Calc!A266, Calc!A267, Calc!A268, Calc!A269, Calc!A270, Calc!A271,
  Calc!A272, Calc!A273, Calc!A274, Calc!A275, Calc!A276, Calc!A277, Calc!A278, Calc!A279, Calc!A280, Calc!A281,
  Calc!A282, Calc!A283, Calc!A284, Calc!A285, Calc!A286, Calc!A287, Calc!A289, Calc!A290, Calc!A291, Calc!A292,
  Calc!A293, Calc!A294, Calc!A295, Calc!A296, Calc!A297, Calc!A298, Calc!A299, Calc!A300, Calc!A301, Calc!A303,
  Calc!A304, Calc!A306, Calc!A307, Calc!A308, Calc!A309, Calc!A310, Calc!A311, Calc!A312, Calc!A313, Calc!A314,
  Calc!A315, Calc!A316, Calc!A317, Calc!A318, Calc!A319, Calc!A320, Calc!A321, Calc!A322, Calc!A323, Calc!A324,
  Calc!A325, Calc!A326, Calc!A327, Calc!A328, Calc!A329, Calc!A330, Calc!A331, Calc!A332, Calc!A333, Calc!A334,
  Calc!A335, Calc!A336, Calc!A337, Calc!A338, Calc!A339, Calc!A340, Calc!A341, Calc!A342, Calc!A343, Calc!A344,
  Calc!A345, Calc!A346, Calc!A347, Calc!A348, Calc!A349, Calc!A350, Calc!A351, Calc!A352, Calc!A353, Calc!A354,
  Calc!A355, Calc!A356, Calc!A357, Calc!A358, Calc!A359, Calc!A360, Calc!A361, Calc!A362, Calc!A363, Calc!A364,
  Calc!A365, Calc!A366, Calc!A367, Calc!A368, Calc!A369, Calc!A370, Calc!A371, Calc!A372, Calc!A373, Calc!A374,
  Calc!A375, Calc!A376, Calc!A377, Calc!A378, Calc!A379, Calc!A380, Calc!A381, Calc!A382, Calc!A383, Calc!A384,
  Calc!A385, Calc!A386, Calc!A387, Calc!A388, Calc!A389, Calc!A391, Calc!A392, Calc!A393, Calc!A394, Calc!A395,
  Calc!A396, Calc!A397, Calc!A398, Calc!A399, Calc!A400, Calc!A401, Calc!A402, Calc!A403, Calc!A404, Calc!A405,
  Calc!A406, Calc!A407, Calc!A408, Calc!A409, Calc!A410, Calc!A411, Calc!A412, Calc!A413, Calc!A414, Calc!A415,
  Calc!A416, Calc!A417, Calc!A418, Calc!A419, Calc!A420, Calc!A421, Calc!A422, Calc!A423, Calc!A424, Calc!A425,
  Calc!A426, Calc!A427, Calc!A428, Calc!A429, Calc!A430, Calc!A431, Calc!A432, Calc!A433, Calc!A434, Calc!A435,
  Calc!A436, Calc!A437, Calc!A438, Calc!A439, Calc!A440
- probes-numeric/t3960-binom-negbinom-small-p-cdf-hold.case.json (176; T-3960, every cell `rustsheet-eval show` lists as failing on 2026-11-03):
  Calc!A1, Calc!A2, Calc!A3, Calc!A4, Calc!A5, Calc!A8, Calc!A9, Calc!A11, Calc!A13, Calc!A14, Calc!A15, Calc!A17,
  Calc!A18, Calc!A19, Calc!A21, Calc!A22, Calc!A23, Calc!A24, Calc!A25, Calc!A26, Calc!A27, Calc!A28, Calc!A29,
  Calc!A30, Calc!A31, Calc!A33, Calc!A34, Calc!A35, Calc!A36, Calc!A39, Calc!A40, Calc!A42, Calc!A44, Calc!A45,
  Calc!A46, Calc!A47, Calc!A48, Calc!A49, Calc!A51, Calc!A52, Calc!A53, Calc!A54, Calc!A55, Calc!A56, Calc!A57,
  Calc!A59, Calc!A61, Calc!A62, Calc!A63, Calc!A64, Calc!A65, Calc!A66, Calc!A67, Calc!A68, Calc!A69, Calc!A73,
  Calc!A74, Calc!A75, Calc!A76, Calc!A77, Calc!A78, Calc!A79, Calc!A80, Calc!A81, Calc!A82, Calc!A83, Calc!A84,
  Calc!A86, Calc!A87, Calc!A88, Calc!A90, Calc!A92, Calc!A93, Calc!A94, Calc!A95, Calc!A96, Calc!A97, Calc!A98,
  Calc!A99, Calc!A101, Calc!A102, Calc!A103, Calc!A104, Calc!A105, Calc!A106, Calc!A107, Calc!A108, Calc!A109,
  Calc!A110, Calc!A111, Calc!A112, Calc!A113, Calc!A114, Calc!A115, Calc!A116, Calc!A117, Calc!A118, Calc!A119,
  Calc!A120, Calc!A121, Calc!A122, Calc!A123, Calc!A124, Calc!A125, Calc!A126, Calc!A127, Calc!A128, Calc!A129,
  Calc!A130, Calc!A131, Calc!A132, Calc!A133, Calc!A134, Calc!A135, Calc!A136, Calc!A137, Calc!A138, Calc!A139,
  Calc!A140, Calc!A141, Calc!A142, Calc!A143, Calc!A144, Calc!A145, Calc!A146, Calc!A147, Calc!A149, Calc!A150,
  Calc!A151, Calc!A152, Calc!A153, Calc!A154, Calc!A155, Calc!A156, Calc!A157, Calc!A158, Calc!A159, Calc!A160,
  Calc!A161, Calc!A162, Calc!A163, Calc!A164, Calc!A165, Calc!A166, Calc!A167, Calc!A168, Calc!A169, Calc!A170,
  Calc!A171, Calc!A172, Calc!A173, Calc!A174, Calc!A175, Calc!A176, Calc!A177, Calc!A178, Calc!A179, Calc!A180,
  Calc!A181, Calc!A182, Calc!A183, Calc!A184, Calc!A185, Calc!A186, Calc!A187, Calc!A188, Calc!A189, Calc!A190,
  Calc!A191, Calc!A192, Calc!A193, Calc!A194, Calc!A195, Calc!A196, Calc!A197, Calc!A198

## Acceptance criteria
- [ ] The Outcome has, for each candidate (including the p' form above and every mixed formation tried), its
      exact / within-4-ulp / within-15-digit counts on T-3960's train case and then once on its hold-out case.
- [ ] Each of the 11 cells above is within half a unit of the 15th significant digit of Excel, or is in statistics.md
      "BINOM.DIST and NEGBINOM.DIST at a large n" as UNVERIFIED with the hypotheses ruled out.
- [ ] The large-n lines A34, A52, A70 and A88 that pass leave eval/baseline/probes.txt (`make eval-baseline`). No new
      known-failure line appears in any eval/baseline file. No gated BETA.DIST, F.DIST, POISSON.DIST,
      BINOM.DIST.RANGE or BINOM.INV cell regresses.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    for the correctly rounded value, on T-3960's split, with the margin stated before scoring;
  - a fresh sweep over the regime, generated and oracle-filled (excel-graph) after the model is frozen (not T-3960's
    cases), with its result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, with every point outside it listed;
  - every fitted constant or cut carries `FITTED(T-3961, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other cdf cells (T-3959) and the pmf (T-3774).
- BINOM.DIST.RANGE and BINOM.INV past the walk (T-3444's follow-ups).

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when each of the 11 cells is within half a unit of the 15th significant
digit of Excel, or is an UNVERIFIED statistics.md entry naming the ruled-out hypotheses. Excel is the inaccurate one
here, so this means reproducing Excel's formulation (time-boxed, decision 0056).
