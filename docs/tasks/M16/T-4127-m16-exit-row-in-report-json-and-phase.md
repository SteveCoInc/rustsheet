---
id: T-4127
title: "M16 exit row in report.json and Phase 1 exit script"
milestone: M16
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4126]
---

## Goal
report.json `milestone_exit` has an M16 row measuring the M16 exit criteria (docs/milestones.md), and scripts/phase1_exit.py includes M16 in PHASE1_MILESTONES.

## Context
Decision 0274 puts M16 in Phase 1. Per stage: every in-scope catalog row with >= 30 oracle cells; pivots, gen-pivot and fresh pivot-corpus cells not visibly off; remaining rows owned by a task or out of scope. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Row computed in crates/rustsheet-eval/src/milestone_exit.rs with tests.
- [x] phase1_exit.py and its tests updated (protected path: cites decision 0274).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing other milestones' rows.

## Outcome
- report.json `milestone_exit` has an M16 row (crates/rustsheet-eval/src/milestone_exit.rs `criteria("M16")`, catalog measures in `milestone_exit/pivot.rs`, tests in `milestone_exit/pivot/tests.rs`). Criteria:
  - gen-pivot, pivot-corpus-lo and pivot-corpus-closedxml: at least one scored cell, none visibly off, no case that did not run; an absent suite is `n/a`, never met.
  - the M16 probe row (the `probes` cases tagged M16): no visibly-off cell but R3 passes and M14-owned ones (decision 0083's rule, not decision 0188 R1(c), so `probe_r1c_off` is null and `scripts/exit_owners.py` lists every counted visibly-off probe cell).
  - per catalog stage: every in-scope row Excel for the web can render has at least 30 oracle cells (`evidence_met`); rows with `web` = `no` are counted apart and do not block; the owners of the remaining rows (`owners_met`) are not measured while any remains (follow-up T-4952).
  - layouts: no sidecar pivot missing or misplaced; not measured while the engine emits no `rowItems`/`colItems`.
  - pivot-corpus fresh and stale pivot counts, shown only.
- `scripts/phase1_exit.py` lists M16 in `PHASE1_MILESTONES` (decision 0274, noted in its Consequences) and its M16 line names each measure's verdict.
- Plan §6 check (b), re-rendering fresh corpus pivots against Excel's saved cells, is **not measured**: freshness is
  decided by the engine's own render, and the corpus suite columns score import and formula cells (check (a)) only.
  `corpus_render_met` stays `None`, so M16 cannot read met until T-4906 decides freshness without the engine.
- Probe visibly-off cells classified R3 pass or M14-owned are left out of the probe measure (milestones.md, decision
  0274 item 5) and still reported (`probe_off`, `probe_left_out`). Pivot cells of probe cases the probe-milestone
  rules assign elsewhere (t2085-getpivotdata-*, M6; gen-m9-17c110a53924beb8-refill-1106, M9) are judged in those rows.
- At f48c8fcdd plus this change (make eval and make phase1-exit, 2026-11-07) the row is **not met**:
  `gen-pivot not met; pivot-corpus-lo met; pivot-corpus-closedxml not met; probes not met; evidence not met (PV1
  139/373, PV2 207/211, PV3 3/127, PV5 0/114 at the floor); owners n/a; layout not met; corpus render n/a`.
  - gen-pivot: 25,941 scored, 3,125 visibly off (87.95%), owned by T-4807 and T-4914..T-4927;
  - pivot-corpus-lo met (2,515 scored, 0 visibly off); pivot-corpus-closedxml not met (1,591 scored, 0 visibly off,
    1 case did not run: `TryToLoad/SheetsWithoutRelId.xlsx`, T-4905);
  - probes not met: 25 visibly off counted, 0 left out;
  - owners not measured while rows remain (T-4952); layout not met: 16 pivots whose location differs from Excel's,
    632 with lines not emitted, 0 missing; corpus render not measured (T-4906); pivot-corpus 48 fresh, 77 stale.
