---
id: T-4516
title: "Gate A table review follow-ups: annuity defs, order-statistic and GAMMA/PERMUT semantics entries"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Close the minor review findings left over from three merged Gate A table PRs: T-4109 (#3523, `defs/annuity_values.py`),
T-4092 (#3515, order statistics) and T-4106 (#3525, GAMMA/PERMUT tables).

## Context
- T-4109 (`tools/src/rustsheet_tools/accuracy/defs/annuity_values.py`):
  - a. `_annuity` is used outside its module; make it public and update every caller.
  - b. IPMT and PPMT at `nper < per < nper+1`: give `NoValueError` where nothing evidences Excel's value; keep any region
    financial.md evidences a value or a `DomainError`.
  - c. PDURATION where `1+rate == 1` in the double and `fv == pv`: `NoValueError` unless evidence exists.
  - d. Comment `_payment`'s form, citing T-1528 where the code relies on its evidence.
- T-4092 (`docs/excel-semantics/statistics.md`): correct the AGGREGATE function-number label (14 LARGE .. 19
  QUARTILE.EXC); cite PERCENTILE.EXC `s1` cells A68 and A84 in the PERCENTILE.EXC domain / QUARTILE floor entry.
- T-4106: correct the PERMUTATIONA binary-powering entry's "Ours" value from the engine; add GAMMA edge-stratum draws near
  its overflow point (x about 171.62) without regenerating the committed table, only if `rustsheet-accuracy verify`
  still passes.

## Acceptance criteria
- [ ] Each item above done, or skipped with the reason in the Outcome.
- [ ] tools pytest, ruff, mypy green; `rustsheet-eval accuracy` 0 failing, 0 table defects; `rustsheet-accuracy verify`
      0 differences; `make eval-math-check` current; duplication not risen.
- [ ] `make check-fast` green; reviewers approve.

## Out of scope
- Regenerating any committed table or changing oracle values.
- New table functions or new envelope entries.
