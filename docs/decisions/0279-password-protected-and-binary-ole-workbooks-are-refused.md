# 0279 — Password-protected and binary OLE workbooks are refused with a typed error

- **Status:** accepted (2026-11-04, T-3972)
- **Context:** Two Apache POI fixtures in the poi suite, `58616.xlsx` and `protected_passtika.xlsx`, are
  password-protected workbooks. Each is an OLE compound file (signature `D0 CF 11 E0 A1 B1 1A E1`, `file`:
  `CDFV2 Encrypted`) that holds an ECMA-376 `EncryptedPackage`, not a zip. The importer reported them as a
  zip error ("Could not find EOCD"), the same message as a corrupt package (`deep-data.xlsx`), and no record
  said why they are refused. Excel opens them only after the user types the password. RustSheet's load API
  (SPEC-0026 facade) takes no password. Reading them would need a password on that API plus ECMA-376
  Agile/Standard decryption (AES, SHA-512 key derivation): a new cryptographic dependency and an API
  change, which no current milestone needs. A legacy binary workbook (`.xls`, BIFF8) is the same kind of
  container and is not read either.
- **Decision:** Out of scope for v1. The importer checks the first 8 bytes for the OLE compound-file
  signature before opening the zip and refuses such a file with `XlsxError::OleCompoundFile`, which says
  it is a password-protected or legacy binary workbook. No password argument and no decryption is added.
  The two poi baseline lines stay as they are (`!InvalidCase`, now with the typed message), and the files
  are not excluded from the suite.
- **Consequences:** A caller can tell "encrypted or binary, not an xlsx" from "corrupt". The poi baseline
  does not change. Supporting encrypted workbooks later needs its own spec task (password on the load
  API, decryption dependency review) and a decision that supersedes this one. `deep-data.xlsx` stays a
  corrupt package refused as a zip error (T-0489).
