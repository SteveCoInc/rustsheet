---
id: T-4665
title: "T-2808 review follow-ups: name t1543 A91/A92 as exceptions to BETA.INV's underflow rule; re-point stale T-2808 cross-references"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2808]
---

## Goal
The docs around BETA.INV's underflow rule (T-2808, PR #3715) state its scope exactly and point to the current owners.

## Context
- Excel-fidelity review of T-2808: the doc comment on `lower_quantile_underflows`
  (crates/rustsheet-engine/src/functions/statistical/beta.rs) and statistics.md's "Underflow test (T-2808)" / "Ours:"
  sentences say Excel is `#NUM!` wherever the quantile underflows for p up to 1/2. Two oracle cells contradict that as a
  general statement: probes-numeric/t1543-inverse-cdf-tiny-p A91 and A92 (`BETA.INV(1E-300,0.5,100)`,
  `BETA.INV(1E-200,0.5,100)`) are 2.7050408442680296E-151 in Excel (a saturation; the true quantile is about 7.9E-603).
  The engine was and is `#NUM!` there; statistics.md (the entry near line 5740 and line 7713) already records them as
  not reproduced. Name them as known exceptions in the comment and the paragraph, with a cross-reference.
- Design review of T-2808: statistics.md (near lines 4755 and 6600) and older task files (T-3862, T-3233 and others)
  still name T-2808 as owner of F.INV.RT's far tail and BETAINV's 0.5; point them to T-4662 and T-4663.
- Docs and comments only; no engine behaviour change.

## Acceptance criteria
- [ ] The comment and statistics.md name t1543 A91/A92 as exceptions to the underflow rule.
- [ ] No doc or open task names T-2808 as owner of the F.INV.RT far tail or BETAINV's 0.5.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the saturation at A91/A92 (T-4663's family; phase 1b).
