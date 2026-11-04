---
id: T-4012
title: "IMSECH complex quotient residue: t3721 C386's imaginary part one 15th digit off (T-3982 leftover)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
probes-numeric/t3721-im-hyperbolic-small-x-fresh-sweep S!C386 prints Excel's text, or stays recorded as UNVERIFIED
residue, with a rule that separates it from the IMSECH texts the current quotient already fits.

## Context
- T-3982 (done) fixed the other leftover (C390, real IMSECH) and left this one: probes-numeric/t3721-im-hyperbolic-small-x-fresh-sweep
  S!C386 `IMSECH("-0.00529114829570232-0.0605498694819928i")`, Excel "1.0018218211334-0.00032135183420584i", ours
  "...-0.000321351834205841i" (one unit in the 15th digit of the imaginary part). The parse is exact (IMREAL and
  IMAGINARY of the text are the correctly rounded doubles), and cosh x does not move it (T-3930).
- T-3982 scored quotient variants of `sech_parts` (crates/rustsheet-engine/src/functions/engineering/complex/trig.rs)
  over the 1040 cached complex IMSECH texts with |x| <= 0.0105 (current: 1003 fit). Each variant that gives C386
  Excel's digit loses others: `-x_sinh * (y_sin / t)` +1 -41, `t` with `y_cos.mul_add(y_cos, sinh²)` +1 -30,
  `1/(cosh z)` as `conj/|cosh z|²` +1 -84, `t = (cosh 2x + cos 2y)/2` +1 -141, `t = cosh²x - sin²y` +1 -272,
  `t = sinh²x + 1 - sin²y` +1 -201. Variants that keep C386 at ...841: `sinh²x` fused (+0 -0 here, -4 over all 1489 complex IMSECH texts), libm sinh (-27),
  libm cosh (-1). One cell with no separating rule is not a fit (decision 0056); docs/excel-semantics/engineering.md
  "IMSINH, IMCOSH, IMSECH and IMCSCH take libm's cosh below |x| 1E-4" records it.
- Deferred to phase 1b: a single last-digit cell, no structural candidate fits it without losses.

## Acceptance criteria
- [ ] probes-numeric/t3721-im-hyperbolic-small-x-fresh-sweep S!C386 matches Excel's text with a rule chosen on a
      dense oracle sweep near it (a new research case with a hold-out, decision 0188 R4), and no other IMSECH oracle
      cell changes from pass to fail; or the Outcome keeps it as residue with the candidates ranked on that sweep.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Real IMSECH (T-3982), IMCSCH (T-3912, T-3914).
