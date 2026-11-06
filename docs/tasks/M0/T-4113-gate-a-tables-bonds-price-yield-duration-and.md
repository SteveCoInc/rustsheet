---
id: T-4113
title: "Gate A tables, bonds: PRICE, YIELD, DURATION and MDURATION"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
PRICE, YIELD, DURATION and MDURATION each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- YIELD is a solver over PRICE. Sources (financial.md): "PRICE and ODDFPRICE discount with Excel's `^`", "PRICE,
  DURATION and MDURATION sum order at a huge coupon rate and yield", "PRICE, YIELD, DURATION and MDURATION check order",
  "PRICE, DURATION and MDURATION at a huge yield or coupon", "YIELD with one coupon period left is closed form", "YIELD
  stops short of PRICE's root", "YIELD's #NUM! cells and its 100-step cap", "Yield and odd-coupon work cap", "Coupon dates
  keep the maturity's day". T-3722, T-3723 and T-3620 own engine cells of DURATION, PRICE and YIELD.

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [ ] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0273.
