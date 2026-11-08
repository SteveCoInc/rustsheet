---
id: T-5077
title: "An unchanged pivot over a defined name saves verbatim again (T-4506 regressed calipers pivot_named_range R0)"
milestone: M16
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-4506]
---

## Goal
A workbook whose pivot reads a defined name, loaded and saved with no edit, is written back as its source again, byte
for byte (the Verbatim tier, SPEC-0034 D2).

## Context
Found while re-recording T-4999's baselines after merging main.
- Since T-4506 (#4037, "Refresh pivots over defined names, Tables and shared caches"), calipers
  `roundtrip/pivot_named_range/golden.xlsx` and `init.xlsx` R0 regenerate instead of saving verbatim. R0 kept drops
  from 100% to 87.87%, 429 failing items per case: core and app properties, theme, styles and so on.
- main's own binary reproduces it: `rustsheet-eval roundtrip --filter pivot_named_range`, against
  `eval/baseline/roundtrip-calipers.txt`. That baseline holds no failing line for these cases.
- CI gates only the fixtures suite, so the merge queue let it through. The nightly calipers run would flag it.
- Likely cause: the refresh on load changes the live model (the pivot's cache or records) without any edit. The
  pristine re-import and the live model then differ, so D2 classifies the save as Structural. The pristine side
  re-imports the source the same way, so the difference is in what the refresh writes into the live model, or in a
  refresh that is not idempotent.

## Acceptance criteria
- [ ] The root cause is found and named in the Outcome.
- [ ] Both cases are Verbatim again at R0, and `rustsheet-eval roundtrip --check` passes on calipers with the baseline
      as it is (no rerecord).
- [ ] A regression test: a fixture with a pivot over a defined name saves verbatim when nothing changed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the refresh semantics of T-4506.
