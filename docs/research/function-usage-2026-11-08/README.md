# Function usage in real-world workbooks (2026-11-08)

This is the evidence for the usage tiers in decision 0355 (foundational and expert functions).

## Sources

- **Catalog priors.** The `prior` field in eval/catalog/functions.yaml gives each function's relative frequency in
  real workbooks.
  - For the most-used functions, it comes from published corpus studies (docs/architecture/synthetic-generator.md
    §1.1): Enron, EUSES and FUSE.
  - For the rest, it was set by hand relative to those. Modern functions get about 0.3× their legacy family.
- **SpreadsheetBench count.** `count_usage.py` counts the functions in SpreadsheetBench's real-world workbooks. This
  is the corpus that corpus-bench samples: the tarball and sha256 are in eval/suites/corpus-bench/suite.toml,
  `[fetch]`. The count covers every `*_input.xlsx`:
  - 2,726 files, of which 1,136 contain formulas;
  - 113 distinct functions are called.

  The corpus comes from forum questions about data wrangling, so it under-counts finance and statistics. That is
  why the tier rule also uses the priors.

## Files

- `usage.tsv`: one row for each of the 506 in-scope functions (Gate B's list at main b84a7256), with these columns:
  - the catalog prior;
  - the number of SpreadsheetBench workbooks that call the function;
  - the number of formula cells that call it (a shared formula counts once per row of its range);
  - the tier under decision 0355's rule.
- `count_usage.py`: the counter. To reproduce:
  1. fetch the tarball and check its sha256;
  2. extract it;
  3. run `python3 -I count_usage.py <dir>`.

## Findings

- **Most used.** IF is called in 33.7% of the workbooks with formulas, SUM in 28.2%, INDEX in 13.1%, IFERROR in
  11.5%, MATCH in 11.0% and VLOOKUP in 10.2%. Then come COUNTIF, AND, SUMPRODUCT, ROW, TODAY, TEXT, MAX, SUMIFS and
  LEFT, each in 5% to 8.4%.
- **Not used at all.** These functions appear in none of the 1,136 workbooks:
  - the financial solvers IRR, XIRR, RATE and NPER;
  - the cash-flow functions NPV, PMT, FV and PV;
  - the distribution functions and their inverses (BETA.*, F.*, T.*, CHISQ.*, NORM.* and GAMMA.*);
  - FORECAST.ETS and LINEST.

  This is where nearly all the hard parity work is.
- **Modern functions.** UNIQUE is in 25 workbooks, XLOOKUP in 18, SORT in 10, FILTER in 8, TEXTJOIN in 6, SEQUENCE in
  6 and LET in 5.
