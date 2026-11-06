# 0319 — IMABS, IMARGUMENT and CONVERT leave exempt.tsv for Gate A tables

- **Status:** accepted (2026-11-06, T-4085). Amends `eval/accuracy/exempt.tsv`, a protected path (decision 0199).
- **Context:**
  - Decision 0270 (T-3800) made IMABS, IMARGUMENT and CONVERT `composite`: their arguments are text (a complex number,
    unit names) and the Gate A generator had no text parameter kind. It named T-4084 for the kind and T-4085 for the
    tables, and said removing the three lines needs its own decision record, because the path is protected.
  - T-4084 added the `text` kind (`Param(kind="text")`: a complex number's text without `choices`, a fixed list with).
  - T-4085 adds one cited definition per function (`tools/src/rustsheet_tools/accuracy/defs/complex_modulus.py`,
    `defs/convert.py`) and an oracle-filled `s1` table for each under `eval/accuracy/tables/`.
- **Options considered:**
  - Keep the `composite` lines and leave the functions to the eval suites (gen-m9, probes, probes-numeric). Rejected:
    decision 0270 made them `composite` only because the generator had no text kind, and T-4084 removed that reason.
    The eval suites do not draw seeded strata, so they cannot stand in for a table.
  - Add the tables but keep the lines. Rejected: `rustsheet-eval accuracy` reports an `exempt.tsv` line for a function
    that has a table as a bad exemption ("the function has a table: remove the line"), so Gate A would not hold.
  - Remove the lines without tables. Rejected: `missing tables` would rise by three.
  - Remove the lines in the change that commits the tables (chosen).
- **Decision:** the three `composite` lines (IMABS, IMARGUMENT, CONVERT) leave `exempt.tsv` in the same change that
  commits their tables. Gate A now scores the three functions through their tables, like any function with a table.
- **Consequences:**
  - `missing tables` does not rise: each function moves from an exemption to a table.
  - IMREAL and IMAGINARY stay `composite` (decision 0270): they return a parsed part, not a computed number.
  - The other decision 0270 lines are unchanged.
