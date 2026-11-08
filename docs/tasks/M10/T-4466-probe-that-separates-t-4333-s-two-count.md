---
id: T-4466
title: "Probe that separates T-4333's two count hypotheses: [s] alone and beside .0, .00, .000 and a decimal-string count, on the t4333 ulp-scan tie serials"
milestone: M10
lane: research
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence that decides between the two hypotheses T-4333 left for the elapsed `[s]`/`[m]`/`[h]` count before a
fraction at a tie, scored by docs/research/t4333/count_models.py, so T-4333 (blocked, decision 0056) can be reopened
on a named model or closed with both refuted.

## Context
- Decision 0355 (proposed): this task owns foundational-tier cells, which the tier gate needs 100% right on type, error and text, so it is P1.
- Gap triage 2026-11-06 (`make eval` on origin/main 07a6b88a, all suites fetched). M10's exit row has 79 visibly-off
  probe cells (decision 0188 R1(c)). 62 are T-4389's `.000` cells, and 16 are T-4333's count cells: probes/t4323-fraction-ties
  A90; t4323-fraction-sweep-2 A1, A24, A41, A84; t4323-fraction-sweep-3 A7; t4323-carry-ties A27, A31;
  t4323-carry-sweep A37, A49, A83; t4333-count-ulp-scan D1:D4, D7, E6. T-4333 is blocked after two
  attempts with no gain, and its "Blocked because" asks for a new hypothesis before it reopens. It names two, and a probe
  that tells them apart: "A probe of `[s]` beside `.0`, `.00` and `.000` on the same serials would separate the two."
  No open task holds that probe.
- The two hypotheses (T-4333, docs/research/t4333/README.md "What the scan shows"):
  (1) the count comes from a decimal form of the serial that differs from binary arithmetic only at the double nearest
  the decimal tie (for example a digit string rounded half-up), since every count/fraction disagreement in the ulp
  scan is at that nearest double (D1:D4 rounded up beside ".9", E6 "3733.0" with the fraction carried and the count not);
  (2) the count comes from the `[s]` token's own whole-second rounding, with a carry test against the fraction it
  formats.
  Under (2) the count should change with the number of fraction digits (`.0`, `.00`, `.000` cut the fraction at
  different places) and `[s]` alone should round, not truncate. Under (1) the count should not depend on the fraction
  digits at all.
- Serials: the eight tie rows of docs/research/t4333/ulp_scan.py (`meta7.json`: day, seconds, ulp offset, exponent),
  stepped -3 to +3 ulp, as probes/t4333-count-ulp-scan.
- Engine today: the count is the half-step register `floor(y*86400/unit)` (`FractionalSeconds::new`,
  formatter/elapsed.rs); the clock and fraction digits use T-4332's field-by-field cut. number-formats.md "Fraction
  digits of a second at a tie" records both.
- R4 discipline: write each hypothesis's predicted text for every new cell (count_models.py extended) and commit them
  before the fill.

## Acceptance criteria
- [ ] A probe case `probes/t4466-count-hypotheses` (excel-probe skill, excel-graph, tag M10) gives, for each of the 56
      t4333 ulp-scan serials, `TEXT(x, f)` for f in `[s]`, `[s].0`, `[s].00`, `[s].000`, `[m]:ss.0`, `[m]:ss.000` and
      `[h]:mm:ss.00`, with the predictions of both hypotheses and of the engine committed before the fill.
- [ ] count_models.py scores both hypotheses and the register on the new cells and on the 590 cells it already scores;
      the README's table and number-formats.md "Fraction digits of a second at a tie" record the counts and the verdict.
- [ ] T-4333's "Blocked because" names the hypothesis the probe supports (and the cells), or records both as refuted;
      any new cell the engine gets wrong is listed there, or in a new task when T-4333's model cannot own it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's count (T-4333) or the `.000` fraction digits (T-4389).
