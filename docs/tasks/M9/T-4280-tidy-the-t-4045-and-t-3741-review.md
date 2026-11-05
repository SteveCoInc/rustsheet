---
id: T-4280
title: "Tidy the T-4045 and T-3741 review MINORs: ret brackets in bup.rs, ret against ret*c, BINOM.INV near-1 wording"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4045, T-3741]
---

## Goal
The comments and semantics notes that T-4045 (#3242) and T-3741 (#3240) added describe the quantities the code and the
oracle cells measure. Behavior does not change.

## Context
The reviewers approved both PRs, with these wording-only MINORs that came after approval:
- T-4045, `crates/rustsheet-engine/src/functions/statistical/bratio/bup.rs`: the doc comment on
  `l140_bup_prefactor_underflows` gives its brackets on ln(x0^a0·y0^c/B(c, a0)) without the 1/c factor. The predicate
  tests ret = P·x0·y0/c. docs/excel-semantics/statistics.md ("Where the exact 0 starts (T-4045)") already gives ret's
  brackets: (-707.42, -708.42), (-708.38, -709.38) and (-707.59, -708.59). Quote those.
- T-4045, statistics.md, the same item: it says the probe "cannot tell ret from ret·c". The fresh sweep
  probes/t4045-f-dist-rt-zero-cut-fresh settles that. Its row 9 lies within 6E-8 of ln(DBL_MIN) in all six families,
  and Excel's 0-or-number agrees with the sign of ln ret − ln(DBL_MIN) at each one: B9 +3.9E-8 is a number, D9 -4.7E-8
  is 0, F9 +2.7E-8 is a number. With the threshold on ret·c, B9 and F9 would be 0. So the note should say ret (with
  1/c) is the quantity. Check each figure with 60-digit mpmath before writing it.
- T-4045, `bratio/mod.rs` `f_closed_or_ported_tails`: add a one-line comment saying why the zero check may run before
  `closed_form_tails` (the L140BupBgrat branch never overlaps Excel's closed forms; the df1 = 2 zeros stay in the
  closed form).
- T-3741, statistics.md, the BINOM.INV/CRITBINOM near-1 alpha bullet (T-3444 entry):
  - The "4.9E-17 at A155" figure measures the cdf against the decimal literal. Excel compares against the double
    alpha. Measured against it, the gap is 6.8E-17 at accuracy/BINOM.INV/s1 A155 and 2.6E-17 at
    accuracy/CRITBINOM/s1 A171. Recompute both.
  - The bullet's mechanism ("a cdf held as a double near 1 reaches alpha there") does not fit the counts. Rounding the
    exact cdf to the nearest double gives 107488466 at A155 and 1552734933 at A171, but Excel returns 107488465 and
    1552734937. Reword it as "consistent with a cdf carrying about 1E-16 absolute error near 1; the exact form is not
    identified", so T-3447 and T-3740 do not build on a rounding model the data rules out.

## Acceptance criteria
- [ ] The four items above are fixed, with every quoted figure recomputed (50- or 60-digit mpmath).
- [ ] No behavior change: `make eval` deltas are 0, and Gate A is unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine behavior change. BETA.DIST and T.DIST.RT at this edge belong to T-4254. Excel's BINOM.INV cdf belongs to
  T-3447 and T-3740.
