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

