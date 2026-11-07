---
id: T-4514
title: "Round-trip R1: the standard edit set, the expected-delta rule, idempotence and per-case exceptions (SPEC-0033 D1, D5, D8)"
milestone: M15
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4509, T-4513]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
Round-trip suites run the R1 scenarios of SPEC-0033 D1 and check the expected-delta rule of D5, so "load, change one
number, save" is proven to change nothing else. The idempotence property and per-case exceptions with their
self-test (D8) land with them.

## Context
SPEC-0033 D1, D5 and D8. The edits E1–E6 apply on the first visible worksheet, through the facade's `edit`/`set`
(crates/rustsheet/src/edit.rs). `Δ(E)` takes its changed-value cells from the engine's two recalculations, not from a
diff. Idempotence means saving our own R0 output again gives an empty `P`. `exceptions.toml` holds narrow tokens,
each naming its decision or task, and a self-test fails on an exception that excuses nothing.

## Acceptance criteria
- [x] The fixtures suite runs R0 and R1/E1–E6 and R1/all.
- [x] A crafted export that also drops an unrelated element is caught as a delta violation (shown at the delta
      rule's level, `r1_rule` in roundtrip/tests.rs; no end-to-end crafted package).
- [x] The idempotence check runs on every case.
- [x] The exception self-test fails on a stale exception.
- [x] The baseline is updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- R2 (M4).
- Excel goldens (T-4180).

## Outcome
- `roundtrip/r1.rs` adds the standard edit set E1–E6 through the facade's `edit`. `Δ(E)` comes from the edits' change
  sets and keeps two cell sets:
  - written cells explain their input, formula, computed flag and value. Their style is explained when the edit
    created the cell (a new `style added` fact) or E6 restyled it;
  - value-changed cells explain their value only.

  Appended number and cell formats and the style counts are explained only under E6. The shared-string counts are
  explained only under E5, and `dimension@ref` only on the edited sheet.
- R0 matching: an R0 item excuses the same R1 item only up to its weight, so a larger `<truncated>` tail stays a
  violation. Validator items also match on their detail.
- Edit targets:
  - E1 and E2 skip spill and array members. The new facade query `Workbook::in_spill` is in `crates/rustsheet/src/spill.rs`.
  - E1 and E2 move on to their next candidate (64 at most) when the engine refuses a write because of its target.
  - Any other edit error is a `scenario error` item, which keeps the case's R0 items.
  - The CLI line counts refused writes and `n/a` edits.
- `roundtrip/scenarios.rs` holds the one table of scenarios: R0, `R0/again` (idempotence), and `R1/E1` … `R1/E6`
  and `R1/all`. `suite.rs` validates manifests against it, the runner dispatches on it, and the rates line groups
  by its families. `KeptRate::of` is `of_scenarios` over every scenario.
- `roundtrip/exceptions.rs`:
  - the `exceptions` manifest key names a file inside the suite;
  - each exception's owner is `T-<n>` or `decision <nnnn>`;
  - a stale exception fails `--check` and `--update-baseline`;
  - excused items are counted in the output.
- Decision 0336: violations ratchet in the baseline until T-4623, T-4831 and T-4832 burn the known ones down. The
  first sheet in tab order stands in for the first visible one until T-4868.
- Scenario sets: the fixtures run all nine scenarios; the corpora run `R0` and `R1/all`. All nine on the corpora took
  22 minutes.
- Per-edit kept rates, from one uncommitted run of every scenario on the corpora (300 s per case). This is a
  different population from the committed baselines: under nine scenarios, Calipers' two 1.4M-formula workbooks
  timed out, so they are missing here. That is why Calipers' E6 (80.18%) is lower than its committed R1 (87.88%).
  The run used round 1's rule; round 2's stricter rule moves R1 by under 0.3 points in the committed baselines.

  | edit | calipers | poi | corpus-bench |
  |---|---|---|---|
  | E1 constant +1 | 99.988% | 99.547% | 99.885% |
  | E2 formula → `=1+1` | 96.47% | 97.73% | 99.45% |
  | E3 empty cell ← 42 | 100% | 99.966% | 99.835% |
  | E4 new row ← 42 | 100% | 99.930% | 99.900% |
  | E5 new text | 99.944% | 98.91% | 99.43% |
  | E6 new number format | 80.18% | 92.79% | 84.92% |
  | R0/again (idempotence) | 99.991% | 99.9997% | 100% |

- E6 dominates the edit losses: a new style still regenerates the whole package (T-4623). E5 regenerates when the
  source has no `sharedStrings.xml` yet (T-4831), and E2 when the formula is in a shared-formula group (T-4832).
- The committed baselines on main's code, with all 541/350/912/29 cases scored:

  | suite | R0 | R1 | refused writes | `n/a` edits |
  |---|---|---|---|---|
  | roundtrip-calipers | 99.99% | 87.88% | 0 | 738 |
  | roundtrip-poi | 99.99% | 92.79% | 5 | 583 |
  | roundtrip-corpus-bench | 100.00% | 84.72% | 7 | 980 |
  | roundtrip-fixtures | 100.00% (R0/again 100%) | 89.92% | 0 | 74 |

  Most `n/a` edits come from first sheets with no numeric constant, no formula or no empty cell in the used range.
  `--update-baseline` over all four took 9 min 27 s at the nightly's 300 s per case.

