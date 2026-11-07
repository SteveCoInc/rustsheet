---
id: T-4763
title: "LOGEST and GROWTH digits at fits whose exponents near the double range (t4590 probe cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2006]
---

## Goal
LOGEST's b and m and GROWTH's value match Excel's digits on the fits of probes/t4590-growth-overflow, whose exponents
lie near the double range. Today they are 5E-14 to 3E-13 relative off, the least-squares solve's last digits.

## Context
- T-4590 implemented GROWTH's overflow and underflow edges from this probe. Its remaining cells are digits, not
  edges.
- Owned cells: probes/t4590-growth-overflow Int! A1, B1, B2, B4, B5, B6, B7, C4, C6, C7, C10, C11 and Slope! A4,
  A5. For example, Int!A1 `GROWTH({1.00709088702808e+152,1,9.92959039626498e-153},{1,2,3},2,TRUE)` is 1 in Excel
  and 1.0000000000001137 in ours.
- The ln fit's coefficients come from the least-squares kernel that T-2006 owns (Excel's evaluation order for
  LINEST, LOGEST, TREND and GROWTH). This task should follow T-2006 and check whether its kernel clears these cells.
- A form chosen by scoring oracle cells owes the decision 0188 R4 hold-out and fresh sweep.

## Acceptance criteria
- [ ] The owned cells pass, or the Outcome shows that they are what is left after T-2006's kernel and says why.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROWTH's overflow and underflow edges (T-4590).
