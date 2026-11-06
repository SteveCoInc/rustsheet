---
id: T-2244
title: "Port ironcalc 4e6db1e6: malformed ZIP input stays a zip error; take zip 9.0.5"
milestone: M5
lane: quality
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Malformed ZIP input (empty bytes, non-ZIP bytes, a truncated `PK\x03\x04` header) passed to
`load_from_xlsx_bytes` is pinned by a unit test to come back as `XlsxError::Zip(_)`, and the
workspace takes the patched `zip` 9.0.5 that upstream moved to.

## Context
Upstream IronCalc abd55b64 ("migrate XLSX ZIP dependency to patched zip 9") moved `xlsx/` from zip
0.6.6 to 9.0.5, and 4e6db1e6 added the test `malformed_zip_inputs_remain_zip_errors` in
`xlsx/src/import/mod.rs`. We are already on zip 9 (9.0.3 in `crates/rustsheet-xlsx`,
`crates/rustsheet` and `crates/rustsheet-eval`, all with the zlib-rs deflate backend), so the migration
itself does not apply; the test and the patch bump do. Our import path goes through
`import/package.rs` (zip-bomb limits); the fuzz target `xlsx_import` covers no-panic but not the
error kind. Triaged by the 2026-10-21 steward run.

## Acceptance criteria
- [x] A unit test in `crates/rustsheet-xlsx` loads `b""`, `b"not a ZIP archive"` and `b"PK\x03\x04"`
  through `load_from_xlsx_bytes` and asserts `XlsxError::Zip(_)` for each (adapt to our signature).
- [x] `zip` is 9.0.5 or later in all three crates and `Cargo.lock`; `cargo deny` passes.
- [x] Commit carries `Ported-From: ironcalc@4e6db1e6`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Upstream 179d0221 (license files): our fork carries its own licensing.

## Outcome
- New test `tests/malformed_zip.rs` pins `b""`, `b"not a ZIP archive"` and `b"PK\x03\x04"` to
  `XlsxError::Zip(_)`.
- `zip` 9.0.3 → 9.0.5 in the three crates and `Cargo.lock` (pinned with `cargo update --precise`;
  9.4.1 would also pull typed-path 0.13 and getrandom's wasm deps). No eval change.
