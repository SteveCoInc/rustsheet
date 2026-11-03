---
id: T-3795
title: "IMSQRT, IMLOG10, IMLOG2 and IMSECH last digits on the T-3643 probes: 31 cells one 15-digit unit off"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The 31 cells of the T-3643 probes where the engine prints one digit off Excel in the 15th print Excel's digits, from a
stated kernel model rather than per-point patches.

## Context
Follow-up of T-3643 (docs/excel-semantics/engineering.md, the T-3643 entries under the IMLN/IMLOG modulus). The cells:
- IMSQRT, 14 cells (probes/t3643-imsqrt-grid A5, A15 and the 1E-300, 1E-20, 1E-8 imaginary columns B, C, D, L, M, N of
  rows 5 and 15): a part of 0.999999999999999 (1-9·2^-53). Excel prints `1` / `+1i`, so its root is at least
  1-4·2^-53, where the engine's `sqrt(sqrt(x²+y²))` gives 1-5·2^-53. Neither `pow(·, 0.25)` nor a correctly rounded
  `exp(0.25·ln ·)` reaches 1-4·2^-53 for the real input; an extended-precision or doubly rounded root chain is the
  first lead (`arithmetic::extended_sqrt`).
  Of these, twelve are visibly off: the engine prints a part as 0.999999999999999 where Excel prints 1
  (t3643-imsqrt-grid B5, B15, C5, C15, D5, D15, L5, L15, M5, M15, N5, N15; owned here, decision 0245).
- IMLOG10, 2 cells (t3643-imlog10-grid I8, S8, `10±1E8i`): imaginary part `atan2/LN_10` prints 0.682188133491472,
  Excel ...473.
- IMSECH, 4 cells (t3643-imsech-grid H5, H15, R5, R15, `±0.999999999999999±10i`): the imaginary part one digit off.
- The T-3643 fresh sweep (probes-numeric/t3643-r4-log-modulus), 11 cells: B29, B52 (IMLOG10's real `log10 |z|`) and
  B56, B99, B101, B132, B144, B167 (IMLOG10's imaginary `atan2/LN_10`), C66, C111, C173 (IMLOG2's imaginary `atan2·K`).

- Gap triage 2026-11-03: all 31 cells are visibly off under decision 0083, not only the twelve named above. Each is a
  text result whose printed part differs from Excel's in its 15th digit or earlier. That is a wrong text, which
  decision 0188 R1(c) counts (`scripts/exit_owners.py --all-off` lists all 31). Under decision 0114 a group with
  visibly-off cells is a campaign task, not `defer: phase-1b`, so the key moved to `campaign: visible`. Excel is not
  the correctly rounded one at the IMSQRT cells: the engine's 1-5·2^-53 is the correctly rounded root of
  1-9·2^-53, and Excel prints a root of at least 1-4·2^-53. So the stage is `visible` (reproduce Excel's chain), not
  `accuracy`.

## Acceptance criteria
- [ ] Each cell matches Excel, or the task records a time-boxed negative fit (decision 0056) naming what was tried.
- [ ] No other IMSQRT, IMLOG10, IMLOG2 or IMSECH cell of the eval suites regresses.
- [ ] `make check` green; reviewers approve.

- [ ] R4 guards for any fitted rounding step (reviewer-excel-fidelity check 7, decision 0188): hold-out counts, a fresh
      sweep after the model is frozen, the Gate A envelope result, a `FITTED(T-3795, <n> cells, <range>)` tag with its
      range, nothing fitted below the reproducibility floor, and no keep-matched branch for the evidenced points.

## Visible-parity stage
Decision 0083: the 31 cells are visibly off (wrong text), so this task is in the campaign at stage `visible`. Its
target is every owned cell printing Excel's text. A time-boxed negative fit (decision 0056) closes the task only with
its follow-up filed.

## Out of scope
- IMSECH's `#NUM!` for |y| ≥ 1E20 (T-3642).

## Outcome
30 of the 31 cells now print Excel's text, with three structural models (decision 0188 R4 kind (c): a
rounding model, an operation order, an algorithm family; no fitted constant, so no `FITTED` tag):
- **IMSQRT** (14 cells): the modulus root is `extended_sqrt(extended_sqrt(x²+y²))`, two x87 roots, each rounded to
  64 bits and then to a double. 1-9·2^-53 squares to 1-18·2^-53; its root is 1-9·2^-53, whose root is the
  extended midpoint 1-4.5·2^-53, rounding to even 1-4·2^-53 (prints 1). This is the lead the task named.
- **IMLOG10 and IMLOG2** (10 + 2 cells): both are IMLN's parts times one double constant (`log10 e`; the
  double one ulp above `log2 e`, as before), with Excel's ATAN2 kernel (`atan(y/x)` ± π, `atan2_or_libm`) as the
  argument in all three logs. It fits all 896 IMLOG10 texts of the suites and all 924 IMLOG2 texts but one.
  Ranked on those cells (imaginary parts): libm `atan2`/`LN_10` 8 misses, libm `atan2`·`log10 e` 3, ATAN2
  kernel/`LN_10` 6, ATAN2 kernel·`log10 e` 0; x87 forms (fpatan, 64-bit constants, one or two roundings) 6+ on
  IMLOG10 and 3+ on IMLOG2; real part `log10 |z|` 2 misses, `ln|z|·log10 e` 0.
- **IMSECH** (4 cells): `t = sinh²x + cos²y` unfused. T-2750's `mul_add` swapped the last digits of
  `IMSECH("±0.999999999999999+10i")`; J34, the cell T-2750 fused for, fits unfused since the later sin, cos and
  sinh kernels. Over all 100 cases with IMSECH or IMCSCH (44,505 cells) only H5, H15, R5 and R15 change.

**Time-boxed negative (decision 0056)**: probes-numeric/t3643-r4-log-modulus C111,
`IMLOG2(COMPLEX(0.190581435720431,0.981671389192299))`, Excel 1.98953694302209i, ours ...210. Excel's
`atan(5.15092871182028)` is one ulp below the correctly rounded 1.3790419226756176 (the true value is 0.37 ulp below
it): Excel's CRT `atan` is not correctly rounded there. Tried: `π/2 - atan(1/q)` and `π/2 - atan(x/y)` for |q| > 1
(fit C111 but miss IMLOG10 I8, S8, B101, B167), fpatan in extended precision with x87 constants (6-12 misses).
The cell stays in the baseline. It is visibly off, so it is owned by follow-up T-3945 (Excel's `atan` last bit).

**R4 guards**
- Hold-out (manual split, `docs/research/t3795/holdout.py`: sha1 of `<case>!<cell>`, first byte % 10 < 3 → hold-out,
  about 30%, margin 5 points; text results carry 15 digits, so "exact" is the 15-digit text; the population
  includes the fresh sweep below). The split was fixed after the candidates above were ranked on all cells (no fit
  lab split exists for these functions yet), so the fresh sweep is the uncontaminated guard.

  | function | split | n | chosen | correctly rounded |
  |---|---|---|---|---|
  | IMLOG10 | hold | 293 | 293 (100.0%) | 245 (83.6%) |
  | IMLOG10 | train | 753 | 753 (100.0%) | 616 (81.8%) |
  | IMLOG2 | hold | 328 | 328 (100.0%) | 233 (71.0%) |
  | IMLOG2 | train | 746 | 745 (99.9%) | 561 (75.2%) |
  | IMSECH | hold | 874 | 873 (99.9%) | 727 (83.2%) |
  | IMSECH | train | 1942 | 1939 (99.8%) | 1626 (83.7%) |
  | IMSQRT | hold | 294 | 294 (100.0%) | 230 (78.2%) |
  | IMSQRT | train | 608 | 608 (100.0%) | 436 (71.7%) |
- Fresh sweep: probes-numeric/t3795-r4-im-last-digits (seed 3795, `docs/research/t3795/gensweep.py`, generated and
  filled from excel-graph after the models froze at 584b6fb63): 580 of 580 cells match Excel's text (150 IMSQRT,
  150 IMLOG10, 150 IMLOG2, 130 IMSECH).
- Gate A envelope: all 580 fresh-sweep cells are within 15 significant digits of Excel, so inside the envelope;
  no point outside.
- Reproducibility floor: each fixed cell is a 15th-digit text difference, a whole printed digit, not a sub-4-ulp
  numeric difference. No keep-matched branch: each model applies to the whole domain.

Eval delta: probes known failures -20 (t3643-imsqrt-grid 14, t3643-imlog10-grid I8 and S8, t3643-imsech-grid H5, H15,
R5, R15); probes-numeric t3643-r4-log-modulus 529 -> 539 of 540 passing (only C111 left); new case
probes-numeric/t3795-r4-im-last-digits 580/580. No regressions (`make eval-baseline`: REGRESSIONS 0).
