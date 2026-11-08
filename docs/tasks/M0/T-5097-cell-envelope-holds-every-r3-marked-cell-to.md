---
id: T-5097
title: "cell_envelope holds every R3-marked cell to the math, not only those the engine already passes (mark_r3_math sets Target::Math only within ε)"
milestone: M0
lane: harness
status: todo
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
- [ ] Every R3-marked cell with a math value is classed against the math only by `cell_envelope`, inside or not;
  the closeness buckets' `r3_math` count (decision 0310) keeps counting only the ones the engine passes.
- [ ] Unit tests: an R3 cell at Excel's value but off the math is `Outside`; one within ε of the math is
  `Inside`; an unmarked cell is unchanged.
- [ ] The `make eval-baseline` run after the change is committed (tool output only), with any newly outside
  R3 cell listed in the Outcome.
- [ ] Decision 0355 item 2 records the owner's 4-ulp ruling (2026-11-08, PR #4093): a value within 4 ulp of Excel's
  is inside on Excel's side, which keeps Gate A's earlier `NumericUlp` handling. SPEC-0037 design 2 says the 4-ulp
  band is wider than 12 digits only at an Excel 0 or a deep subnormal (below about 1e-311), not at an Excel 0 only.
  (PR #4093 review, Excel-fidelity MINOR.)
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which cells R3 marks (decision 0188, the R3 list).
