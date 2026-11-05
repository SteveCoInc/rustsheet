---
id: T-4243
title: "Spec: read password-protected xlsx (a password on the load API, ECMA-376 decryption) and decide on legacy .xls"
milestone: M12
lane: api
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-3972]
---

## Goal
A spec in docs/specs/ says how RustSheet will read the workbooks it refuses today with `XlsxError::OleCompoundFile`
(decision 0279), so that known gap has a planned close.

## Context
- T-3972 made the importer refuse an OLE compound file with a typed error. That covers a password-protected xlsx (an
  ECMA-376 `EncryptedPackage`: poi 58616, protected_passtika) and a legacy binary `.xls` (BIFF8). Decision 0279 records
  this as the current behavior of a known gap, not a deviation (decision 0052: only directive 5 can authorize one).
  docs/excel-semantics/xlsx-storage.md "Password-protected and non-zip packages" has the entry.
- Encrypted xlsx: the spec needs a password argument on the SPEC-0026 load API (and in the Python bindings), ECMA-376
  Agile and Standard encryption (AES, SHA-512 or SHA-1 key derivation, spin count bounded under directive 5), a review
  of the crypto dependency (cargo-deny), and the typed error for a wrong password. First confirm on excel-graph or desktop
  Excel that Excel opens the two poi fixtures with their passwords (POI's own tests publish the passtika password; the
  password is not in the file).
- Legacy `.xls`: Excel opens BIFF8 workbooks with no password, so the spec decides whether RustSheet reads them (a BIFF8
  reader is a large importer of its own) and in which milestone. If it does not, that is a scope decision for the owner.

## Acceptance criteria
- [ ] A spec covers encrypted xlsx (API, decryption, limits, errors, tests) and states the `.xls` plan.
- [ ] Implementation tasks are filed from it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing either reader (the spec's tasks do that).
