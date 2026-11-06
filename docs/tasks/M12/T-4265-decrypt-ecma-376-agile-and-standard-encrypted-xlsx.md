---
id: T-4265
title: "Decrypt ECMA-376 Agile and Standard encrypted xlsx packages with bounded limits (SPEC-0031 D2, D3, D5)"
milestone: M12
lane: api
status: done
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
- From T-4264's review (PR #3279, MINOR findings deferred here, where the streams are first read):
  - Read `EncryptionInfo` with `take(1 MiB + 1)` (SPEC-0031 D1); T-4264 checks only `EncryptedPackage`'s declared size.
  - `cfb` checks a stream's chain against its declared size only on read, so a short read is malformed, not a
    truncated package (docs/excel-semantics/xlsx-storage.md).
  - Add the OLE bounds (raw container size against `max_uncompressed_bytes`, the 4,096 directory entries, the
    declared `EncryptedPackage` size) to the resource-limits DEVIATION entry of xlsx-storage.md, with import/ole.rs in
    its "Implemented in": once decryption works they change what a caller sees.
  - `MAX_DIRECTORY_ENTRIES`'s doc comment: the cap bounds the walk, not `cfb`'s eager directory parse, which the byte
    budget bounds.

## Acceptance criteria
- [x] Known-answer tests for both key derivations, and round trips on fixtures made by an independent tool.
- [x] The malformed inputs listed in SPEC-0031's test plan each give a typed error. An `encrypted_package` fuzz target
      exists.
- [x] `spinCount` above 10,000,000, or above `max_spin_count` once T-4266 lands, is refused before any hashing.
- [x] The verifier and HMAC comparisons are constant-time, and the shape checks of SPEC-0031 D3 (ciphertext length a
      multiple of 16, declared size past the decrypted length, checked segment arithmetic, XML element count and
      depth, short hash output) each have a malformed-input test.
- [x] `make check` green; reviewers approve.

## Out of scope
- The public API (T-4266) and the poi suite (T-4267).

## Outcome
- New `crates/rustsheet-xlsx/src/import/encryption.rs`: `decrypt_package(info, package, password, limits)` reads
  Agile (4.4: AES-128/192/256-CBC, SHA-1/256/384/512, HMAC verified in constant time before any plain text is
  returned) and Standard (2.2/3.2/4.2 with `fCryptoAPI` + `fAES`: AES-ECB, the 50,000-round SHA-1 derivation)
  packages. The verifier comparisons use `subtle::ConstantTimeEq`, the password's UTF-16LE bytes and every derived
  key live in `Zeroizing` buffers, and all the shape checks of SPEC-0031 D3 run before the data is used.
- `import/ole.rs`: `decrypt_ole_package(bytes, password, limits)` opens the container through the same bounded path
  as `classify_ole_compound_file` (shared `open_container`), reads `EncryptionInfo` with `take(1 MiB + 1)` and
  `EncryptedPackage` with `take(max_uncompressed_bytes + 1)`, and treats a stream that ends before its declared
  size as `MalformedEncryption` (T-4264's deferred findings). `MAX_DIRECTORY_ENTRIES`'s doc now says it bounds the
  walk, not `cfb`'s eager parse.
- New `XlsxError` variants `WrongPassword`, `UnsupportedEncryption(String)`, `MalformedEncryption(String)` (none
  ever holds the password; a test checks `Display` and `Debug`), and `ImportLimit::SpinCount` with
  `ImportLimits::max_spin_count` (default 1,000,000, SPEC-0031 D3). A `spinCount` past 10,000,000 is
  `UnsupportedEncryption`, past the limit the `SpinCount` limit error, both before any hashing. T-4266 maps them
  onto `LoadError` and `LoadOptions::max_spin_count`.
- Dependencies: `aes` 0.8.4, `cbc` 0.1.2, `ecb` 0.1.2, `sha1` 0.10.6, `hmac` 0.12.1, `base64` 0.22.1, `zeroize`
  1.8.2, `subtle` 2.6.1, pinned exactly with default features off, plus the `sha2` 0.10.9 already in Cargo.lock.
  The crates.io state was rechecked: the newest RustCrypto releases (`aes` 0.9, `hmac` 0.13) move to digest 0.11 /
  cipher 0.5, which would build a second digest stack beside the `sha2` 0.10 already locked, so the 0.10 line is
  used. Ten crates enter Cargo.lock (the eight above plus `cipher` and `inout`); `cargo deny check`: advisories,
  bans, licenses and sources ok.
- Tests: known answers for both derivations and the Standard verifier (msoffcrypto-tool 6.1.0's published doctest
  vectors); round trips on `tests/encrypted/agile_sha512.xlsx` (made by msoffcrypto-tool from `basic_text.xlsx`,
  non-ASCII password outside the BMP, command in the test's doc comment) and poi's `protected_passtika.xlsx`
  (Standard, `tika`; the decrypted zip's SHA-256 is msoffcrypto-tool's); wrong passwords; every malformed input of
  SPEC-0031's test plan (truncated package, declared size past the stream and past the budget, spinCount
  10,000,001 and past u32, keyBits 7, salt of 0 and 65 bytes, a 2 MiB `EncryptionInfo`, a certificate-only key
  encryptor, versions 3.3/4.3/1.1, RC4 and fExternal flags), the HMAC on a tampered last block and size field, XML
  element-count and depth caps, a short verifier hash, a short CFB stream, and Standard header truncations.
- Checked by hand (not committed): poi `58616.xlsx` decrypts with `VelvetSweatshop` (Standard 4.2, AES-128) and
  `protected_passtika.xlsx` with `tika`, each byte-identical to msoffcrypto-tool's output, and both decrypted zips
  load. That is input for T-4263 (E1, E2) and T-4267's password table.
- Fuzz: new target `encrypted_package` (decision 0287 adds it to the nightly matrix; seeds are the encrypted
  fixtures via `fuzz/seed_corpus.py`).
- docs/excel-semantics/xlsx-storage.md: the resource-limits DEVIATION entry now lists the OLE bounds and the
  spin-count limit; the encrypted-package entry describes the decryption and its errors.
- Eval delta: none (the load API does not call the decryption until T-4266; poi stays `!InvalidCase`).
