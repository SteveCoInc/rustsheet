# 0313 — Mark T-3337's df 1 CHISQ.INV.RT and CHIINV cells under decision 0267 item 3

- **Status:** accepted (2026-11-06, T-3337)
- **Context:** decision 0267 item 3 (owner ruling, 2026-11-04), as narrowed by decision 0301 (owner ruling,
  2026-11-06), rules the math the target (decision 0188 R3) on the CHISQ.INV.RT and CHIINV df 1 cells T-3337 lists
  "on each evidenced point where Excel is more than G off the 50-digit math", with Excel kept wherever it is within G.
  Marking a cell in `eval/accuracy/math-over-excel.tsv` is a protected-path change (SPEC-0029, T-3379). 0267's analysis
  named 6 such cells (p 1.5E-16, 3E-16 and 5E-16), counting only the cells visibly off then. T-3337 checked every df 1
  cell of t3336-chisq-df1-right-tail (T-3336 lists all 291) against its committed math sidecar value:
  - 15 cells have Excel 1.1E-3 to 1.1E-2 relative off the math: CHISQ.INV.RT at A202, A217, A220, A223, A226, A229,
    A232 and A253 (p 1.5E-15, 1E-16, 1.5E-16, 2E-16, 3E-16, 5E-16, 7E-16 and 9E-17), and CHIINV at A203, A218, A221,
    A224, A227, A230 and A233. At A202, A217, A223, A232 and A253 Excel equals the engine's old value, the quantile
    of the double 1 - p, which is just as far off; 0267 had not listed them because they were not visibly off.
  - CHIINV had no Gate A definition, so its cells had no math values. T-3337 adds one: CHIINV is CHISQ.INV.RT's
    legacy name (tools/src/rustsheet_tools/accuracy/defs/gamma.py), as TINV is T.INV.2T's, and has no reference
    table. `rustsheet-accuracy math --function CHIINV` then rewrote the math sidecars of the probes and
    probes-numeric cases that call it, adding CHIINV entries only (7 existing sidecars). In m9-legacy-chiinv and
    t3336 every CHIINV cell equals its CHISQ.INV.RT twin in Excel.
  - The fresh sweep t3337-chisq-df1-r3-fresh (52 cells, p 1.2E-16 to 1.2E-12, filled from excel-graph after the
    engine rule was frozen) has 30 more such cells, A1 to A30, with Excel 3.9E-4 to 7.8E-3 off. At A31 to A52 Excel
    is within 2.8E-5 of the math.
  - Not marked: t3336 A250 to A252 (p 6E-17 to 8E-17), where Excel is `#NUM!`. An error never takes a mark (0188 R3),
    and T-3858 matches that `#NUM!`. Every other df 1 cell has Excel within G, so Excel stays the target there
    (directive 1). That includes t3336's Tier A cells at p 1E-5 to 1E-13, where Excel is at most 2.8E-5 off.
- **Decision:** the 45 cells go into `math-over-excel.tsv` one by one, under the existing statistics.md entry
  "Tiny-p quantiles: Excel is grossly wrong, RustSheet returns the math" (`Status: MATH-OVER-EXCEL`), whose Region now
  names them. The engine returns the math within ε there. At df 1, where the quantile of the double 1 - p is more
  than G off the upper-tail quantile of p itself, it returns the upper-tail quantile. On every evidenced point that
  test agrees with "Excel is more than G off". Of the 225 numeric df 1 cells with a math value in t3336 and the fresh
  sweep, the test fires on exactly the 45 where Excel is more than G off. It also fires at t3336 A250 to A252, where
  Excel is `#NUM!` and the engine already returned a number (T-3858's cut). This applies the owner's
  rulings point by point. The fresh-sweep cells are not in the case list decision 0301 names; they are in scope
  because 0267 item 3 rules on each evidenced point of the same functions at df 1 in probes-numeric, and 0301
  removed only Gate A points. T-4555 relies on the same reading. It adds no R3 case, rules no band and widens nothing beyond 0267 item 3 as 0301 left it, so
  no new owner ruling is needed.
- **Consequences:** R1(c) classes the 45 cells `r3_pass`. Eval pass/fail changes as follows: probes-numeric is
  ungated, so no baseline file changes. In t3336, A217, A218 and A253 passed against Excel and now fail against it,
  and A202, A203, A223, A224, A232 and A233 were within 15 digits and now fail too. All of them pass as R3 cells. The
  new CHIINV math values also give the other CHIINV cells of those cases an R1(c) class, where before they counted
  for want of a math value. T-3858 merged first and brought Excel values at p 8.3254E-17 to 1.1E-16 that are 9.4E-4
  to 8.2E-3 off. The engine returns the math there, so its 88 value cells in t3858-chisq-df1-num-cutoff and -fresh
  take marks by the same rule in this PR (T-4555, folded in). The marks become 133 in all.
