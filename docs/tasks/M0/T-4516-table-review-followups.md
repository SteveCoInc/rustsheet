---
id: T-4516
title: "Gate A table review follow-ups: annuity defs, order-statistic and GAMMA/PERMUT semantics entries"
milestone: M0
lane: harness
status: done
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
- [x] Each item above done, or skipped with the reason in the Outcome.
- [x] tools pytest, ruff, mypy green; `rustsheet-eval accuracy` 0 failing, 0 table defects; `rustsheet-accuracy verify`
      0 differences; `make eval-math-check` current; duplication not risen.
- [x] `make check-fast` green
- [ ] reviewers approve.

## Out of scope
- Regenerating any committed table or changing oracle values.
- New table functions or new envelope entries.

## Outcome
- 1a done: `defs/annuity.py`'s `_annuity` is now the public `draw_annuity`; its callers (NPER's and RATE's draws in
  `annuity.py`, `_base` in `annuity_values.py`, and T-4110's `_make` in `annuity_cumulative.py`, which reached into
  the private name) use it.
- 1b skipped (code), documented instead: giving IPMT `NoValueError` at a period strictly between nper and nper + 1 is a
  sidecar removal. `rustsheet-accuracy math --suite probes --function IPMT` then moves
  probes/t4314-ipmt-balance-overflow D1:D2 (`IPMT(1E-300,1,1E-10,...)`, period 1 over nper 1E-10) from math values to
  `skipped`. Excel is `#NUM!` there because of the subnormal `1-Q` (T-4314), which says nothing about the band. So the
  definition keeps the closed form, and a comment on `_ipmt` and financial.md "IPMT and PPMT take a period up to
  nper + 1" record why. PPMT's band stays a value (accuracy/PPMT/s1 A112, A118, now also a unit test). No table point
  is in IPMT's band.
- 1c done: PDURATION where `1+rate` is 1 in the double and fv equals pv is `NoValueError` (0/0, unprobed; financial.md
  "PDURATION's domain" now says so). No table or sidecar cell is affected: regenerating the probes sidecars for
  PDURATION changed nothing.
- 1d done: `_payment` documents its form (the quotient of two present values, finite past P's overflow) and cites
  T-1528 from financial.md "Annuity evaluation order and `(1+r)^n`", which found that Excel's PMT is not the accurate
  value.
- 2 done: statistics.md "PERCENTILE.EXC's k and rank domain" labels the AGGREGATE cells as functions 17 QUARTILE.INC,
  18 PERCENTILE.EXC and 19 QUARTILE.EXC (the cited `#NUM!` and n = 1 cells are all function 18). It also cites
  PERCENTILE.EXC/s1 A68 and A84 (k 0.5 over one value: rank 1 = n is inside, the value).
- 3a done: the PERMUTATIONA entry's "Ours" said 35. The release `rustsheet-eval accuracy` points show 32 of the 105
  numeric cells differ from Excel: 30 inside Excel's 15 digits, plus A58 and A86 that match only the math.
- 3b already on main: GAMMA's edge stratum already draws GAMMA_LAST, the next double up and down, 171.5, 171.6,
  171.7 and 172. Nothing was changed. (`rustsheet-accuracy verify` only recomputes committed formulas, so it would not
  have blocked a draw change.)
- Gates: tools pytest (1884 passed), ruff, ruff format and mypy are green. `rustsheet-eval accuracy` reports 0 failing
  and 0 table defects. `rustsheet-accuracy verify` gives 0 differences for FV, PV, PMT, IPMT, PPMT, ISPMT, PDURATION,
  NPER, RATE, GAMMA and PERMUTATIONA. `make eval-math-check` is current. Duplication is 2876 (baseline 2876).
