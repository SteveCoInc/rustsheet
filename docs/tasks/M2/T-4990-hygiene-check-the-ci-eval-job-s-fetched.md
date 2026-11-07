---
id: T-4990
title: "Hygiene check: the CI eval job's fetched-suite lists match the gated external suites (decision 0344)"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A suite that is fetched (`[fetch]` in its suite.toml) and gated (`gate = true`) cannot be left out of CI's
`eval (fetched gate suites)` job. The job's three lists of those suites cannot drift from the suites themselves.

## Context
- Decision 0344 (T-4726) names the fetched gate suites in three places:
  - the `eval-fetched-gates` Makefile target (`check --suite …`, `roundtrip --check --suite roundtrip-…`);
  - the `actions/cache` `path:` list in `.github/workflows/ci.yml`;
  - the `hashFiles(...)` list of the cache key in `.github/workflows/ci.yml`.
- Today they name poi and corpus-bench, and the suites that are fetched and gated are exactly poi and corpus-bench
  (`eval/suites/*/suite.toml`, `eval/roundtrip/*/suite.toml` through `cases_of`). Nothing checks that they stay
  equal. A newly gated fetched suite, or a renamed one, would drop out of CI's fetched job while the vendored job
  skips it. That is the silent skip SPEC-0035 D2 forbids.
- Found by the design review of #3999 (T-4726), MINOR, filed as a follow-up.
- `scripts/test_check_ci_workflow.py` already checks ci.yml in the hygiene job, so a natural home for the new check
  is beside it.

## Acceptance criteria
- [ ] A hygiene check fails, naming the difference, unless the gated fetched suites (and their gated round-trip
      suites) equal:
  - the suites `eval-fetched-gates` names;
  - the cache `path:` entries;
  - the `hashFiles` entries.
- [ ] Unit tests cover a matching set, a suite missing from each of the three lists, and an extra one.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Deriving the lists automatically (a generated ci.yml).
