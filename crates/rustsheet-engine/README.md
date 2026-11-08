# rustsheet-engine

RustSheet's calculation engine core: the formula parser, evaluator, Excel functions and cell
storage. It started as a fork of IronCalc's `ironcalc_base` (MIT, see `LICENSE` at the repository
root).

This crate is internal to the workspace. Its `Model` API is inherited from the fork and has no
stability promise, load limits or typed errors. Use the [`rustsheet`](../rustsheet) crate
(`rustsheet::Workbook`) instead.

## Tests

```bash
cargo test -p rustsheet_engine
```

Excel compatibility is measured by the eval harness (`make eval`), not by this crate's tests.
