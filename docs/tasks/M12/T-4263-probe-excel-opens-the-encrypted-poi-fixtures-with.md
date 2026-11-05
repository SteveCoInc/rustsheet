---
id: T-4263
title: "Probe: Excel opens the encrypted poi fixtures with their passwords; VelvetSweatshop, 255-char cap and HMAC mismatch (SPEC-0031 E1-E4)"
milestone: M12
lane: research
status: todo
priority: P3
size: S
tier: opus
spec: docs/specs/SPEC-0031-encrypted-xlsx-and-legacy-xls.md
depends_on: []
---

## Goal
Evidence for SPEC-0031's Excel rules E1-E4: Excel opens poi `58616.xlsx` and `protected_passtika.xlsx` with their
passwords (and what it shows), whether it opens an xlsx encrypted with `VelvetSweatshop` without a prompt, whether a
password over 255 characters is refused or truncated, and whether it opens an Agile file whose HMAC does not match.

## Context
- SPEC-0031 "Excel behavior to match" E1-E4, all UNVERIFIED. docs/excel-semantics/xlsx-storage.md "Password-protected
  and non-zip packages".
- The oracle may not be able to open an encrypted file unattended. If excel-graph cannot, the probe needs desktop
  Excel (excel-probe skill); say which was used. The fixtures for E2-E4 are made with an independent tool
  (msoffcrypto-tool); record the commands.
- The passwords are POI's public test data, not secrets.

## Acceptance criteria
- [ ] Each of E1-E4 is VERIFIED or refuted in xlsx-storage.md, with the evidence cited, and SPEC-0031 is updated to
      match (D4's `VelvetSweatshop` step and D5's HMAC rule follow the result).
- [ ] Each fixture's `EncryptionInfo` version (Agile 4.4 or Standard x.2) is recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing anything (T-4264 to T-4266).
