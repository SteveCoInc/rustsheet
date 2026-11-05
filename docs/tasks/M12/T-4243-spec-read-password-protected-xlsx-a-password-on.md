---
id: T-4243
title: "Spec: read password-protected xlsx (a password on the load API, ECMA-376 decryption) and decide on legacy .xls"
milestone: M12
lane: api
status: done
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
- [x] A spec covers encrypted xlsx (API, decryption, limits, errors, tests) and states the `.xls` plan.
- [x] Implementation tasks are filed from it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing either reader (the spec's tasks do that).

## Outcome
- docs/specs/SPEC-0031-encrypted-xlsx-and-legacy-xls.md (draft): OLE container classification with the `cfb` crate
  (D1); ECMA-376 Agile and Standard decryption over RustCrypto primitives (D2); limits under directive 5: a
  `max_spin_count` default of 1,000,000, below MS-OFFCRYPTO's 10,000,000 format cap (a `DEVIATION` entry T-4266
  records), and at most one key derivation per load (D3); `LoadOptions::password` as a zeroizing, `Debug`-redacting
  `Password`, typed `LoadError` variants with `UnsupportedKind`, and the Python `password=` keyword (D4); HMAC verification (D5); and the
  `.xls` plan (refuse with a typed error until an owner ruling).
- Tasks filed: T-4263 (probe E1-E4), T-4264 (classification), T-4265 (decryption), T-4266 (load API), T-4267 (poi
  suite passwords) and T-4268 (owner ruling on `.xls`, blocked on the owner). The poi fixtures' known failures are now
  T-4267's, and xlsx-storage.md points at the spec.
- Review round 1 (security, design): the spin-count default, the password's `Debug` exposure, the SPEC-0001/0026
  citations, constant-time comparisons and shape checks were fixed in the spec and the task criteria.
- No probe was run: SPEC-0031's Excel rules are UNVERIFIED until T-4263. Eval delta: none (docs only).
