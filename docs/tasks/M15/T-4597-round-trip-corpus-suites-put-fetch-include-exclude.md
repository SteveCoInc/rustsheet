---
id: T-4597
title: "Round-trip corpus suites: put fetch include/exclude in the cache pin, and a sanctioned way to drop a case"
milestone: M15
lane: harness
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: [T-4173]
---

## Goal
A corpus round-trip suite's cache is refetched whenever its fetch would take other files, and a case can leave a
`cases` baseline through a recorded change instead of a hand edit.

## Context
Found by the T-4173 excel-fidelity review:
- `FetchSpec::pin()` (crates/rustsheet-eval/src/suite.rs) covers url, rev and subdir, not `include`/`exclude`. A
  cache fetched before T-4173 widened the calipers include to `roundtrip/**/init.xlsx` therefore still counts as
  fetched, and lacks those files.
- T-4173 made `roundtrip --update-baseline` refuse to drop cases from a `cases` suite, so that cache cannot silently
  remove the 83 `init.xlsx` cases. With that refusal, an upstream rev bump or a new exclude has no sanctioned way
  to drop a case (directive 2 forbids editing `eval/baseline/` by hand).

## Acceptance criteria
- [ ] The cache marker includes the fetch's include and exclude lists, so a change to them makes the cache stale.
- [ ] A case dropped because the manifest's pin or include changed in the same change is recorded by
      `--update-baseline`; a case dropped while the pin is unchanged is still refused. Unit tests cover both.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New corpora.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
