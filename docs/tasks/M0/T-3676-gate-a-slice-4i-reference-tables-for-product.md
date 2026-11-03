---
id: T-3676
title: "Gate A slice 4i: reference tables for PRODUCT, SUMSQ, SERIESSUM and PERCENTOF"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-3551, T-3549, T-3586]
---

## Goal
PRODUCT, SUMSQ, SERIESSUM and PERCENTOF each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- PRODUCT and SUMSQ are variadic: use `Definition.variadic` (T-3549). numbers.md "Overflow in `+ - * /` is #NUM! where it happens" (its aggregate-overflow
  paragraph, T-0192): PRODUCT and SUMSQ overflow at an intermediate point like SUM (`PRODUCT(1E+200,1E+200)` stays `#NUM!` with a later factor of 0), so
  the domain follows the running product or sum, as T-3549's SUM does (decision 0224).
- SERIESSUM(x, n, m, coefficients) takes an array literal of coefficients (T-3586's array kind); numbers.md "QUOTIENT,
  FACTDOUBLE and SERIESSUM argument modes".
- PERCENTOF(subset, all) is SUM(subset)/SUM(all) over two arrays; probes/percentof-snap-overflow-union (T-0416) holds
  Excel's evidence on whether its sums snap and overflow: read it before writing the definition.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [ ] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [ ] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 4. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Outcome
- Definitions (decision 0228), each citing numbers.md:
  - `defs/product.py`: PRODUCT is the exact product and SUMSQ the exact sum of squares of the literals' doubles, both
    `variadic`. Domain: a running product, square or running sum of squares past the double range (x87 running values
    as SUM's, T-0990), so `PRODUCT(1E+200,1E+200,0)` and `SUMSQ(1E+200)` are domain errors.
  - `defs/seriessum.py`: the exact sum of c_i·x^(n+i·m) over an array literal of coefficients, with numbers.md's x = 0
    rule (`#NUM!` for a power at or below 0), the T-2380 zero term (negative x, non-integer power) and the T-2163
    overflow (power, term or running sum past the double range).
  - `defs/percentof.py`: SUM(subset)/SUM(all) through SUM's own definition (`aggregate.sum_value`, the snap), so a
    snapped or zero total is a domain error (`#DIV/0!`, probes/percentof-snap-overflow-union D2, D10), like an
    overflowing sum (D6) or quotient.
  - Underflow follows numbers.md "Subnormal results become 0" (T-0189, T-1007, T-1057). A subnormal literal reads as
    0. PRODUCT flushes every running product, so `PRODUCT(1E-200,1E-110,1E+300)` is 0. SUMSQ flushes each square
    (`SUMSQ(1E-155,1E-150)` is exactly 1E-300). PERCENTOF flushes its quotient (T-0416 D14), and SERIESSUM its power
    and its term. Round 0 left the flush out. Comparing the new probes math sidecars with their oracle cells showed the
    gap: 8 cells of subnormal-function-results, subnormal-flush-edges and subnormal-sumsq-sumproduct-edges disagreed.
    The s1 draws keep every value normal, so the tables did not change.
  - Unit tests: `tools/tests/test_accuracy_product_seriessum.py`, against the values numbers.md and the T-0416 probe cite.
- Tables: `s1` for each (PRODUCT, SUMSQ and PERCENTOF 120 points, SERIESSUM 200), filled from excel-graph on 2026-11-03.
  `rustsheet-accuracy verify` finds 0 differences on all four. `rustsheet-eval accuracy` reports 11704 points in 81 tables,
  0 failing and 0 table defects. Missing tables went from 213 to 209 (−4). No point fails, so there is no `known-failures.tsv` line.
- Excel against the math: every Excel error is a math domain error and every math domain error an Excel error. That
  covers PRODUCT 7 `#NUM!`, SUMSQ 5 `#NUM!`, SERIESSUM 15 `#NUM!` and PERCENTOF 7 `#DIV/0!`. The widest gaps are
  1.5e-15 (PRODUCT), 7.6e-16 (SUMSQ), 3.3e-14 (SERIESSUM) and 1.1e-13 relative (PERCENTOF, a near-cancelling total).
  None is an R3 candidate. The engine differs from Excel on 2 PRODUCT points and 1 PERCENTOF point, all inside the envelope.
- PERCENTOF's first draw wrote arrays of up to 510 literals, and excel-graph refused the workbook: those formulas were over
  Excel's 8192-character limit. The long-array edge is now 80 + 160 values and the extreme stratum 10 to 60 values per
  array; the committed table is that second draw.
- Eval delta: none (no engine change).

## Landing notes
- Resumed 2026-11-03 11:45 by an opus run (work-loop resume, branch idle 2 h).
- PR #2951 (draft), head = the commit that adds these notes. CI had not run on a finished head, and no reviewer has
  run. The task is still `status: in-progress`.
- In flight when the window closed: `cd tools && uv run rustsheet-accuracy math --suite probes` (a full run takes 20+
  minutes) was regenerating the probes math sidecars with the final definitions (subnormal flush included). The
  sidecars committed on this branch come partly from an earlier run of the pre-flush definitions, so they are not
  trustworthy. Next steps:
  1. Rerun `math --suite probes` and then `math --suite probes-numeric`. The probes-numeric run so far used the
     pre-flush definitions; its committed changes are `double-rounding-ops` and `double-rounding-range-edges`.
  2. Keep only the sidecar changes in cells that call PRODUCT, SUMSQ, SERIESSUM or PERCENTOF. Revert the other
     rewrites: t1538/t1543/t2586/t3350/t3443 (F.DIST, BETA.INV, CHISQ.INV, basym) and irr-guess-edges are solver-load
     or other-definition churn, as T-3675 also found. T-3675's triage script idea works: diff each `*.math.json`
     against HEAD and classify the changed cells by formula.
  3. Compare every new or changed cell for these four functions against its `*.oracle.json`. With the flush, the
     8 subnormal cells (subnormal-function-results A9, A10, A66, A68; subnormal-flush-edges A1, A4;
     subnormal-sumsq-sumproduct-edges A1, A8) should now agree.
  4. `make eval-math-check` and `make check-fast`, then mark done, run the review panel and fill the PR's review table.
- Do not repeat: PERCENTOF's first s1 draw wrote formulas over Excel's 8192-character limit (excel-graph 501
  "unsupportedWorkbook"). The draw caps fix that, and the committed table is the second draw.
