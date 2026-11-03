---
id: T-3851
title: "IMSEC, IMCSC and IMSECH of 705+705i are one ulp off Excel in the 3.768E-307 part"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
IMSEC, IMCSC and IMSECH of 705+705i print Excel's last digit, or the task records why no model in doubles can.

## Context
- probes/im-trig-magnitude-grid D161, E161, I161 (excel-graph, T-3642): `IMSEC(COMPLEX(705,705))` is
  3.76819332944316E-307+1.27412580321457E-306i in Excel, ours 3.76819332944317E-307+...; IMCSC and IMSECH have the
  same digit swap in their 3.768E-307 part. They are the grid's last cells outside T-3616's band and T-2668/T-2914's
  IMCSCH cells after T-3758 moved the large-argument sines onto the x87 reduction.
- The path is the overflowed reciprocal (`overflowed_reciprocal` in
  crates/rustsheet-engine/src/functions/engineering/complex/trig.rs): the trig parts times 2e^-|v| with v = 705, which
  lands near the bottom of the normal range. Likely the scaling (`2·EXP(-v)` vs `EXP(ln 2 - v)`, or the order of the
  product), not the sine; sin 705 and cos 705 are SIN's and COS's own.
- IMCSCH of 705+705i (grid J161) has the same digit since T-3881: inside IMCSCH's negated band it is -IMSECH(z)
  through IMSECH's parts, Excel -3.76819332944316E-307+1.27412580321457E-306i, ours ...317E-307+.... It follows I161.
- Phase 1b under decision 0188 R5: one digit at the 15th place, not visibly off.

## Acceptance criteria
- [x] D161, E161 and I161 of probes/im-trig-magnitude-grid pass with no other IM* cell moving, or an Outcome lists the
      forms tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMCSCH's tiny parts (T-2668, T-2914) and T-3616's y = 709.78 band.

## Outcome
Fixed by T-3883's change to `overflowed_parts` (each part divided by Excel's EXP(|v|)/2, engineering.md "IM*
reciprocals past the denominator's overflow (T-3883)"): D161, E161, I161 and J161 of probes/im-trig-magnitude-grid now
pass and their known-failure lines left eval/baseline/probes.txt; no other cell moved. No separate PR.
