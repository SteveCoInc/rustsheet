# 0283 — The cfb_container fuzz target joins the nightly fuzz job

- **Status:** accepted (2026-11-05, T-4264)
- **Context:** SPEC-0031 D1 has the importer parse OLE compound files (encrypted xlsx, legacy
  `.xls`) with the `cfb` crate to classify them, and requires a fuzz target, `cfb_container`, run
  in the fuzz job, to show that `cfb` returns errors rather than panicking or allocating without
  bound on our inputs. The job's target list is the matrix in `.github/workflows/fuzz.yml`, a
  protected path. Its seed corpus is written by a Rust binary, like `bitcode_workbook`'s (T-2895),
  so `fuzz/seed_corpus.py`'s runner for such binaries is generalized and its unit test in
  `scripts/test_seed_corpus.py` (also protected) follows the new name.
- **Decision:** the fuzz matrix gains `cfb_container` (`-max_len=65536`), with the same time,
  timeout, memory and permission settings as the other targets (decision 0035). The target feeds
  `Workbook::from_xlsx_bytes` an input that always starts with the OLE signature, so every run
  reaches the container parser. Its seeds come from `fuzz/seeds/cfb_seeds.rs`: small version 3
  containers laid out like the poi encrypted fixtures, a BIFF8 and a BIFF5 workbook and two
  other OLE files. `seed_corpus.py`'s `bitcode_seeds` becomes `binary_seeds(binary, directory)`
  for both seed binaries; the test checks the same behavior under the new name.
- **Consequences:** one more 20-minute nightly job. A crash it finds in `cfb` that cannot be fixed
  upstream switches T-4264's parser to the minimal reader of SPEC-0031's "Alternatives considered"
  (steward files the task, decision 0035). No PR gate changes.
