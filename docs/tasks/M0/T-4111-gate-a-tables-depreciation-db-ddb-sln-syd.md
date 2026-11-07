---
id: T-4111
title: "Gate A tables, depreciation: DB, DDB, SLN, SYD, VDB, AMORDEGRC and AMORLINC"
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
DB, DDB, SLN, SYD, VDB, AMORDEGRC and AMORLINC each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- These follow Excel's own arithmetic: DB's 3-digit rate, VDB's charge order, AMORDEGRC's ROUND-like charge. The math
  reference is the documented algorithm at 50 digits, with the rounding steps applied where Excel applies them. Sources
  (financial.md): "DB and SYD argument domains", "DB and VDB period cap", "AMORLINC and AMORDEGRC over many periods",
  "AMORLINC and AMORDEGRC check order", "AMORDEGRC below period 1", "AMORDEGRC past 2^22 periods", "AMORDEGRC rounds its
  charge like ROUND", "T-1625 probes: VDB, AMORDEGRC, FVSCHEDULE, MDURATION, RRI".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green; reviewers approve. (`make check-fast` green locally, CI runs `make check`; design, security and Excel fidelity approve at 22b1cca9.)

## Out of scope
- The other groups of decision 0273.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/depreciation.py` (SLN, SYD, DB, DDB, VDB) and `defs/amortization.py`
  (AMORLINC, AMORDEGRC): each function's documented algorithm at 50 digits on the double arguments, with Excel's own
  rounding steps applied (DB's rate ROUND(.., 3), AMORDEGRC's ROUND-like charge with its 15-digit cut, AMOR*'s double
  `ceil(1/rate)`). AMOR* draw dates jointly (purchase <= first period) and reuse defs/yearfrac.py's `us_30_360`,
  `eu_30_360`, `displayed_date` and `_serial`; no solver.
  - Domain errors, each from a semantics entry or eval cells: before generating, the definitions ran over every literal
    one-call eval cell of the seven functions: 1,200 values, all within 1e-9 of Excel; 363 domain errors, each an
    Excel error of the same code; 310 without a value. No math domain error lies where Excel has a number, and every
    Excel error in a table falls on a coded domain (0 table defects). The domains no entry stated are now in financial.md "SLN, SYD, DB,
    DDB and VDB argument domains" with their eval and table cells.
  - No math value (`NoValueError`), as nothing evidences Excel's result or Excel's doubles may land either side: AMORDEGRC
    at a period strictly between 0 and 1 (workbook state, "AMORDEGRC below period 1"); a DB period 100,001 to
    2147483647 and a VDB end past 100,000 (the engine's cap DEVIATION); VDB switching with life = start = end; a DB
    life at most 0; AMOR* 30/360 dates below serial 61, AMORLINC below period 1 when the full charge does not fit,
    periods past 1E6, a salvage equal to the cost below period 1; DB's rate within 1e-9 of a rounding half, an AMORDEGRC
    charge within (n + 2)·1e-14 of a half, a remain or an AMORLINC remainder at a whole number of charges, and
    cancelling book-value-less-salvage differences past `CONDITION_LIMIT`.
  - Tests in tools/tests/test_accuracy_depreciation.py (Microsoft's examples, the semantics entries' Excel values,
    the evidenced domains, the two rules below, draws).
- Tables `eval/accuracy/tables/{SLN,SYD,DB,DDB,VDB,AMORLINC,AMORDEGRC}/s1`, 200 points each (1,400), filled from
  excel-graph with 0 failures.
- `rustsheet-eval accuracy`: 1,387 points inside the envelope (all on both sides), 13 known failures added, 0 failing,
  0 table defects. `missing tables` 102 to 95 (on main after T-4109, T-4092 and T-4106). The engine is bit-exact with Excel on 1,364 of 1,400; 23 more are 1-2
  ulp off Excel inside the envelope (DB 12, VDB 9, SYD 1, DDB 1).
- New quirks, recorded in financial.md (the first draft of each definition encoded the old reading, and the oracle
  check flagged the points):
  - "AMORLINC and AMORDEGRC check order": a salvage *equal* to the cost is 0 at periods 1 to 5, not `#NUM!`
    (accuracy/AMORDEGRC/s1 A114, A118, A128, A130, A131; accuracy/AMORLINC/s1 A124, A125). The engine is `#NUM!`:
    known failures owned by new task T-4504.
  - "VDB charge order": where straight line already wins at period 0 (a factor below 1 - salvage/cost), every period
    of the window charges (cost - salvage)/life, a fractional start included (accuracy/VDB/s1 A158, A183, A187, A188,
    A189, A200; up to 10.7% from the fractional-grid model the engine keeps). Known failures owned by new task T-4510.
- **Excel kept as the target (decision 0300):** none. With the two rules above in the definitions, no point is more
  than 1e-4 relative off the math (before them, the six VDB points were, by 5.5E-5 to 0.107).
- Math sidecars: `rustsheet-accuracy math --function` over the seven functions, for probes and probes-numeric, rerun
  after merging origin/main: 121 new probes sidecars and 17 existing ones extended (7,406 lines added, none removed or
  changed; 933 math cells in the 123 regenerated cases); probes-numeric has no such cell. `make eval-math-check` prints
  "current".
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
