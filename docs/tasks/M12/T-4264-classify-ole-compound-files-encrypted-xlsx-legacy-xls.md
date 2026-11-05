---
id: T-4264
title: "Classify OLE compound files: encrypted xlsx, legacy .xls and other get their own typed errors (cfb crate, fuzz target; SPEC-0031 D1)"
milestone: M12
lane: api
status: in-progress
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
- [ ] `EncryptionInfo` + `EncryptedPackage` give an encrypted-package error, and `Workbook`/`Book` gives
      `LegacyBinaryWorkbook`. Anything else stays `OleCompoundFile`. Tests cover the two poi fixtures and a minimal
      `.xls` container built in the test.
- [ ] Malformed containers (a directory cycle, a stream past the bounds, an input over `max_uncompressed_bytes`) give
      errors, not panics. A `cfb_container` fuzz target exists and runs in the fuzz job. A panic or unbounded allocation
      it finds in `cfb` that cannot be fixed upstream triggers SPEC-0031's fallback (a minimal reader of our own).
- [ ] `cfb` is pinned exactly, checked against RustSec, and `cargo deny check` passes; the PR reports `cargo tree`'s
      transitive count.
- [ ] xlsx-storage.md updated, and the poi baseline lines keep `!InvalidCase` with the new message.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Decryption (T-4265) and the API (T-4266).
