# 0270 — Gate A classifies the M9 engineering and database functions with no table

- **Status:** accepted (2026-11-04, T-3800). Amends `eval/accuracy/exempt.tsv`, a protected path (decision 0199).
- **Context:**
  - Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds") needs every in-scope numeric catalog function to have a
    reference table or an `exempt.tsv` line. Decision 0228 classified M5, M6 and M7.
  - T-3800 listed 25 M9 engineering and database functions with neither. On main (92a5d396), `rustsheet-eval accuracy`
    printed `missing tables 199`. ERF, ERF.PRECISE, ERFC and ERFC.PRECISE now have tables, so 21 of the 25 remain.
  - `exact` means no floating-point kernel. `composite` means a table of literals cannot drive the function, and the
    reason names the eval suites that do. Counts of calls in the suites on main: every function below is driven by
    gen-m9 (77 to 922 calls) and probes (18 to 4,264), and most also by probes-numeric.
- **Decision:** each of the 21 functions is classified below.
  - **`exact`, 3 functions: BIN2DEC, OCT2DEC, HEX2DEC.** Each turns at most 10 digits of base text into a whole number
    in two's complement, with no rounding (engineering.md "Leading plus in IM*, 11-digit OCT/HEX text and booleans in
    OCT2*/HEX2*"). This is the ARABIC case of decision 0228. DECIMAL, which reads any base and length, stays `composite`.
  - **`composite`, 14 functions:**
    - DAVERAGE, DMAX, DMIN, DPRODUCT, DSTDEV, DSTDEVP, DSUM, DVAR and DVARP. They take a database range and a criteria
      range. Excel for the web refuses a workbook with an array-literal database (criteria.md "D-function front end
      across all twelve functions"), so no literal shape has Excel values. gen-m9 and probes drive them (the database
      sweeps, probes/db-front-end-sweep). DCOUNT and DCOUNTA are already `exact`;
    - IMREAL and IMAGINARY. They return a part parsed from complex text, which numeric literals cannot drive (the VALUE
      case of decision 0228);
    - IMABS, IMARGUMENT and CONVERT. They have real floating-point kernels (engineering.md "IM* reciprocal kernels,
      IMABS and IMARGUMENT", "CONVERT constants and evaluation order"). But their arguments are text (complex numbers,
      unit names), and the generator has no text parameter kind (`accuracy/model.py` `Kind`). As with the matrix
      functions in decision 0228, they are `composite` until T-4084 adds a text kind and T-4085 replaces the three
      lines with tables.
  - **Needs a table, 4 functions: BESSELI, BESSELJ, BESSELK, BESSELY.** These take real x and an integer order, which
    the generator drives. Excel's BESSELJ is about 1e-7 from the math (decision 0188 R3), inside G, so Excel stays the
    target. T-4083 builds the four tables.
- **Consequences:**
  - `exempt.tsv` gains 17 lines (3 `exact`, 14 `composite`). `missing tables` falls from 199 to 182, with no stale or
    bad exemption.
  - T-4083 brings this group to zero missing tables. T-4084 and T-4085 later swap three `composite` lines for tables.
    That swap needs a new decision record, because the path is protected.
  - The same rule as decision 0228 applies: if an `exact` function shows a rounding question, its line comes out and
    a table task replaces it.
