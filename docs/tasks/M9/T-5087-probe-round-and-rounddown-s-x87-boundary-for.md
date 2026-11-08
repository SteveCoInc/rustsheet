---
id: T-5087
title: "Probe ROUND and ROUNDDOWN's x87 boundary for 7 to 15 kept digits (T-4975 widened it by inference)"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
ROUND/ROUNDDOWN's x87 conversion boundary for 7 to 15 kept digits is evidenced by a probe.

## Context
T-4975 (audit 6) widened the model from m < 1e6 to 15 significant digits (`X87_DECIMAL_DIGITS`); numbers.md marks 7–15 digits UNVERIFIED.

Also probe SUBTOTAL 6/106's subnormal flush (`SUBTOTAL(6,1E-200,1E-120)`: 0 since T-4975, 1E-320 before;
numbers.md SUBTOTAL entry).

Filed by T-4975's review (PR #4073).

## Acceptance criteria
- [ ] The goal holds, shown by tests or eval cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
