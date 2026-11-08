---
id: T-4460
title: "Oracle sweep for T-4333: the [s] count at the t4333 tie serials beside no fraction, .0, .00 and .000, scored against its two reopen hypotheses"
milestone: M10
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The evidence decides between the two hypotheses T-4333 recorded when it was time-boxed, or rules both out. Each
hypothesis explains how Excel computes the elapsed `[h]`/`[m]`/`[s]` count before a fraction of a second at a
decimal tie. Then T-4333 either reopens with a named model or stays blocked with both recorded as tested.

## Context
- Decision 0355: this task owns foundational-tier cells, which the tier gate needs 100% right on type, error and text, so it is P1.
- Gap triage 2026-11-06 (`make eval` on origin/main 16a00ebc7). The M10 probe row is "not met" with 79 visibly-off
  cells. Every one is a TEXT cell. 17 belong to T-4333, which is `blocked` (time-boxed, decision 0056) after two
  attempts:
  - probes/t4323-carry-sweep A37, A49, A83;
  - probes/t4323-carry-ties A27, A31;
  - probes/t4323-fraction-sweep-2 A1, A24, A41;
  - probes/t4323-fraction-sweep-3 A7;
  - probes/t4323-fraction-ties A90;
  - probes/t4333-count-ulp-scan D1:D4, D7, E6.

  Example: `TEXT(45000+0.95/86400,"[s].0")` is "3888000001.9" in Excel and "3888000000.9" in the engine. The other 62
  belong to T-4389 (open PR #3440).
- docs/research/t4333 (README, `count_models.py`, `ulp_scan.py`) scored 13 count models on 590 cells. The engine's
  register `floor(y*86400/unit)` is best at 573. Every miss is the double nearest a decimal tie. One ulp or more away
  from that double, count and fraction always agree. T-4333's "Blocked because" names two untested hypotheses:
  1. The count comes from a decimal conversion of the serial that is exact only at the nearest double, for example a
     digit string rounded half-up. The snaps tried so far were 13, 15 and 17 digits, half-even, then exact
     arithmetic.
  2. The count comes from the `[s]` token's own whole-second rounding, with a carry test against the fraction it
     formats. T-4333 notes that a probe of `[s]` beside `.0`, `.00` and `.000` on the same serials would separate
     the two.
- What separates them. Under (2), the bare `[s]` (no fraction; it rounds to the nearest second) and the `.0`, `.00`
  and `.000` forms share one rounding step, so the count flips at the same x for every form. Under (1), the flip
  follows the decimal string, so `.00` and `.000` move it to the nearest double of their own ties (k.995, k.9995).
- Probe design:
  - The eight `ulp_scan.py` seeds stepped -3..+3 ulp, under `[s]`, `[s].0`, `[s].00` and `[s].000` (224 cells; the
    `.0` column re-measures the existing scan).
  - The same seeds at the `.00` and `.000` ties (k.995 and k.9995 seconds) under their own format.
  - An `[m]:ss.0` and an `[h]:mm:ss.0` column at the four round-up seeds, to show whether the larger units follow
    the same rule (the register already gets 285 of 285 `[h]` and 100 of 101 `[m]` counts).
  - Hold-out: a further eight seeds drawn after the models are written, by a fixed seed, and filled last.
  - The cases go under eval/suites/probes.

## Acceptance criteria
- [x] The new case(s) under eval/suites/probes are oracle-filled from excel-graph. A generator script in
      docs/research/t4333 rebuilds them.
- [x] `count_models.py` scores both hypotheses, the register and the README's other candidates on every
      elapsed-count cell with fraction digits, old and new. The README table gives train and hold-out counts side by
      side, with the hold-out seeds fixed before any scoring.
- [x] number-formats.md "Fraction digits of a second at a tie" records what the new cells show, citing them. That
      covers whether the bare `[s]` flips where `[s].0` does, and where `.00` and `.000` flip.
- [x] T-4333's "Blocked because" section names the result. If one model gets every count cell (train and hold-out),
      T-4333 goes back to `todo` with that model named and `depends_on` cleared. Otherwise it stays blocked with both
      hypotheses marked as tested.
- [x] No engine change in this task, and no known-failure line added to eval/baseline outside the new case's own
      lines.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the count model (T-4333). The fraction digits below 0.06 s (T-4389).

## Outcome
Superseded by the t4466 sweeps (probes/t4466-count-hypotheses, t4466-count-fresh), which separate the count models; the adopted rule is T-4333's Outcome.
