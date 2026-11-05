---
id: T-4290
title: "Read Excel's in-window large-shape gamma exponent from ulp runs on exact x/a (after T-4255)"
milestone: M9
lane: research
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-4255]
campaign: visible
---

## Goal
Excel's exponent for the large-shape (a >= 20) gamma prefactor between x/a 0.672 and 1.5 is identified, so
the engine can take it there. If it cannot be, the probes that rule out each family are recorded.

## Context
T-4255 (docs/excel-semantics/statistics.md, the T-4255 paragraph under "Incomplete gamma: accurate kernel";
script docs/research/t4255/score_forms.py) found these facts:
- probes/t4255-gamma-window-exact-ratio uses x/a = k/64, exact. Up to x/a 0.65625 the ratio form, built on
  Excel's own LN(a/x) (column F), is Excel's at shapes 48, 64, 100 and 256. From 0.671875 on it is not.
- Inside the window, Excel's per-unit exponent error (E_excel - E)/a is the same at shapes 64 and 256 (a power
  of two apart) and lies within about +-0.5 ulp of ln(x/a). Shapes 48 and 100 add a product rounding.
- It is not a double logarithm. Excel's LN(x/a) (column E) is the correctly rounded ln(x/a) at every k, but at
  x/a 0.75 the implied value sits 0.25 ulp from ln(0.75) at all four shapes. The correctly rounded ln(0.75)
  sits 0.47 ulp from it.
- No tested form comes closer than chance (0.29 ulp rms for a uniform +-0.5 ulp error). The forms tested:
  ln, log1p, log2 times ln 2, TOMS 654's rlog, Loader's bd0, atanh series, reciprocal-multiply arguments,
  (u e^(1-u))^a, x87 64-bit intermediates.
- At x/a = 1.5 Excel is neither form at shapes 48, 64 and 256 (10 to 36 ulp off); at shape 100 it is the
  ratio form. So the upper end is not at 1.5a everywhere (T-2638 saw the ratio form at 1.5a at shapes 25,
  50 and 500).

The visibly-off cells this task owns are these:
- probes/t4255-gamma-window-exact-ratio A5:D58 (the densities from x/a 0.671875 up) and G15 (the shape-64 cdf at
  x/a 0.828125).
- probes/t3252-gamma-prefactor-holdout-2 A5:A12, A17:A24 and A29:A36; probes/t3252-gamma-prefactor-holdout A5:A14,
  A19:A28 and A33:A42; probes/t3252-gamma-prefactor-window A4:A10, A11:A20, A24:A30 and A31:A40.
- probes/t3252-gamma-prefactor-sweep A9, A19, A20, A23, A53, A54, A55, A57, A58 and A63.

Read docs/research/excel-kernel-playbook.md first. Its suggested next probes:
- 32-double ulp runs in x at fixed shape 64 around x/a 0.75, 0.9 and 1.2. The per-unit error's sawtooth should
  then show which intermediate is rounded (section 2: "long runs of consecutive doubles").
- Epsilon ladders below and above x/a = 2/3 and 3/2, to pin both switch points.
- Density and cdf side by side. The cdf decouples from the density from x/a 0.72 (statistics.md), so the cdf
  may take Temme's expansion there.

## Acceptance criteria
- [ ] A candidate is scored with a hold-out fixed before scoring (decision 0188 R4) and adopted if it is
      Excel's on the hold-out. Otherwise each rejected family goes into statistics.md with its scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Shapes below 20 (T-3219).
- POISSON.DIST's far upper tail (T-4291).
