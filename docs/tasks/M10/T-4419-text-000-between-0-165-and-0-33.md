---
id: T-4419
title: "TEXT .000 between 0.165 and 0.33 s: which half-ulp ties of the 2^-72-day offset Excel rounds up"
milestone: M10
lane: functions-text
status: blocked
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
TEXT under `.000` matches Excel where the half-stepped serial is between 2^-19 and 2^-18 days (0.165 to 0.33 s). In
that band the 2^-72-day offset of T-4389 is exactly half an ulp of the serial.

## Context
- T-4389 (`millisecond_offset`, formatter/elapsed.rs; number-formats.md "Fraction digits of a second at a tie")
  adds 2^-72 days to the half-stepped serial under `.000`. Below 2^-19 days that model matches every probed cell.
- In [2^-19, 2^-18) the addition is half an ulp, and the double addition rounds it to even, so only serials with an
  odd last mantissa bit move up. probes/t4389-confirm A1:A40 holds 40 band cells where that rule and "no offset"
  disagree. Excel follows the round-to-even addition in 29 of them and not in 11.
- Visibly-off cells this task owns: probes/t4389-confirm A4, A6, A9, A14:A19, A31, A36.
- The misses all sit at k >= 0.2575 s, but several k in the same range match, so the band edge alone does not
  explain them. Possible lines of attack: the half step's own rounding, an x87 intermediate, or a constant slightly
  different from 2^-72 (T-4389's window is 1.74E-17 to 1.86E-17 s). Probe the band more densely before fitting
  anything.

## Acceptance criteria
- [ ] The owned cells pass, with no other cell of the t4323, t4332, t4370 or t4389 fraction cases moving.
- [ ] Any new constant or rule meets the R4 guards: a hold-out and a fresh sweep generated after the freeze.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Serials below 2^-19 days (T-4389).

## Cells added by T-4419 (2026-11-06)
probes/t4419-band-ulp-map (excel-graph, 2026-11-06; generator and predictions in docs/research/t4419): every `.000` tie
k = 0.1655, 0.1675, ..., 0.3295 s stepped 19 to 9 ulp below, under "s.000". 66 of the 913 cells are 1 ms off (the
engine's prediction, committed before the fill, scores 847):
probes/t4419-band-serials (913 cells) reads the same serials back and passes in full.
- probes/t4419-band-ulp-map.case.json (66): Sheet1!A6, A17, A28, A39, A51, A62, A73, A84, A95, A106, A117, A127, A138, A159, A181, A203, A226, A248, A270, A292, A314, A336, A347, A358, A380, A392, A403, A436, A447, A479, A480, A500, A512, A523, A534, A556, A567, A577, A588, A610, A621, A632, A656, A667, A678, A700, A711, A722, A744, A755, A766, A788, A799, A810, A829, A830, A842, A843, A853, A873, A874, A884, A885, A886, A887, A897

## Blocked because
Blocked because: time-boxed (decision 0056). Opus run, 2026-11-06. The dense probe below shows the band's flips, but no
candidate rule fits them, and shipping the best one would be an unevidenced fit (decision 0188 R4). The task reopens
only on a new hypothesis recorded here.

## Findings (2026-11-06, nothing shipped)
- probes/t4419-band-ulp-map maps the flip per tie. For each of the 83 ties, Excel's digit goes from the tie minus 1 ms
  to the tie at a single ulp step (no tie flips back), but that step is one ulp before or after the engine's on 66
  cells, in both directions.
- Excel computes the serials as the generator assumes: probes/t4419-band-serials (the same 913 formulas without TEXT,
  excel-graph 2026-11-06) returns each one equal to the generator's double.
- No single threshold on the exact distance to the tie separates Excel's ups from its downs. The exact value of
  `y + 0.5 ms`, the double sum and the double sum plus 2^-72 days overlap on 217 to 226 of the 913 cells. So the
  decision rests on float error of the size of a serial ulp (3.7E-17 s), not on the real distance.
- Candidates scored on the 913 cells (docs/research/t4419; the engine 847):
  - offset 2^-72 days added after the half step (the engine) 847, or none 838;
  - a constant c added to the exact sum before the half step's rounding, c = 2^-80 to 2^-61 days, scored on the 1039
    `.000` cells of the t4323/t4332/t4370/t4389 cases: 2^-72 equals the engine (1028, the 11 confirm band misses), every
    other c is worse;
  - x87-style extended intermediates (53 to 64 bits) on the half step and the cut: worse than the engine on the train
    cells (776 to 839 of 1039 `.000` train cells);
  - the cut in other orders (1440,60; 86400; 24,3600; 60,24,60): 994 to 1015 of 1039 train cells, no band gain;
  - milliseconds computed by a factor chain (24,60,60,1000; 86400,1000; 86400000; 1440,60,1000; 24,3600,1000;
    24,60,60000), the half step after any stage, optional 2^-72, 15-digit rounding after any stage
    (search_pipelines.py): best 887/913, (24,60,60,1000) with the half step in hours;
  - the 15-digit rounding done in doubles (r*1e15, r*10 fifteen times, r*1e3*1e12, r/1e-15, r*1e16/10, half up or
    even) on six remainders (search_double15.py): best 869/913.
- Next hypothesis to try: probe the same ties under `[s].000`, `h:mm:ss.000` and `[h]:mm:ss.000` to see whether
  the flip step depends on the format, and the same ties from serial 1 up (`1+k/86400`), where the cut's float
  error differs; a flip that follows the format points at the text path, one that follows the serial at the cut.
