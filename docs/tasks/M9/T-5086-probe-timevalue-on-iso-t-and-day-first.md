---
id: T-5086
title: "Probe TIMEVALUE on ISO T and day-first text (T-4975 made them #VALUE! by inference)"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
TIMEVALUE's results on ISO `T` text (`"2023-01-01T14:30:00"`) and day-first dates are evidenced by an excel-graph probe.

## Context
T-4975 routed TIMEVALUE through the shared date parser, so those shapes are now `#VALUE!`, inferred from VALUE and DATEVALUE (dates.md, UNVERIFIED).

Filed by T-4975's review (PR #4073).

## Acceptance criteria
- [ ] The goal holds, shown by tests or eval cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
