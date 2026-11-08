---
id: T-5089
title: "Rework the T-4975 kernels that miss T-4974's validation probes (131 cells) and decision 0351's accepted regressions"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The T-4975 kernels pass main's T-4974 validation probes they now miss (131 cells) and decision 0351's accepted regressions shrink.

## Context
Decision 0351 lists them: probes/t4974-* (beta-inv-as109-accuracy-exponent 42, beta-inv-large-shape-round-trip 28, f-inv-back-transform 17, f-inv-as109-tolerance-cap 8, chisq-inv-unit-half-boundary 9, gamma-split-complement-region 5, t-dist-df1-and-large-df-centre 5, lognorm-density-holdout 4, others), the LINEST/GROWTH QR cells (14 at 260–1022 ulp in t4590/t4765), and the AS 109 centre cells such as `T.INV(0.2,5)`. Also the CHISQ right-tail P search above 0.9999 keeps Newton to hold one gated cell (a keep-matched branch to remove).

Filed by T-4975's review (PR #4073).

## Acceptance criteria
- [x] The goal holds, shown by tests or eval cells (partly: see Outcome; what is left is owned below).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
One PR, stacked on T-4975's (#4073), applies the 2026-11-08 review of main's parity fixes
(`docs/research/main-implementation-review-2026-11-08.md`, PR #4087) to T-4975's kernels, measured on the
T-4974 probes and every suite, and scored under decision 0355's envelope (12 digits of Excel or 1e-9 of the
math). Against T-4975's head, per family:
- **BETA.INV / F.INV (AS 109, `inverse_beta/as109.rs`, `beta.rs`, `fisher.rs`):** the accuracy floor of 1E-300
  (#4087 point 4, endpoint saturation: `BETA.INV(1E-300,0.5,0.5)` is the 1E-6 seed divided by 3 for 303 steps);
  a floor stop is Excel's value below the root, and above it only where F.INV's root underflows; an
  unconverged deep-tail search is `#NUM!` (point 7: `F.INV(1E-300,11,3)`); a slope stop at the first estimate;
  the crossing where AS 109's first estimate overflows. Gated: 19 wrong-type cells fixed, 28 moved inside the
  envelope, 10 to bit-exact; nothing worse in any class. Main's crossing by default (#4087's advice) was
  measured and loses 540 gated cells for 84, so it is not adopted.
- **LINEST/LOGEST/GROWTH/TREND (`linest.rs`):** R's diagonal is alpha and data-column reflectors divide the
  scale (`2·dot/|v|²`), except an identity fit keeps the reflected head so it stays exact. +165/−5 bit-exact
  (the 5 are 1-ulp TREND/GROWTH cells), 1 wrong type and 8 outside cells fixed; two Gate A lines stale.
  Excel's no-constant slope is not Σxy/Σx² (t4222 A51).
- **CHISQ.INV / CHIINV (`incomplete_gamma/closed_form.rs`):** Halley steps above the Q-to-P switch too, removing
  the Newton branch kept for one cell (`CHIINV(1-1E-10,7)`, now 1 ulp off). +90/−23 gated, last digits only.
- **T.DIST large df:** `n > SERIES_MAX_DF` was already right; the switch back to the kernel is bracketed in
  (0.95, 0.999999] but no fitted cut is shipped (no hold-out); df 999/1000/1001 controls pinned.
- **Measured, not shipped (recorded in docs/excel-semantics/statistics.md):** the gamma small-x region (an
  exponent cap in (2.97, 4.38) cdf / (0.30, 2.29) density, not y ≤ 1; fitted, no hold-out); the Loader store
  chain (no rule separates main's 392 lost probes-numeric cells; #4087 point 10's cells are bd0's v = 0.2 cut,
  T-2586); LOGNORM.DIST's holdout (140/148 as written); NORM.S.INV's centre (interleaved, no region).
- **Left:** the six T-4974 derived cells of decision 0351 that leave the envelope (large-shape round trip
  F1/F6/F12/F38, `E-A` against Excel's exact 0; f-inv-back-transform C6/D6, x/(1-x) of a 1-ulp
  `BETA.INV(0.999,1/2,1/2)`) stay known failures under decision 0351 (owner T-4994); F.INV at 1E-300 with
  df1 6–9 is 11–12 digits off (long floor walk) and huge-df1 F.INV stays far off (T-4994).
- Gate A: 0 failing; three stale lines removed (F.INV.RT A142, LOGEST A98/A110).

