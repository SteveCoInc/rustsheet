---
id: T-3972
title: "Password-protected xlsx (an OLE compound file holding EncryptedPackage): a typed import error and a recorded ruling for poi 58616 and protected_passtika"
milestone: M11
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A password-protected workbook (an OLE compound file holding an ECMA-376 `EncryptedPackage`, not a zip) gets its own
typed import error that names the cause. A decision record says whether RustSheet reads such files, so the two poi
known failures that come from them are an accepted, documented refusal and not an unowned "zip error".

## Context
- Gap triage 2026-11-03 (opus; `make eval` on origin/main 69b7d87b, then `rustsheet-eval fetch --suite poi`). Every
  failing cell of the run has an open owner (`scripts/exit_owners.py --all-off`). The only failures that no open task
  or decision names are three whole-case lines in eval/baseline/poi.txt. This task owns two of them:
  - `F poi/58616.xlsx !InvalidCase`
  - `F poi/protected_passtika.xlsx !InvalidCase`
  `rustsheet-eval show` gives `message: zip error: invalid Zip archive: Could not find EOCD` for both. `file` reports
  `CDFV2 Encrypted`, and both start with the OLE compound-file signature `D0 CF 11 E0 A1 B1 1A E1`. They are Apache
  POI's encrypted-xlsx fixtures (the password of `protected_passtika` is in POI's own tests, not in the file).
- The third unnamed line, `poi/deep-data.xlsx` (the same EOCD message), is a corrupt package, not an encrypted one.
  It has 17 local headers with garbled names (`_rels/.rEls`, binary names from offset 225813) and no central directory.
  T-0489's Context already puts it with "the three zip files with no end-of-central-directory record ... stay out of
  scope". This task only names it in the xlsx-storage.md entry and does not change it.
- Decision 0012 relabelled four poi crash lines as typed errors. It covers the 32,767-unit shared string
  (`poc-shared-strings.xlsx`) but not these files. T-0489 asks whether Excel for the web opens the refused poi files,
  and it leaves these out as "not zip files". So no record yet says why a password-protected workbook is refused.
- Today the error comes from `impl From<ZipError> for XlsxError` (crates/rustsheet-xlsx/src/error.rs), as
  `XlsxError::Zip`. A caller cannot tell "encrypted, ask for a password" from "corrupt".
- Design question for the decision (this is why the task is opus): Excel opens these only after the user types the
  password. RustSheet has no password in its API (SPEC-0026 facade), and formula evaluation does no I/O (directive 5).
  Reading them would need a password argument on the load API plus ECMA-376 Agile/Standard decryption (AES, SHA-512
  key derivation), which is a new dependency and an API change. The likely ruling is "out of scope for v1, refused
  with a typed error". It could also be "deferred to M12 with a spec task". Either way, record it.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) rules on password-protected workbooks. It says either "out of scope, a
      typed refusal" or "supported later", and if later it names the spec task it files. It also says whether the two
      baseline lines stay as `!InvalidCase` or the poi suite excludes the files (an `exclude` entry in
      eval/suites/poi/suite.toml is a gate change and needs this record to approve it; never hand-edit eval/baseline/).
- [ ] The importer checks the first 8 bytes for the OLE compound-file signature before it opens the zip, and returns
      a new typed `XlsxError` variant (thiserror, with a doc comment) whose message says the file is an encrypted or
      legacy binary (OLE) workbook. A unit test in crates/rustsheet-xlsx feeds an 8-byte-signature buffer and asserts
      the variant. A non-zip, non-OLE buffer still gives `XlsxError::Zip`.
- [ ] `rustsheet-eval show poi/58616.xlsx` and `poi/protected_passtika.xlsx` show the new message (needs
      `rustsheet-eval fetch --suite poi`). `make eval-baseline` reports no regression. Any baseline-line change is the
      one the decision approves.
- [ ] docs/excel-semantics/xlsx-storage.md has an entry, "Password-protected and non-zip packages", that names the
      three files and the ruling. Where Excel behavior is cited, status is DEVIATION or VERIFIED. deep-data.xlsx is
      listed as a corrupt package that stays refused.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Decrypting workbooks or adding a password to the load API (file a spec task if the decision chooses that).
- The legacy binary `.xls` format (BIFF8), which shares the OLE signature: the new error only names it.
- Changing what happens to deep-data.xlsx or the other refused poi files (T-0489, decision 0012).
