# rustsheet-xlsx

RustSheet's xlsx importer and exporter. It started as a fork of IronCalc's `ironcalc` crate (MIT).

This crate is internal to the workspace. Applications open and save workbooks through the
[`rustsheet`](../rustsheet) crate (`rustsheet::Workbook::open`, `Workbook::save_xlsx`), which adds
the load limits and typed errors this crate's API lacks.

## Tests

Every Excel-authored workbook in `tests/calc_tests` is checked automatically: before any
`cargo test` run, `build.rs` generates one test per file.

```bash
cargo test -p rustsheet_xlsx
```

The `test` binary (`cargo build --release -p rustsheet_xlsx`, then `target/release/test <file>`)
checks that RustSheet computes the values Excel saved in a given xlsx file.
