# 0312 — The pivot_import fuzz target joins the nightly fuzz job

- **Status:** accepted (2026-11-06, T-4133)
- **Context:** M16 (decision 0274) reads untrusted pivot cache and pivot table parts
  (`crates/rustsheet-xlsx/src/import/pivots.rs`) under SPEC-0015's limits, and directive 5 asks for
  no panic and bounded work on them. `xlsx_import` reaches the pivot reader only when a mutation
  keeps the zip intact and the parts well-formed enough to be found, which random byte flips
  inside a deflated stream almost never do. T-4133 asks for a target fed hostile pivot parts
  directly. The job's target list is the matrix in `.github/workflows/fuzz.yml`, a protected path
  (decisions 0283 and 0287 added targets the same way).
- **Decision:** the fuzz matrix gains `pivot_import` (`-max_len=65536`), with the same time,
  timeout, memory and permission settings as the other targets (decision 0035). Its input is the
  three pivot parts (cache definition, cache records, table definition) separated by NUL bytes;
  they replace the first pivot's parts of crates/rustsheet-xlsx/tests/pivot_tables.xlsx, the
  package is rebuilt in memory (stored, through the `zip` crate the importer uses) and loaded
  through `Workbook::from_xlsx_bytes` with the `xlsx_import` limits, then GETPIVOTDATA reads the
  pivot, every used range is read and the workbook is saved. Seeds come from `fuzz/seeds/pivot_seeds.rs`
  (the fixture's two pivots and ten hostile edits), run by `fuzz/seed_corpus.py`.
- **Consequences:** one more 20-minute nightly job; a crash it finds is filed like any fuzz
  finding (decisions 0035, 0302). No PR gate changes. The fuzz crate gains a `zip` dependency
  (the version and features of rustsheet_xlsx's). When the pivot renderer lands (SPEC-0032), the
  target should also render the pivot (a follow-up of the render task, not this decision).
