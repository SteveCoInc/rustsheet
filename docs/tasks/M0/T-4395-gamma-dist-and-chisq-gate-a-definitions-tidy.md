---
id: T-4395
title: "GAMMA.DIST and CHISQ Gate A definitions: tidy the T-3925 review's minor findings"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-3925]
---

## Goal
The T-3925 Gate A definitions (tools/src/rustsheet_tools/accuracy/defs/gamma_dist.py) and their semantics notes carry
no stale or unstated assumption the T-3925 reviewers flagged as MINOR.

## Context
Reviewer findings on PR #3412 (all MINOR, none changes a table or sidecar value today):
- `NORMAL_MIN = sys.float_info.min` is defined in both gamma_dist.py and continuous.py: move it to `defs/_common.py`
  beside `CONDITION_LIMIT` and import it in both (directive 3).
- `_gamma_dist_domain`'s "x / beta overflows" rule applies to every shape, but its evidence is shape 1 only
  (probes/incomplete-gamma-cap E6, E18): say so in the module docstring. statistics.md's T-2738 entry ("GAMMA.DIST's
  density is #NUM! at a subnormal x / beta") still lists "an x / beta that overflows" as unprobed, which line ~1106
  (E6, E18) contradicts: drop that clause.
- CHISQ.DIST's cdf keeps a math value at a subnormal x / 2, where GAMMA.DIST's cdf gets none (Excel reads the
  ratio as 0); the region is unprobed and undrawn. Give it the same no-value rule, or record it as unprobed in the
  docstring.
- The T-2753 entry's new paragraph (T-3925) and T-4392 name an underflow cut of "about 1E-162" for shape 2 that is
  derived, not probed (A15 at 1E-200 is 0, A16 at 1E-150 is a number): mark it UNVERIFIED.
- Optional: one `_scalar` helper for the repeated `assert not isinstance(..., (bool, tuple))` lines.

## Acceptance criteria
- [x] Each item above is done or noted moot in the Outcome; `make eval-math-check` stays current (regenerate the
      stale cases only if a definition change moves a value).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The R3 ruling itself (T-4392).

## Outcome
- `NORMAL_MIN` now lives in `defs/_common.py` beside `CONDITION_LIMIT`; gamma_dist.py and continuous.py import it
  (their `sys` imports are gone).
- gamma_dist.py's docstring says the "x / beta overflows" rule's evidence (probes/incomplete-gamma-cap E6, E18) is
  shape 1 only and is applied at every shape, unprobed at the others. statistics.md's T-2738 entry no longer lists an
  overflowing x / beta as unprobed; it cites E6, E18 at shape 1 and leaves other shapes unprobed.
- CHISQ.DIST's cdf gets GAMMA.DIST's no-value rule at a positive x whose x / 2 is subnormal (one helper,
  `_subnormal_ratio_cdf`, used by both), with a unit test either side of the cut. No table or sidecar point is in that
  region: `verify` gives 0 differences and `make eval-math-check` is current.
- The shape-2 underflow cut "about 1E-162" is marked derived, UNVERIFIED (A15 at 1E-200 is 0, A16 at 1E-150 a number)
  in the T-2753 entry and in T-4392's Context. The docstring's "R3 candidates (T-4392)" now cites the ruling (decision
  0300, Excel kept).
- Optional item done: `_common.scalar` replaces gamma_dist.py's repeated `assert not isinstance(...)` lines.
- Verify (0 differences): GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT, CHIDIST, and every continuous.py function
  (LOGNORM.DIST, LOGNORMDIST, LOGNORM.INV, LOGINV, EXPON.DIST, EXPONDIST, WEIBULL.DIST, WEIBULL, FISHER, FISHERINV).
  Eval delta: none.
