---
id: T-5092
title: "Usage-tier gates: the 12-digit envelope, cell_envelope, R1(c), usage_gates and the Phase 1 exit lines (SPEC-0037 step 2)"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-5091]
spec: docs/specs/SPEC-0037-usage-tier-gates.md
---

## Goal
The eval measures decision 0355's tier gates: the 12-digit envelope everywhere it is used, the `usage_gates` report
section, R1(c) and the milestone rows, and the `foundational` and `expert` lines of `scripts/phase1_exit.py`.

## Context
- SPEC-0037, design sections 2, 3, 4 and 5. The schema and signatures there are fixed, because T-5093, T-5094 and
  T-5095 are built against them in parallel.
- Code: crates/rustsheet-eval/src/accuracy/envelope.rs, accuracy.rs, r1c.rs, probe_milestones.rs,
  milestone_exit.rs, report.rs, attribution.rs, phase1_exceptions.rs; scripts/phase1_exit.py.
- Worked example: report.json of main b84a7256, with the tier counts in decision 0355's Context.

## Acceptance criteria
- [ ] `envelope()` takes its Excel side at `ENVELOPE_DIGITS` (12), the same test `cell_envelope` uses; both are in
  accuracy/envelope.rs since T-5091. `Envelope::Excel15` is renamed, with every reader of accuracy.json updated.
- [ ] R1(c) counts `Outside` and `WrongType` cells only (SPEC-0037 design 2). The milestone rows apply the tier
  bars (design 4).
- [ ] The accuracy check runs at 12 digits. Its stale known-failure lines are removed from the check's own output.
- [ ] report.json has `usage_gates` (design 3) and report.md has its "Usage tiers" section.
- [ ] `scripts/phase1_exit.py` prints `foundational` and `expert` in place of `gate A` and `function rate`, with
  the old two as `info`. scripts/test_phase1_exit.py covers this.
- [ ] Unit tests per SPEC-0037's test plan. A full `make eval` shows the tier numbers in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The CI ratchet (T-5093), the dashboard (T-5094) and task ordering (T-5095).
