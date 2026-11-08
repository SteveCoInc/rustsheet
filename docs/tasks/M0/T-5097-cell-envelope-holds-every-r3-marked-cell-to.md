---
id: T-5097
title: "cell_envelope holds every R3-marked cell to the math, not only those the engine already passes (mark_r3_math sets Target::Math only within ε)"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Every cell decision 0188 R3 marks is judged against the math by `cell_envelope`, whether or not the engine is
already inside the envelope there, so an R3 cell that leaves the math counts as outside and the T-5093 ratchet
blocks it.

## Context
- Found by the Excel-fidelity reviewer of T-5093 (PR #4091).
- crates/rustsheet-eval/src/r1c.rs: `mark_r3_math` sets `Target::Math` only where `r3_math` holds, and
  `r3_math` requires `envelope(.., math, ..).is_inside()`: the engine already within ε of the math. An R3-marked
  cell where the engine is off the math keeps `Target::Excel`.
- crates/rustsheet-eval/src/accuracy/envelope.rs `cell_envelope` lets a `Target::Excel` cell in when it is
  within 12 digits of Excel. So an R3 cell that moves from the math toward Excel's (wrong, decision 0188 R3)
  value stays `Inside`, and one that is off both is judged against Excel's side as well, where R3 says only the
  math counts.
- Decision 0355 item 2 and SPEC-0037 design 2: an R3 cell is held to the math only.
- The visible-parity lock, the cell baselines' `E`/`W` lines and the usage tiers (T-5093) all classify with
  `cell_envelope`, so the fix reaches each of them; the next `make eval-baseline` records any R3 cell that turns
  out to be outside (a regression needing a decision record if one appears in an unchanged case).

## Acceptance criteria
- [x] Every R3-marked cell with a math value is classed against the math only by `cell_envelope`, inside or not;
  the closeness buckets' `r3_math` count (decision 0310) keeps counting only the ones the engine passes.
- [x] Unit tests: an R3 cell at Excel's value but off the math is `Outside`; one within ε of the math is
  `Inside`; an unmarked cell is unchanged.
- [x] The `make eval-baseline` run after the change is committed (tool output only), with any newly outside
  R3 cell listed in the Outcome.
- [x] Decision 0355 item 2 records the owner's 4-ulp ruling (2026-11-08, PR #4093): a value within 4 ulp of Excel's
  is inside on Excel's side, which keeps Gate A's earlier `NumericUlp` handling. SPEC-0037 design 2 says the 4-ulp
  band is wider than 12 digits only at an Excel 0 or a deep subnormal (below about 1e-311), not at an Excel 0 only.
  (PR #4093 review, Excel-fidelity MINOR.)
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which cells R3 marks (decision 0188, the R3 list).

## Outcome
- `runner::Target` gains `MathMissed`: a cell decision 0188 R3 marks with its evidence (Excel more than G = 1e-4 off
  the math) where the engine is not within ε of the math. `r1c::r3_marked` is the new test (marked, two numbers, a
  math value, Excel more than G off it); `r3_math` is `r3_marked` plus the engine inside on the math, as before.
  `mark_r3_math` sets `Math`, then `MathMissed`, else `Excel`.
- `cell_envelope` takes Excel's side only for `Target::Excel` cells, so every R3-marked cell is held to the math
  alone, passing or not: an R3 cell that moves to Excel's value is now `Outside`, and R1(c), the `E`/`W` ratchet,
  the visible lock and the tier gates all see it. The closeness buckets read `Target::Math` only, so the
  `r3_math` count (decision 0310) is unchanged: a `MathMissed` cell stays visibly off.
- Tests: envelope.rs `cell_envelope_holds_an_r3_cell_the_engine_misses_to_the_math_only`; r1c
  `a_marked_cell_at_excels_value_off_the_math_counts` (Phase 1b before, Counts now) and the updated
  `a_marked_cell_off_the_math_stays_visibly_off` (`MathMissed`, r3_math 0, visibly off 1). `cargo test -p
  rustsheet_eval`: 504 unit + 4 integration pass.
- `make eval-baseline`: no line changed, no regression. On the full report of main e53d43fa (vendored suites,
  2026-11-08), all 457 R3-marked cells it lists are on the math (`Math`); none would be `MathMissed`, so no cell
  moves today. The change guards against a future one.
- Decision 0355 item 2 now records the 4-ulp ruling and the R3 rule; SPEC-0037 design 2 says the 4-ulp band is
  wider than 12 digits only at an Excel 0 or a deep subnormal, and names `MathMissed`.
- Eval delta: none.
