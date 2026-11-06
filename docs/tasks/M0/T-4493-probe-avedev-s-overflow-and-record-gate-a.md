---
id: T-4493
title: "Probe AVEDEV's overflow and record Gate A's zero-math dispersion points"
milestone: M0
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
AVEDEV's `#NUM!` overflow rule, which T-4090's Gate A definition infers but no probe shows, is backed by excel-graph
cells, and statistics.md records the result. The dispersion tables' points where the math is exactly 0 and Excel is
not are listed with the reason they are no R3 candidates.

## Context
- Follow-up of T-4090 (PR #3507), from the MINOR findings of its excel-fidelity review.
- `tools/src/rustsheet_tools/accuracy/defs/dispersion.py` `absolute_deviations_overflow` makes AVEDEV `#NUM!` when the
  sum of the numbers, an absolute deviation or their sum is beyond the double range. This is inferred from the
  engine's two sums (`fn_avedev` in crates/rustsheet-engine/src/functions/statistical/count_and_average.rs) and not
  probed. T-4439 does the same for DEVSQ. `test_dispersion_ii_domain` asserts `AVEDEV(1.7E+308,1.7E+308)` is
  `#NUM!` and `AVEDEV(1E+200,1E+200)` is not.
- Zero-math points: where the math is exactly 0 (equal numbers), Excel's two-pass residual is not 0, and the relative
  error has no bound. Excel matches the documented switch (statistics.md "VAR/STDEV switch between a one-pass and a
  two-pass formula"), and the engine matches Excel.
  - T-4090's tables: STDEV.P/s1 A69 (7.275957614183426E-12) and A78 (2.2737367544323206E-13), and VARP/s1 A78
    (1.97E-31).
  - T-4089's tables: STDEV.S A68, VAR.S A74 and A84, and DEVSQ A80.
  - T-4090's Outcome gave these as relative gaps. It also misquotes two worst gaps: STDEVP is 2.16E-15 (not 2.3E-15)
    and VAR.P is 6.02E-16 (not 6.2E-16).
- AVEDEV_SOURCE cites only statistics.md "Statistical function argument modes". The arithmetic is numbers.md's AVEDEV
  double-rounding entry (T-1665, probes-numeric/double-rounding-avedev-deviations), which should be cited too.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) holds AVEDEV of 1E+200 twice, of 1E+200 and -1E+200, of
      1.7E+308 twice, and of 1E+308, 1E+308, -1E+308. Its oracle sidecar is committed.
- [ ] statistics.md records AVEDEV's overflow rule with those cells. If Excel differs from
      `absolute_deviations_overflow` or from the engine, the definition (a new batch if a committed value changes) and
      an owner task follow.
- [ ] statistics.md, or the Gate A notes, lists the zero-math dispersion points above with the reason they are not
      R3 candidates. `AVEDEV_SOURCE` also cites the numbers.md AVEDEV entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DEVSQ's overflow (T-4439).
