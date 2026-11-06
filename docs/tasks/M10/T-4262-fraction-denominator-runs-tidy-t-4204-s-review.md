---
id: T-4262
title: "Fraction denominator runs: tidy T-4204's review notes and probe the open mixed shapes"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4204]
---

## Goal
T-4204's fraction-denominator code, notes and case read accurately, and the two mixed shapes its reviewers left
open are probed.

## Context
- PR #3227 (T-4204) was approved with these MINOR findings (design and excel-fidelity reviewers):
  - `crates/rustsheet-engine/src/formatter/fraction.rs` `fill_left`: the doc comment is badly wrapped, with one
    line far over the width of the others. The final `for (kind, unused) in kinds.iter().zip(unused)` loop shadows
    the `unused` vec; rename the loop variable (e.g. `is_unused`). The "Leading `#`s take no digit" wording holds
    only when the denominator is short: `# ?/#?` on 0.123 is " 8/65" (probes/t4204-fraction-denominator-mixed
    A14). Say instead that a leading `#` reserves no position, while the run's full length still bounds the digits.
    Fix the same wording in docs/excel-semantics/number-formats.md.
  - number-formats.md "Fraction formats": the `# ?/?0?` example claims "12 1/20 " on 12.5, but the unit test and
    the oracle cover that shape only at 12.125 ("12 1/80 "); cite 12.125 or check the claim against the oracle. The
    "all 325 cells pass" count is stale (the section now also cites t3711 rows 5 and 9-19 and t4204's 42 cells).
    The 12.5 list leaves out `# ?/?0#` "12 1/20" (t4204 A3).
  - eval/suites/probes/cases/t4204-fraction-denominator-mixed.case.json `description` still says "the denominator
    digit limit after a ? run is cut at its 0", round 1's disproved hypothesis. Reword it to "the whole run bounds
    the denominator's digits" (a description edit does not change the oracle sidecar's input hash only if the
    builder ignores it; check with `rustsheet-oracle status` and leave it if it would force a refill).
  - Unprobed engine choices: the order among several left-out `#`s (ours: from the left), and a 2-digit
    denominator in a 3-placeholder run with `0` or `#` in the middle (`# ?/0#?`, `# ?/?#0` on 0.123). Add them to
    number-formats.md's Unprobed note.
  - T-4204's task file Landing notes still say `make check-fast` had not run and name an old head. Mark them
    superseded (check-fast passed on 8b0719c, the merged head).

## Acceptance criteria
- [ ] The wording, count, list and rename fixes above are in; `cargo test -p rustsheet_engine fraction` passes.
- [ ] number-formats.md's Unprobed note lists the two open shapes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing the open shapes (an opus or research task if they matter).
