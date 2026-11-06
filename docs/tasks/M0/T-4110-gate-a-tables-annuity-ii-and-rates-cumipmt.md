---
id: T-4110
title: "Gate A tables, annuity II and rates: CUMIPMT, CUMPRINC, RRI, EFFECT, NOMINAL, DOLLARDE and DOLLARFR"
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
CUMIPMT, CUMPRINC, RRI, EFFECT, NOMINAL, DOLLARDE and DOLLARFR each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Reuse `defs/annuity.py` for CUMIPMT and CUMPRINC. Sources (financial.md): "CUMIPMT and CUMPRINC over many periods",
  "CUMIPMT and CUMPRINC work cap", "EFFECT and NOMINAL raise to their power with `^`", "T-1625 probes: VDB, AMORDEGRC,
  FVSCHEDULE, MDURATION, RRI". T-3731 (done) settled CUMIPMT and CUMPRINC at a tiny rate: ulps off, not an R3 case.
- DOLLARDE and DOLLARFR truncate `fraction` and do decimal arithmetic; no semantics entry rules them exact, so they get
  a table (T-3797's default; arrays.md lists their argument lifting).

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [ ] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0273.
