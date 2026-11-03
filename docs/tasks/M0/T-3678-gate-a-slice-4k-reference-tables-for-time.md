---
id: T-3678
title: "Gate A slice 4k: reference tables for TIME, HOUR, MINUTE, SECOND and YEARFRAC"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
TIME, HOUR, MINUTE, SECOND and YEARFRAC each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- dates.md, "TIME divides by 86400 first, then drops whole days" and "TIME: an argument past 32767 is `#NUM!`; a
  near-integer hour counts as that integer (T-2267)": TIME's math reads Excel's rule.
- dates.md, "HOUR, MINUTE and SECOND round to the nearest second" and "HOUR, MINUTE and SECOND past 9999-12-31": the
  integer result follows Excel's rounding of the fractional day, so the definition reads that rule (they are not
  `exact`, decision 0228).
- dates.md, "YEARFRAC accepts serial 0", and the basis argument (0-4): YEARFRAC's math per basis is the day-count
  convention; cite the semantics entries for each basis before writing it.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [x] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 5. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Outcome
- **Definitions** (decision 0228):
  - `tools/src/rustsheet_tools/accuracy/defs/clock.py` holds TIME, HOUR, MINUTE and SECOND.
    - TIME reads each argument with the near-integer window and takes the exact `(h*3600+m*60+s)/86400 mod 1`.
    - HOUR, MINUTE and SECOND take Excel's double steps from dates.md: half a second added, then floors. The rule is
      defined on the doubles, as MROUND's step is (decision 0246), so an exact half follows Excel's binary noise.
      The last-date cut and negative serials are `#NUM!`.
  - `defs/yearfrac.py` holds YEARFRAC on all five bases, from dates.md's YEARFRAC entries:
    - serial 0 and the fictitious 1900-02-29;
    - the swap of a reversed pair;
    - basis 1's real-calendar year length.

    Basis 1's within-a-year rule is the engine's ExcelFinancialFunctions port, not a semantics entry. This table is
    its first oracle sweep, and all 200 points match.
  - Unit tests: `tools/tests/test_accuracy_clock_yearfrac.py` (71 cases from dates.md's excel-graph values and the
    new probe).
- **Excel changed. TIME wraps an argument below -32768 by 65536.**
  - TIME/s1 A149 (`TIME(14120,-33313.544720951,-1389.80591442708)`) is a number in Excel; dates.md said `#NUM!`.
  - probes/t3678-time-argument-far-negative (102 cells, excel-graph, 2026-11-03) shows the rule. After the
    near-integer read, a whole value below -32768 wraps by 65536 into -32768..32767, in any position. Values above
    32767 stay `#NUM!`.
  - Its column G repeats the eight cells that were `#NUM!` on 2026-10-21: time-argument-bound A22:A24,
    time-argument-window A73, A75 and A76, and time-argument-negative A65 and A66. All eight are numbers now.
  - Recorded in dates.md (the TIME bound entry, "Update 2026-11-03"). The definition follows current Excel.
  - The engine still gives `#NUM!`. **T-3983** (new, sonnet, functions-date, P1) owns the engine change and the
    refill of the three older probe sidecars. TIME/s1 A149 is listed under T-3983 in `known-failures.tsv`.
- **Tables:** `s1` for each function, filled from excel-graph on 2026-11-03:
  - TIME has 200 points; HOUR, MINUTE and SECOND have 120 each; YEARFRAC has 200.
  - `rustsheet-accuracy verify` finds 0 differences on all five.
  - `rustsheet-eval accuracy`: 13138 points in 90 tables. 0 fail and there are 0 table defects. The one known
    failure added is A149 (T-3983).
  - Missing tables went from 205 to 200 (−5).
- **Excel against the math:** every Excel error is a math domain error, and every math domain error is an Excel error:
  - TIME: 28 `#NUM!`;
  - HOUR: 18 `#NUM!`;
  - MINUTE: 13 `#NUM!`;
  - SECOND: 9 `#NUM!`;
  - YEARFRAC: 5 `#NUM!`.

  The widest gaps are TIME 2.7e-12 relative (Excel divides by 86400 in doubles before it drops whole days) and
  YEARFRAC 2.1e-16. HOUR, MINUTE and SECOND have no gap. No R3 candidate.
- **Math sidecars:** probes-numeric has no cell of these functions, so it is unchanged. In probes, 1270 cells over
  20 sidecars (18 of them new files). 1261 agree with their oracle cell. The other 9:
  - the 8 drift cells above, whose sidecars still hold the 2026-10-21 `#NUM!` (T-3983);
  - date-system-1900-financial-daycount A40, an XIRR cell that disagrees on main too.

  `math --check` is clean for every case of these functions. A full `math --suite probes` regeneration was too slow
  for this session (over 30 minutes in the solver cases), so I wrote these cases' sidecars with `case_math`
  directly. That changed nothing else on main.
- **Tool fix:** the math tool did not model the 1904 date system. It wrote 1900-calendar YEARFRAC and HOUR values
  for 1904 cases, which disagree with Excel. The new `Definition.serials_1900` flag (YEARFRAC, HOUR, MINUTE,
  SECOND) makes `eval_math` skip those cells in a 1904 workbook ("a 1904 workbook").
- Eval delta: no engine change. The new probe case adds the visibly-off cells where Excel wraps and the engine
  gives `#NUM!`. T-3983 owns them; they are recorded through `make eval-baseline`.
