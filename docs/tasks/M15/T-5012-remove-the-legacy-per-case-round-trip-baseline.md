---
id: T-5012
title: "Remove the legacy per-case round-trip baseline path once T-4961 has landed (T-4961 review follow-up)"
milestone: M15
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4961]
---

## Goal
The round-trip ratchet has one baseline form (`<case>#<scenario>` keys, decision 0345) and one judging rule; the
legacy per-case form and its rule are gone.

## Context
- T-4961 (PR #4007) keyed round-trip baselines by case and scenario and kept the per-case path only to judge a
  baseline file still in the old form: `is_legacy` in crates/rustsheet-eval/src/roundtrip/by_scenario.rs, the
  `if is_legacy` branch in `roundtrip.rs`'s command, and `roundtrip::baseline`. All five committed baselines are
  migrated by that PR, so once no branch cut before it re-records the old form, the path is dead outside tests.
- #4007's reviews (design MINOR): `by_scenario::baseline` repeats `roundtrip::baseline`'s structure (mode and None
  handling, the Pass/Fail match, the Cases/Cells branch); the tests in crates/rustsheet-eval/src/roundtrip/tests.rs
  that exercise the legacy builder should move to `by_scenario::baseline`. Security MINOR: `is_legacy` treats any
  key without `#` as legacy, so deleting it also removes that ambiguity.
- docs/process/quality-gates.md's round-trip row still says "a case may not compare less"; the rule is now per case
  and scenario (decision 0345). Changing it is a protected path: cite decision 0345.

## Acceptance criteria
- [ ] `is_legacy`, the legacy branch and `roundtrip::baseline` are removed; one `baseline` builder remains, and the
      legacy tests are re-pointed at it (or deleted where they only tested the legacy form).
- [ ] `rustsheet-eval roundtrip --check` passes on every round-trip suite with the baselines unchanged.
- [ ] quality-gates.md's round-trip row states the per-case-and-scenario rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what the ratchet counts as a regression.
