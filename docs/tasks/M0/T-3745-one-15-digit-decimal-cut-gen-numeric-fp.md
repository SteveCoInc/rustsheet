---
id: T-3745
title: "One 15-digit decimal cut: gen numeric_fp _cut calls accuracy literal decimal15"
milestone: M0
lane: quality
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
One definition of "a double's exact decimal cut to 15 significant digits" in tools/.

## Context
- Deferred MINOR finding of the quality review of PR #2705 (T-3627, carrying T-3555): `decimal15` in
  tools/src/rustsheet_tools/accuracy/literal.py is the half-even case of `_cut(x, rounding)` in
  tools/src/rustsheet_tools/gen/numeric_fp.py. CLAUDE.md directive 3 (one definition per concept).
- Give `decimal15` an optional `rounding` argument (default half-even) and have `numeric_fp._cut` call it, or the
  reverse; keep every caller's output unchanged.

## Acceptance criteria
- [ ] One function cuts to 15 digits; the other is gone; `scripts/check_duplication.py` does not rise.
- [ ] Existing tools tests pass unchanged; a unit test covers both rounding modes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any rounding mode.

## Outcome
`literal.decimal15(x, rounding=ROUND_HALF_EVEN)` is the one 15-digit cut; `gen/numeric_fp._cut` and
its unused `SIG_DIGITS` are gone and `_cut_tags`/`_boundary_pairs` call it. Outputs unchanged;
`tests/test_accuracy_literal.py` covers both rounding modes. No eval delta.
