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
- [x] A crafted export that also drops an unrelated element is caught as a delta violation.
- [x] The idempotence check runs on every case.
- [x] The exception self-test fails on a stale exception.
- [x] The baseline is updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- R2 (M4).
- Excel goldens (T-4180).

## Outcome
- `roundtrip/r1.rs` adds the standard edit set E1–E6 through the facade's `edit`, with `Δ(E)` taken from the edits'
  change sets (inputs and changed values) and the delta rule `violations`. A failing item of the edited save is
  allowed only when it is one of these:
  - already failing in R0;
  - a fact about a Δ cell;
  - an appended number format or cell format;
  - a derived count or range (N9, N10).

  Anything else is kept with scenario `R1/<edit>`.
- Implementation choices (also recorded in SPEC-0033 D1, D5 and D8):
  - The edits act on the first sheet in tab order.
  - Each edit is its own transaction. A refused edit (a target inside a spill) counts as `n/a`.
  - Violations ratchet in the baseline rather than failing outright, while E6 forces regeneration (T-4623).
- `R0/again` saves our own R0 output again (idempotence). `roundtrip/exceptions.rs` adds the `exceptions` manifest key,
  and a stale exception fails `--check` and `--update-baseline`.
- `CaseResult::scenarios` keeps each scenario's checked count. The CLI and the report give one rate per family: R0, R1
  and R0/again.
- Scenario sets: the fixtures run all nine scenarios; the corpora run `R0` and `R1/all`. Every scenario reruns
  every case, so all nine on the corpora took 22 minutes.
- Per-edit kept rates, from one run of every scenario on the corpora (300 s per case; Calipers' two heaviest
  workbooks timed out under nine scenarios):

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
  source has no `sharedStrings.xml` yet, and E2 when the formula is in a shared-formula group; these are T-4831 and T-4832.
- The committed baselines on main's code, 541/350/912/29 cases, all scored:

  | suite | R0 | R1 |
  |---|---|---|
  | roundtrip-calipers | 99.99% | 87.88% |
  | roundtrip-poi | 99.99% | 92.79% |
  | roundtrip-corpus-bench | 100.00% | 84.92% |
  | roundtrip-fixtures | 100.00% (R0/again 100%) | 89.93% |

  `--update-baseline` over all four took 8 min 36 s at the nightly's 300 s per case.

