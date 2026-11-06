# xlsx round-trip census: scripts

Prototype tools behind `docs/research/xlsx-round-trip-2026-11-04.md`. These are research
scripts, not harness code. The plan's stage 0 replaces them with a `roundtrip` suite kind in
`rustsheet-eval` (the plan's "Test harness" section).

| File | What it does |
|---|---|
| `rt_main.rs`, `rt_Cargo.toml` | A tiny binary. It loads a workbook with `load_from_xlsx`, optionally sets one far cell, evaluates unless `NOEVAL` is set, then calls `save_to_xlsx`. Build it as its own cargo project with a path dependency on `crates/rustsheet-xlsx`. |
| `rtdiff.py` | The package census. It reports the parts lost per part kind, plus an element/attribute census of workbook, worksheet, styles, sharedStrings and theme parts: a count per element path, keyed by `ext@uri` inside `extLst`, with the losses being paths whose count fell. |
| `semdiff.py` | Compares two `rustsheet-eval snapshot` JSONs: sheet names, then per-cell formula text and stored value. |
| `features.py` | Maps the census to named features, giving the table in `census-2026-11-04.md`. |
| `run.sh` | The driver: `run.sh <list of xlsx paths> <outdir>`. Set `RTENV=NOEVAL=1` for an IO-only run. |

Corpora: poi, calipers and corpus-bench, from `rustsheet-eval fetch`, plus the 292 local
workbooks in `crates/rustsheet-xlsx/tests` and `eval/suites/fixtures`. That is 2,028 files in
all. 29 did not load, all of them hostile or malformed POI files.

Limits of the census:
- An element that is present in the output with different content counts as kept.
- Sheets are pooled per part kind, not matched one to one.
- A count rising in one place and falling in another can hide a loss.

The stage 0 differ fixes all three.
