---
id: T-4071
title: "BETA.DIST at a = b = 1/2: port the reflection above x = 0.9 (855 of 962 against the direct form's 737) once R4.4 evidence or an owner ruling allows a 1-ulp cut"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
campaign: accuracy
---

## Goal
BETA.DIST's (and BETADIST's) cdf at a = b = 1/2 takes Excel's reflection above x = 0.9, if decision 0188 R4.4 allows
the cut.

## Context
- statistics.md "BETA.DIST at a = b = 1/2 is an arctangent closed form, reflected above x = 0.9" (T-3192). The engine
  ports the direct form `2*atan(sqrt x/sqrt y)/pi` everywhere (`incomplete_beta::half_half_cdf`).
- The switch to `(0.5 - w(y)) + 0.5` above x = 0.9 scores 281 / 287 / 287 of the three sweeps
  (probes-numeric/t3192-beta-half-half, -2, -3), 855 of 962 in total. The direct form scores 270 / 228 / 239, 737 in
  total (docs/research/t3192/score_half.py).
- The two forms differ by 1 ulp on every cell, which is below decision 0112's floor. The cut is therefore last-digit
  (Phase 1b) work. It needs a desktop-backed reproducibility check or an owner ruling (decision 0188 R4.4), the R4.1
  split, and an `r4-T-4071` Gate A batch.
- `rustsheet-accuracy generate BETA.DIST` draws the shapes and cannot pin a = b = 1/2. Add a shape pin to
  tools/src/rustsheet_tools/accuracy (defs/beta.py, cli.py) first.
- T-3886 found no other closed form that does better (docs/research/t3886/RESULTS.md): no atan, asin or acos form,
  and no 64-bit-intermediate variant, beats the direct form. Excel's x = 0.5 value of 0.5000000000000001 has no
  model.

## Acceptance criteria
- [ ] The switch is ported with its R4 evidence (hold-out, fresh sweep, Gate A batch, `FITTED` tag), and no gated
      cell regresses; or the ruling against it is recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The remaining 1 to 2 ulp misses with no form (13 to 54 per sweep under the switch), and x = 0.5.
