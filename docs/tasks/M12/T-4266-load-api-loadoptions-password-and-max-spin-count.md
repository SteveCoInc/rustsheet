---
id: T-4266
title: "Load API: LoadOptions::password and max_spin_count, typed password errors, Python password= keyword (SPEC-0031 D4)"
milestone: M12
lane: api
status: done
priority: P3
size: S
tier: opus
spec: docs/specs/SPEC-0031-encrypted-xlsx-and-legacy-xls.md
depends_on: [T-4265]
---

## Goal
A caller opens a password-protected xlsx through `Workbook::open` / `from_xlsx_bytes` and the Python bindings, and
gets typed errors for a missing or wrong password and for unsupported files.

## Context
- SPEC-0031 D3 and D4: `LoadOptions::password` (a `Password` newtype) and `max_spin_count`; `LoadError::{PasswordRequired,
  WrongPassword, Unsupported(UnsupportedKind)}`; Python `password=` keyword and `PasswordRequiredError` / `WrongPasswordError` subclasses of
  `LoadError`.
- `LoadOptions` is not `#[non_exhaustive]`: the new fields break struct literals. Note it in the CHANGELOG.
- The password never appears in logs or error text. Code: crates/rustsheet/src/workbook.rs, error.rs,
  bindings/python/src/facade.rs, the `.pyi` stub, bindings/python/README.md.

## Acceptance criteria
- [x] Facade and Python tests: correct password, wrong password, missing password, `.xls`, and no error message
      containing the password.
- [x] `Password` is a zeroizing newtype with a redacting `Debug`; a test asserts `format!("{opts:?}")` of a
      `LoadOptions` with a password does not contain it.
- [x] `max_spin_count` defaults to 1,000,000, at most one key derivation runs per load, and the refusal of
      spinCount 1,000,001 to 10,000,000 is a `DEVIATION` entry in xlsx-storage.md (decision 0014).
- [x] A `VelvetSweatshop` retry exists only if T-4263 verified E2.
- [x] `make check` green; reviewers approve.

## Out of scope
- Writing encrypted files.

## Outcome
- `rustsheet::Password`: a newtype over `Zeroizing<String>` (zeroed on drop), `Debug` prints `Password(***)`, no
  `Display`/`Serialize`. `LoadOptions` gains `password: Option<Password>` (default `None`) and `max_spin_count: u32`
  (default 1,000,000, passed to `ImportLimits::max_spin_count`).
- `Workbook::from_xlsx_bytes` (and so `open`): with a password, an OLE file goes through T-4265's
  `decrypt_ole_package` (only that password is tried: at most one key derivation, the spin-count check before any
  hashing); the decrypted zip (held in a `Zeroizing` buffer) loads as usual. Without one, the importer's refusal
  maps to `LoadError::PasswordRequired`. New `import_load_error` maps `XlsxError` to the new `LoadError` variants
  `PasswordRequired`, `WrongPassword`, `Unsupported(UnsupportedKind::{LegacyBinaryWorkbook, Encryption(kind),
  OtherOleFile})` (`UnsupportedKind` is `#[non_exhaustive]`); limits stay `Options`, malformed or tampered packages
  `Format`. A decrypted package that is itself an OLE file is `Format`. A password on a plain xlsx is ignored.
  `save_xlsx`'s doc says the output is never encrypted.
- No `VelvetSweatshop` retry: T-4263 (E2) is not done.
- Python: `password=` on `Workbook.open`, `Workbook.from_bytes` and `rustsheet.open`; new `PasswordRequiredError`
  and `WrongPasswordError` (subclasses of `LoadError`), exported, in the `.pyi` stub and the README (which says the
  Python `str` cannot be zeroized). The load still runs with the GIL released.
- `LoadOptions` is not `#[non_exhaustive]`: the break is recorded in the new crates/rustsheet/CHANGELOG.md (no
  changelog existed before).
- Tests: crates/rustsheet/tests/encrypted_load.rs (Agile and Standard with the right password, from bytes and from a
  file; missing and wrong passwords, never echoed in `Display`/`Debug`; `.xls` and another OLE file as
  `Unsupported` with and without a password; `max_spin_count` 999 refuses and 1,000 opens the 1,000-round fixture;
  an oversized encrypted file is `Options`; `format!("{opts:?}")` hides the password) and four Python tests. New
  fixture crates/rustsheet/tests/fixtures/biff_container.xls (2,560 bytes: a version 3 container with a 16-byte
  `Workbook` stream, written by `cfb` 0.16.0, shared by the Rust and Python tests). `cfb` (already locked) is a
  dev-dependency of the facade, `zeroize` (already locked, same pin) a normal one: no new crate in Cargo.lock.
- docs/excel-semantics/xlsx-storage.md: the resource-limit DEVIATION names `LoadOptions::max_spin_count` and the
  one-derivation rule; the encrypted-package entry describes the load API and its errors.
- Eval delta: none (the eval runner does not pass passwords until T-4267; every suite unchanged).
