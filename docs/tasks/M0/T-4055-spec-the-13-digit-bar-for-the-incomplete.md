---
id: T-4055
title: "Spec: the 13-digit bar for the incomplete-beta family (decision 0267): Phase 1 rate, headline, campaign and exit counts"
milestone: M0
lane: harness
status: todo
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ defines how the eval counts the incomplete-beta family under decision 0267, and the design
reviewer approves it. The family is BETA.DIST, BETADIST, F.DIST, F.DIST.RT, FDIST, BINOM.DIST, BINOMDIST, NEGBINOM.DIST and NEGBINOMDIST. A number attributed to one of these functions that is off Excel by more
than half a unit in the 15th significant digit, but within half a unit in the 13th, becomes Phase 1b residue: it is
reported every run but does not count as visibly off. It is never silently dropped.

## Context
- Decision 0267 (the owner's ruling, 2026-11-04). On a run of probes-numeric at main 985d511b, 86,821 of the 89,166
  visibly-off numbers were in this family. Of those, 84,948 were first wrong only at the 14th or 15th digit. Excel
  was correctly rounded on 1% of a 2,026-cell sample of them, so those digits are Excel's rounding noise.
- "Visibly off" is decision 0083's `Visible::is_off` / `within_fifteen_digits` (crates/rustsheet-eval/src/closeness.rs).
  It feeds:
  - the Phase 1 rate and tiers (v15_checkpoint.rs, attribution.rs);
  - the probe rows and R1(c) (r1c.rs, probe_milestones.rs);
  - Gate B for gated generated suites (gate_b.rs; decision 0211 counts every visibly-off cell);
  - the milestone exits (milestone_exit.rs);
  - scripts/phase1_exit.py, scripts/exit_owners.py and scripts/new_case_owners.py;
  - the progress dashboard (tools/src/rustsheet_tools/progress_report/).
- The spec has to name every one of these consumers and say whether it reads the new bar. Decision 0267 says yes for
  the rate, the tiers, the headline, the campaign counts, the probe rows, Gate B and the milestone exits. It says no
  for the regression gates.

## Acceptance criteria
- [ ] docs/specs/SPEC-XXXX-*.md. It defines the bar:
  - half a unit in the 13th significant digit, the same boundary rule as `within_fifteen_digits` at 13;
  - attribution only when every function a cell's formula calls is in the family or is a routing function;
  - composite cells (for example `BETA.INV(BETA.DIST(..))`) left out unless the spec argues them in.
- [ ] It names every consumer listed in Context, and what each one counts after the change.
- [ ] It keeps these unchanged:
  - the bit-level baselines and `make eval-check`;
  - eval/baseline/visible-parity.txt: a family cell within 15 digits today that moves off 15 is still a regression;
  - the type, error and text verdicts;
  - every function outside the family.
- [ ] It defines the residue the progress report shows: the count of family cells first wrong at the 14th or 15th
      digit, a separate line from the 13-digit headline.
- [ ] The design reviewer approves.

## Out of scope
- The implementation (T-4056).
- Any function outside the family, such as T.DIST, GAMMA.DIST or the quantiles.
