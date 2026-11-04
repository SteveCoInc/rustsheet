---
id: T-3990
title: "Probe ODDFYIELD's #NUM! at extreme prices (T-3031 follow-up)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
financial.md records, on excel-graph evidence, where ODDFYIELD is #NUM! at extreme prices, and the engine matches it.

## Context
Since T-2007 ODDFYIELD's #NUM! comes only from YIELD's Newton leaving ODDFPRICE's domain (a non-finite step; no
bisection fallback), and no oracle cell exercises it (T-3031 context). Write a probe case
(eval/suites/probes/cases/oddfyield-extreme-price.case.json, tag M9): the probes/oddfyield-solver-sweep bonds (short
first period) and the probes/oddf-date-order-domain-sweep long-first bonds at prices 1, 5, 500 and 5000, and a 0%
coupon at prices 1E-3 and 1E-6, every frequency; fill it on excel-graph and compare. Record the rule in financial.md
("ODDFYIELD stops short of ODDFPRICE's root" or a new entry next to "YIELD's #NUM! cells and its 100-step cap").

## Acceptance criteria
- [ ] The probe case is oracle-filled and committed; every visibly-off cell it adds has an owner task.
- [ ] financial.md states where ODDFYIELD is #NUM! with the probe as evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing a mismatch the probe finds beyond a one-line change (file a task).
