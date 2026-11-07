---
id: T-4728
title: "Nightly job checks the calipers cell baseline for regressions (SPEC-0035)"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3556, T-4727]
---

## Goal
The nightly job scores calipers cells against eval/baseline/calipers.txt and goes red on a regression, and its
`make eval` report scores every fetched suite.

## Context
- docs/specs/SPEC-0035-fetched-suite-gate.md D3 and D4. Decision 0323's nightly fetches the three suites after `make eval`, so the report skips them,
  and it checks only their round trips.
- Measured: calipers `check` takes 463 s locally, about 9.7 min on the runner; the nightly job's timeout is 120 min.
- T-3556's context cites a stale nightly.yml comment ("downloaded here once their fetchers exist (T-0006)"). It is
  gone on main (T-4632, decision 0323); re-check and remove any leftover.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) amends decision 0323.
- [ ] nightly.yml fetches first, then runs `make eval`, then `rustsheet-eval check --suite calipers --timeout 300
      --regressions-only` (T-4727) through `scripts/eval_guard.sh`, then the existing round-trip check.
- [ ] docs/process/agent-workflow.md's "Nightly eval run red" row covers a calipers cell regression.
- [ ] A `workflow_dispatch` run on the branch shows the calipers summary line and its time (Outcome).
- [ ] No `T-0006` reference is left in .github/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Gating calipers on PRs (SPEC-0035 D4).
