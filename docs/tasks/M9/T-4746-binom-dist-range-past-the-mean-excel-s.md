---
id: T-4746
title: "BINOM.DIST.RANGE past the mean: Excel's exact form, last digits past the upper-tail difference (T-3740 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BINOM.DIST.RANGE past the mean at a large n matches Excel's last digits, not just its first 12, or the Outcome ranks
the forms tried and shows why none reproduces Excel's bits.

## Context
- T-3740 put the engine on the upper-tail difference `P(X > s1 - 1) - P(X > s2)` past the walk's budget, when the
  lower end is past the mean (`binom.rs` `binom_dist_range_of`, `excel_binom_upper_tail`). On the fresh sweep
  probes/t3740-binom-range-past-mean-sweep (excel-graph, 2026-11-07), Excel is within 1E-12 relative of that form on
  all 60 rows past the mean. The form is visibly off on 19 rows, where the cdf difference it replaced was off on 38.
  It is bit-exact on 7 rows, and the cdf difference on 9. See docs/research/t3740 and statistics.md "BINOM.DIST.RANGE
  and BINOM.INV at a large n sit on BINOM.DIST's own cdf".
- Owns the visibly-off cells of probes/t3740-binom-range-past-mean-sweep Range!: A4, A16, A19, A31, A33, A34, A35, A38, A45, A46, A48, A50, A51, A52, A54, A57, A58, A59, A60, A61, A64, A70, B9, B31, B33, B35, B38, B51, B61, B64, B70. Column B, and A61-A70 (the
  controls below the mean, which equal Excel's own cdf difference), carry BINOM.DIST's own last digits, which T-3447
  also works on.
- Owns the 21 rows T-3740 inherited from T-3444: probes/t3444-binom-range-inv-large-n Range! A6, A12, A18, A24, A30,
  A36, A42, A48, A54, A60, A66, A72, A78, A84, A90, A96, A102, A108, A114, A120, A126 (ranges at mean + 3..5 sd).
- Candidates (from T-3740): the tails' other bratio branch (w1 complement), a pmf sum over the range from the mode
  side, a switch other than the mean (the mode, cdf(s1 - 1) > 0.5), and how the walk behaves at n = 1E4 to 1E5. On 9
  rows near the mean the cdf difference lands nearer Excel's bits than the upper-tail difference does.
- A form or switch chosen by scoring oracle cells owes decision 0188 R4: a hold-out, a fresh sweep drawn after the
  freeze, and a `FITTED` tag where a value is scored.

## Acceptance criteria
- [ ] A candidate beats the upper-tail form on the owned cells and on a fresh sweep drawn after its freeze, with no
      Gate A point leaving the envelope; or the Outcome ranks the candidates and says why none does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST's own last digits away from BINOM.DIST.RANGE (T-3447).
