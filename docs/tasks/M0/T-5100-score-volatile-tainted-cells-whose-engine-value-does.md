---
id: T-5100
title: "Score volatile-tainted cells whose engine value does not depend on the random draws"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A cell that calls RAND, RANDBETWEEN or RANDARRAY, or reads one, is scored against Excel when the
engine's value for it does not depend on the draws. The eval harness decides this by evaluating
the case under several random sources, not by reading the formula text. A cell whose value
changes with the draws stays skipped as volatile. NOW and TODAY keep today's rule.

## Context
- `crates/rustsheet-eval/src/volatile.rs` skips every formula cell whose text, or a precedent's,
  calls a random function (T-0005), unless the case lists the cell in `invariant` (T-1283). So a
  cell whose answer cannot depend on the draw is never scored unless someone lists it by hand:
  `IF(RAND()>2,1,NA())`, `ISNUMBER(RAND())`, `ROWS(RANDARRAY(3))`, a LAMBDA or wrapper that hands
  a RAND result on to a function that ignores it. A wrong answer there (0 where Excel gives
  `#N/A`) goes unseen. The skip is a guess from the text; the engine can measure it.
- Excel's draws cannot be reproduced: the seed is neither documented nor settable, the
  Mersenne Twister output-to-double mapping and the RANDBETWEEN/RANDARRAY derivations are
  undocumented, the draw order across cells depends on multithreaded recalc, and every recalc
  re-rolls (the oracle's double fill shows this, T-1283). So a draw-dependent value is never
  compared with Excel; only draw-independent ones are.
- The engine draws from `rand::random()` (thread RNG, unseeded), so no two harness runs agree.
  The workbook clock (SPEC-0008) is the model for a per-workbook source that the harness fixes.
- Decision 0225 scores RAND/RANDBETWEEN/RANDARRAY on their invariant probe cells for Gate B; this
  task widens which cells are found invariant, so it needs a decision record (0357).

## Acceptance criteria
- [ ] The engine draws RAND, RANDBETWEEN and RANDARRAY from a per-model random source: the system
      source by default (behavior unchanged), a seeded one that is deterministic, or a fixed draw.
      The facade can set it on a workbook. Unit tests for each source.
- [ ] The harness evaluates every case under a fixed seeded source. A case with cells tainted
      only by a random function (not by NOW/TODAY) is evaluated again under alternate sources (other seeds, and
      fixed draws at 0 and just below 1); each such cell whose value is identical in every run is
      scored, every other one is skipped as volatile, as today. Unit tests in the runner for a
      draw-independent cell (scored), a draw-dependent one (skipped), and a cell read through a
      precedent.
- [ ] Cells listed in `invariant` keep working; NOW/TODAY cells keep the clock rule.
- [ ] Eval delta reported; newly scored cells that fail are baselined as new coverage and filed
      as tasks; decision 0357 written; `docs/architecture/eval-harness.md` "Scoring" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Reproducing Excel's draws, or any distribution test of ours (decision 0225).
- Varying the clock to find NOW/TODAY-invariant cells (a follow-up if wanted).
- Changing `rustsheet-oracle fill` to fill every random-tainted cell twice (today it does so only
  for `invariant` cells); a follow-up if Excel-side draw dependence shows up in failures.
