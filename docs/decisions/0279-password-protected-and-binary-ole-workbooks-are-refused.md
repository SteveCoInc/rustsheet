# 0279 — Password-protected and binary OLE workbooks are refused with a typed error

- **Status:** accepted (2026-11-04, T-3972). A scope and scheduling ruling. It authorizes no deviation from Excel
  (decision 0052): the refusal is the current behavior of a known gap that T-4243 plans to close.
- **Context:** Two Apache POI fixtures in the poi suite, `58616.xlsx` and `protected_passtika.xlsx`, are
  password-protected workbooks. Each is an OLE compound file (signature `D0 CF 11 E0 A1 B1 1A E1`, `file`:
  `CDFV2 Encrypted`) that holds an ECMA-376 `EncryptedPackage`, not a zip. The importer reported them as a zip
  error ("Could not find EOCD"), the same message as a corrupt package (`deep-data.xlsx`), and no record said
  why they fail. Reading them needs a password on the load API (SPEC-0026 facade takes none) plus ECMA-376
  Agile/Standard decryption, an API change and a new dependency, so it needs a spec first. A legacy binary
  workbook (`.xls`, BIFF8) is the same kind of container, and Excel opens it with no password, so it is a gap too.
- **Decision:** Until T-4243's spec lands and its tasks implement it, the importer checks the first 8 bytes for the
  OLE compound-file signature before opening the zip and refuses such a file with `XlsxError::OleCompoundFile`,
  which names it as a password-protected or legacy binary workbook. The two poi baseline lines stay as they are
  (`!InvalidCase`, now with the typed message) as known failures owned by T-4243. The files are not excluded from
  the suite.
- **Consequences:** A caller can tell "encrypted or binary, not an xlsx" from "corrupt". The poi baseline does not
  change. T-4243 (spec: a password on the load API, ECMA-376 decryption, and the `.xls` plan) owns closing the gap.
  The xlsx-storage.md entry is a known gap, not a DEVIATION. `deep-data.xlsx` stays a corrupt package refused as a
  zip error (T-0489).
