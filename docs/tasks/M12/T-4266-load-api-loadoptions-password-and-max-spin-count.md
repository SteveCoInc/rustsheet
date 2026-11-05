---
id: T-4266
title: "Load API: LoadOptions::password and max_spin_count, typed password errors, Python password= keyword (SPEC-0031 D4)"
milestone: M12
lane: api
status: todo
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
- SPEC-0031 D4: `LoadOptions::password` and `max_spin_count`; `LoadError::{PasswordRequired, WrongPassword,
  Unsupported}`; Python `password=` keyword and `PasswordRequiredError` / `WrongPasswordError` subclasses of
  `LoadError`.
- `LoadOptions` is not `#[non_exhaustive]`: the new fields break struct literals. Note it in the CHANGELOG.
- The password never appears in logs or error text. Code: crates/rustsheet/src/workbook.rs, error.rs,
  bindings/python/src/facade.rs, the `.pyi` stub, bindings/python/README.md.

## Acceptance criteria
- [ ] Facade and Python tests: correct password, wrong password, missing password, `.xls`, and no error message
      containing the password.
- [ ] A `VelvetSweatshop` retry exists only if T-4263 verified E2.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing encrypted files.
