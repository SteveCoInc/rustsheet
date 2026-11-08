---
id: T-4419
title: "TEXT .000 between 0.165 and 0.33 s: which half-ulp ties of the 2^-72-day offset Excel rounds up"
milestone: M10
lane: functions-text
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
TEXT under `.000` matches Excel where the half-stepped serial is between 2^-19 and 2^-18 days (0.165 to 0.33 s). In
that band the 2^-72-day offset of T-4389 is exactly half an ulp of the serial.

## Context
- Decision 0355: this task owns foundational-tier cells, which the tier gate needs 100% right on type, error and text, so it is P1.
- T-4389 (`millisecond_offset`, formatter/elapsed.rs; number-formats.md "Fraction digits of a second at a tie")
  adds 2^-72 days to the half-stepped serial under `.000`. Below 2^-19 days that model matches every probed cell.
- In [2^-19, 2^-18) the addition is half an ulp, and the double addition rounds it to even, so only serials with an
  odd last mantissa bit move up. probes/t4389-confirm A1:A40 holds 40 band cells where that rule and "no offset"
  disagree. Excel follows the round-to-even addition in 29 of them and not in 11.
- Visibly-off cells this task owns: probes/t4389-confirm A4, A6, A9, A14:A19, A31, A36; probes/t4419-band-flip F5,
  F11, G12, H1, H7, H9, H10, H12, H13, I12.
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

## Progress (2026-11-08)
- The 11 cells are decimal millisecond ties t/86400 + n*2^-71 (n = -13..-15) under `s.000`. Excel rounds up for
  k = 178..254 and 273..290 ms and down for 257..265 and 315..326, with the same ulp distances in both groups, so the
  decision follows the tie's position, not the distance below it.
- Ruled out (docs/research/t4466 scratch work, T-4333's count search): any order of x24, x60, x60, x1000 as a chain or a
  field cut on y or frac(y); the half step's constant forms; 80-bit (x87) arithmetic for the step and the cut; 15-digit
  rounding of the seconds then half-up to the millisecond without a half step; the binary side of k/1000. None
  separates the two groups while keeping the other 1,069 `.000` cells.
- The `.0` and `.00` cells need no offset at all (the field cut with 15-digit rounding scores 768 of 768 and 465 of
  465), so the millisecond path is the only one with the 2^-72 fit.

## Progress (2026-11-08, second pass)
- New evidence: probes/t4419-band-flip (excel-graph, 169 cells). It covers 13 ties t/86400000 + n*2^-71, with t from 179.5
  to 327.5 ms and n from -20 to -8, under `s.000`. For every tie Excel's answer switches once, at n = -11 to -15, so
  the decision is a threshold. The engine gets 159 of 169.
- docs/research/t4419/chains.py: the best chains of doubles (every left-chain grouping of 24, 60, 60 and 1000, all
  products exact, then the milliseconds' 15-digit rounding and half up) score 166 of 169. Its docstring lists the
  chains and their misses. Ad-hoc runs that were not committed scored no better: 15-digit rounding after any subset
  of the steps, and a half step in days (with or without 2^-72) followed by a cut, whose best was 161.
- `windows()` prints the error each tie needs in its computed seconds, in units of 5e-16 s, for the 15-digit
  half-unit threshold to hold. Below 0.25 s the needed error is near 0. Just above 0.25 s it turns positive (253.5 and
  255.5: up to +0.15), then negative (258.5 to 266.5, 320.5 at -0.17 and 327.5). That is about one ulp of the seconds,
  2^-54 s or 2^-53 s, either side. So the pattern follows the binade of the seconds at 0.25 s. Next: arithmetic whose
  rounding changes at that binade, such as seconds formed as hours*3600 or from a split at 0.25, or a 2-step product
  with an intermediate in [0.125, 0.5).

## Progress (2026-11-08, third pass; research paused by the owner)
- This task owns every visibly-off cell of probes/t4419-band-formats and probes/t4419-band-dense (the evidence below).
- probes/t4419-band-formats (excel-graph, 455 cells): the 13 band-flip ties at n = -16..-10 under s.000, ss.000,
  [s].000, mm:ss.000 and h:mm:ss.000. Every tie switches at the same n under all five formats, so the decision depends
  only on the value, not on the field path.
- probes/t4419-band-dense (excel-graph, 1,148 cells): every tie k+0.5 ms, k = 165..328, at n = -16..-10 under s.000.
  All 164 ties switch exactly once (docs/research/t4419/dense.py). The engine gets 1,035. As a fresh test, the chains
  found on band-flip score no better: x/(1/24)*60*60*1000 (168 of band-flip's 169) gets 1,085. The best of the families
  searched in uncommitted runs gets 1,094. Those families were products of 24, 60, 60 and 1000 in any order, with *f or
  /(1/f), half steps in days and cuts, and directed rounding per step at 53 or 64 bits.
- struct.py: the threshold sits in the exact-tie frame (slope -1.23 on the tie double's offset), 13.84 x-ulps below the
  tie on average (sd 0.69). Within runs of k it drifts linearly, then resets.
- features.py: a constant threshold fits 100 of the 164 tie windows. Rounding errors of doubles built from k, or of the
  products at the tie, add at most 1, so the scatter is per-value arithmetic not yet identified.
- The owner paused the research on 2026-11-08. These cells stay known failures owned here, and the foundational gate
  stays unmet until the rule is found.
