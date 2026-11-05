---
id: T-4264
title: "Classify OLE compound files: encrypted xlsx, legacy .xls and other get their own typed errors (cfb crate, fuzz target; SPEC-0031 D1)"
milestone: M12
lane: api
status: done
priority: P3
size: S
tier: opus
spec: docs/specs/SPEC-0031-encrypted-xlsx-and-legacy-xls.md
depends_on: []
---

## Goal
The importer tells an encrypted xlsx, a legacy binary `.xls` and any other OLE compound file apart, with a typed
error for each, instead of the one `XlsxError::OleCompoundFile` of decision 0279.

## Context
- SPEC-0031 D1: the `cfb` crate over an in-memory cursor, bounded (4,096 directory entries; `EncryptionInfo` at most
  1 MiB; `EncryptedPackage` at most `max_uncompressed_bytes`), and classified by the root entries.
- Code: crates/rustsheet-xlsx/src/import/package.rs (`is_ole_compound_file`, `Package::new`), error.rs, the eval
  snapshot reader (crates/rustsheet-eval/src/xlsx_snapshot.rs), crates/rustsheet/src/workbook.rs (the error mapping).
- A new dependency: cargo-deny must pass, and the security reviewer reviews it.

## Acceptance criteria
- [x] `EncryptionInfo` + `EncryptedPackage` give an encrypted-package error, and `Workbook`/`Book` gives
      `LegacyBinaryWorkbook`. Anything else stays `OleCompoundFile`. Tests cover the two poi fixtures and a minimal
      `.xls` container built in the test.
- [x] Malformed containers (a directory cycle, a stream past the bounds, an input over `max_uncompressed_bytes`) give
      errors, not panics. A `cfb_container` fuzz target exists and runs in the fuzz job. A panic or unbounded allocation
      it finds in `cfb` that cannot be fixed upstream triggers SPEC-0031's fallback (a minimal reader of our own).
- [x] `cfb` is pinned exactly, checked against RustSec, and `cargo deny check` passes; the PR reports `cargo tree`'s
      transitive count.
- [x] xlsx-storage.md updated, and the poi baseline lines keep `!InvalidCase` with the new message.
- [x] `make check` green; reviewers approve.

## Out of scope
- Decryption (T-4265) and the API (T-4266).

## Outcome
- New `import/ole.rs`: `classify_ole_compound_file(bytes, limits)` parses an OLE file with `cfb` (pinned `=0.16.0`)
  over the in-memory bytes, after `Package::new` bounds them by `max_uncompressed_bytes`, and names the refusal from
  the root storage's streams: `EncryptionInfo` + `EncryptedPackage` → new `XlsxError::EncryptedPackage`; `Workbook`
  or `Book` → new `XlsxError::LegacyBinaryWorkbook`; anything else, more than 4,096 directory entries or a malformed
  container → `XlsxError::OleCompoundFile`. An `EncryptedPackage` declaring more than the budget is the
  uncompressed-bytes limit error. No stream is read. The eval snapshot reader uses the same function.
- The facade still maps these to `LoadError::Format` (typed `LoadError` variants are T-4266).
- Tests: crates/rustsheet-xlsx/tests/ole_container.rs (the poi fixtures' exact directory, BIFF8/BIFF5, other and empty
  containers, a nested `Workbook`, the byte budget, a declared size past the file, a directory cycle, a sector-chain
  cycle, the 4,096-entry cap, and every single-byte corruption of two containers: all errors, no panic). The old OLE
  test moved there from malformed_import.rs.
- Fuzz: target `cfb_container` (input always starts with the OLE signature), seeds from `fuzz/seeds/cfb_seeds.rs`,
  added to the nightly matrix (decision 0283). `seed_corpus.py`'s `bitcode_seeds` became `binary_seeds`. cargo-fuzz is
  not installed in the authoring sandbox, so the target was built, not run; the first nightly run is its evidence.
- Found in passing: `cfb` checks a stream's chain against its declared size only when the stream is read, and a
  version 3 file's stream size is 32 bits. Recorded in xlsx-storage.md; T-4265 must treat a short read as malformed.
- Dependencies: `cargo tree` adds 3 crates to the normal build (cfb, fnv, web-time; uuid was already in the lock as
  a dev-dependency). `cargo deny check`: advisories, bans, licenses and sources ok.
- poi (fetched): 58616.xlsx and protected_passtika.xlsx report "Encrypted Workbook Error: the file is a
  password-protected xlsx, which cannot be opened yet"; their baseline lines keep `!InvalidCase`.
- Eval delta: none (poi 115 → 115 known failures, every suite unchanged).
