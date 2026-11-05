---
id: T-4265
title: "Decrypt ECMA-376 Agile and Standard encrypted xlsx packages with bounded limits (SPEC-0031 D2, D3, D5)"
milestone: M12
lane: api
status: todo
priority: P3
size: M
tier: opus
spec: docs/specs/SPEC-0031-encrypted-xlsx-and-legacy-xls.md
depends_on: [T-4264]
---

## Goal
`decrypt_package` turns an ECMA-376 Agile or Standard encrypted package plus its password into the plain zip bytes,
within the limits of SPEC-0031 D3, verifying the Agile HMAC per D5.

## Context
- SPEC-0031 D2, D3, D5 give the algorithms (MS-OFFCRYPTO 2.3.4.7, 2.3.4.11) and the RustCrypto dependencies (`aes`,
  `cbc`, `ecb`, `sha1`, `sha2`, `hmac`, `base64`, `zeroize`). No hand-rolled crypto.
- Before adding them, recheck the crates.io state named in SPEC-0031 "Alternatives considered".
- New module crates/rustsheet-xlsx/src/import/encryption.rs. Depends on T-4264's container reader.

## Acceptance criteria
- [ ] Known-answer tests for both key derivations, and round trips on fixtures made by an independent tool.
- [ ] The malformed inputs listed in SPEC-0031's test plan each give a typed error. An `encrypted_package` fuzz target
      exists.
- [ ] `spinCount` above 10,000,000, or above `max_spin_count` once T-4266 lands, is refused before any hashing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The public API (T-4266) and the poi suite (T-4267).
