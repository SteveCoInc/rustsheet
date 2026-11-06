---
id: T-4116
title: "Gate A tables, cash-flow arrays: NPV, XNPV, MIRR and FVSCHEDULE"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
NPV, XNPV, MIRR and FVSCHEDULE each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0273 (T-3797) classified these functions as needing a table.
- Date arguments are integer serials. An order constraint (settlement < maturity, issue < settlement < first coupon) is a
  Definition's joint `draw` or a `domain` rule (`accuracy/model.py`), as T-3535's NPER and RATE use (`defs/annuity.py`).
  Basis is an `int` parameter over 0..4. Reuse `defs/yearfrac.py` for the day counts (directive 3).
- Solvers follow the IRR and RATE pattern (T-3535, T-3586): `solver=True`, 300 points, and Excel's root choice cited from
  a semantics entry. A point with several roots is not drawn without one.
- Where Excel's own arithmetic (a rounding step, an operation order) puts it far from the math, list the points in the
  Outcome as R3 candidates for an owner ruling and do not mark them (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- One-row array literals (T-3586) drive the flows and the schedule; XNPV's dates are a second array, as XIRR's are in
  `defs/cashflow.py`: reuse it. Sources (financial.md): "XNPV discounts with Excel's `^`", "XNPV, XIRR and FVSCHEDULE
  beside their whole data", "NPV, IRR and MIRR error order", "NPV's error order", "NPV's discount past the double range",
  "NPV with a rate of -1 or 1E+300 is #DIV/0!", "NPV and FVSCHEDULE arguments", "MIRR's evaluation order", "MIRR error and
  edge cases".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0273.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/cashflow_values.py`: one definition each for NPV, XNPV, MIRR and FVSCHEDULE,
  the exact forms on the double literals with `1 + rate` exact (NPV sum v_i/(1+r)^i; XNPV on whole days over 365;
  MIRR (P/-N)^(1/(n-1)) - 1; FVSCHEDULE principal times prod(1 + s_i)). No solver: none has a root. The flows, schedule
  and XNPV's dates are one-row array literals; the draws reuse IRR's and XIRR's (`_irr_draw`, `_xirr_draw` of
  defs/cashflow.py) with a rate drawn on its own, and the power cuts reuse T-4109's `_log2_growth`. The "four strata
  from one maker plus an edge" dispatch moved from annuity_values.py into `_common.edge_strata` (the PMT and ISPMT
  tables regenerate byte-identical), so no third copy was written.
  - Domain errors, each from a semantics entry: NPV `#DIV/0!` at rate -1 and where (1+rate)^(n-1) leaves the double
    range; XNPV `#NUM!` for a rate at or below 0, different counts, a date that is negative, 0, past 2958465 (a later
    one checked before truncation) or before the first; MIRR `#DIV/0!` for one flow, no negative flow, a reinvest
    rate of -1, a finance rate of -1 with a later negative flow, an error for a power past the double range, and
    `#NUM!` for a negative ratio over three flows. FVSCHEDULE has none.
  - No math value where nothing evidences Excel's result: NPV's power in the subnormal band, XNPV's discount past the
    double range and dates that truncate to 0 or (first) to 2958465, MIRR's powers at the edge of the range or below
    the normal range, a zero later flow at finance -1, a negative ratio over other than three flows, and a FVSCHEDULE
    product past the double range (OPEN, T-4250) or below the normal range.
  - Before generating, the definitions were run over the 3,376 one-call cells of these functions in probes,
    probes-numeric, gen-m9, gen-m2, gen-m10, smoke and fixtures: 1,233 have literal arguments that fit; 81 math domain
    errors are all Excel errors, 1,140 math numbers all Excel numbers, 12 have no value, 0 mismatches. Excel is more
    than 1e-9 off the math only on probes/xirr-exact-root-sign (two-flow XNPVs at an exact root, a few ulps from 0,
    "XNPV discounts with Excel's `^`").
  - Tests in tools/tests/test_accuracy_cashflow_values.py (Microsoft's examples, the semantics entries' Excel values,
    XNPV's date truncation and order, the evidenced domains, the unprobed no-value points, draws).
- Tables `eval/accuracy/tables/{NPV,XNPV,MIRR,FVSCHEDULE}/s1`: 120, 200, 200 and 120 points (640; 26 domain errors:
  NPV 4, XNPV 11, MIRR 11), filled from excel-graph with 0 failures. Every Excel error matches a coded domain.
- `rustsheet-eval accuracy`: 639 of 640 inside the envelope (636 on both sides, 3 through Excel's 15 digits only:
  NPV A64, MIRR A190 and A199, cancelling sums or a MIRR near 0, 2.8E-6 at most), 0 failing, 1 known failure added,
  `missing tables` 102 to 98. The engine is bit-exact with Excel on 639 of 640: all 614 numeric points and 25 of the 26
  errors.
  - Known failure accuracy/MIRR/s1 A125, owner T-4537 (filed here, its scope is exactly this rule):
    `MIRR({-3496.38522311601,97.9347197399849,-81.6919783865016,180.891974550287,218.629532704649},1E+200,0.00166838654471219)`
    is `#DIV/0!` in Excel and `#NUM!` in the engine.
- New quirk, recorded in financial.md, "MIRR's finance power overflowing under a later negative flow is #DIV/0!
  (T-4116)": with a positive flow, the finance overflow is `#DIV/0!` beside a later negative flow (A125), not the
  `#NUM!` of fin-mirr-edges A15 and A24, whose only negative flow is the first. What exactly separates the two is left
  for T-4537 to probe.
- **Excel kept as the target (decisions 0300, 0301):** no point of the four tables is more than 1e-4 relative off the
  math.
- Math sidecars: `rustsheet-accuracy math --function` over the four functions, for probes and probes-numeric: 13 new
  sidecars (539 cells, 62 of them math domain errors) and 606 cells added to 14 existing ones, with no existing cell
  changed or removed. `make eval-math-check` prints "current".
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
