# SPEC-0031 — Reading password-protected xlsx, and the legacy `.xls` plan

- **Status:** draft (the encrypted-xlsx part is ready to implement; the `.xls` part waits on the owner, T-4268)
- **Milestone / lane:** M12 / api (the decryption lives in crates/rustsheet-xlsx)
- **Tasks:** T-4243 (this spec), T-4263 (probe), T-4264 (container classification), T-4265 (decryption),
  T-4266 (load API and Python), T-4267 (poi suite passwords), T-4268 (owner ruling on `.xls`)

## Problem
Since T-3972 (decision 0279) the importer refuses any file that starts with the OLE compound-file signature
(`D0 CF 11 E0 A1 B1 1A E1`) with `XlsxError::OleCompoundFile`. That one error covers two different gaps:

- **Password-protected xlsx.** Excel encrypts an xlsx by wrapping the zip in an OLE compound file (MS-CFB) holding an
  `EncryptionInfo` stream and an `EncryptedPackage` stream (ECMA-376 Part 2 / MS-OFFCRYPTO). Excel asks for the
  password and opens it. Two poi fixtures are such files: `58616.xlsx` and `protected_passtika.xlsx`. Both are known
  failures (`!InvalidCase` in eval/baseline/poi.txt) owned by T-4243. The SPEC-0026 load API has no way to pass a
  password.
- **Legacy binary workbooks.** An `.xls` (BIFF8) uses the same container with a `Workbook` (or `Book`) stream. Excel
  opens it with no password. No suite in eval/ has one today.

docs/excel-semantics/xlsx-storage.md "Password-protected and non-zip packages" records the current behavior as a
known gap, not a deviation.

## Goals / non-goals
Goals:
- `Workbook::open` / `from_xlsx_bytes` (and the Python `Workbook.open` / `from_xlsx_bytes`) take an optional password
  and read an ECMA-376 encrypted xlsx with Agile encryption (what Excel 2010 and later write) or Standard encryption
  (Excel 2007).
- Wrong, missing and unsupported cases are typed errors the caller can tell apart.
- Every step is bounded under directive 5: CFB sector chains, stream sizes, the `EncryptionInfo` XML, the key
  derivation's spin count, and the decrypted size.
- The two poi fixtures pass on their cached values.

Non-goals:
- Writing encrypted files (save stays plain xlsx).
- Extensible encryption (`EncryptionInfo` 3.3 / 4.3), IRM/DRM-protected packages (`\x06DataSpaces` with a transform
  other than the password one) and RC4 / CryptoAPI encryption (`.xls` only). They are refused with
  `UnsupportedEncryption`.
- Workbook/sheet *protection* passwords (`<workbookProtection>`, `<sheetProtection>`): those are not encryption, and
  the importer already reads such files.
- Reading `.xls`: see "The `.xls` plan".

## Excel behavior to match
- **E1. Excel opens the two fixtures with their passwords.** UNVERIFIED: T-4263 probes it. POI's own tests publish
  passtika's password. For `58616.xlsx` the probe takes the password from POI's test code, or records that none is
  published (the fixture then leaves the suite's scope by T-4267's table, not by a password guess). The probe records each fixture's `EncryptionInfo` version and the cached values Excel shows after
  opening.
- **E2. Default password.** Excel opens without a prompt a file encrypted with the password `VelvetSweatshop`.
  UNVERIFIED for xlsx: it is documented for `.xls` write-reservation files. T-4263 probes it with an xlsx encrypted
  that way. If Excel does open it silently, the loader tries `VelvetSweatshop` when no password is given (rule D4).
- **E3. Password text.** A password is at most 255 UTF-16 code units and is hashed as UTF-16LE, case-sensitive
  (MS-OFFCRYPTO 2.3.4.7 and 2.3.4.11). A longer password is refused, not truncated. UNVERIFIED that Excel refuses
  rather than truncates; T-4263 checks it.
- **E4. Integrity.** Agile files carry an HMAC of the package (`dataIntegrity`). Whether Excel refuses a file whose
  HMAC does not match, or opens it anyway, is UNVERIFIED (T-4263, with a fixture whose last ciphertext block is
  altered). Until the probe shows otherwise the loader verifies it and refuses a mismatch (rule D5).

The probe's findings go into the xlsx-storage.md entry. The entry's "known gap" status ends when T-4266 merges.

## Design

### D1. Container: a bounded MS-CFB reader (T-4264)
Classification runs on the bytes `is_ole_compound_file` already detects (crates/rustsheet-xlsx/src/import/package.rs).
- Use the `cfb` crate (MIT; one dependency, `uuid`, with no I/O beyond the `Read + Seek` we hand it, an in-memory
  `Cursor`) rather than a reader of our own. It is reviewed under cargo-deny like any dependency, and its version is
  pinned in the workspace. Under it, `Package::new` keeps its 8-byte check, so a zip never reaches `cfb`.
- Bounds, checked before any stream is read: at most 4,096 directory entries; each stream read with
  `take(limit + 1)`, where `limit` is `max_uncompressed_bytes` for `EncryptedPackage`, 1 MiB for `EncryptionInfo`, and
  nothing else is read. `cfb` validates sector chains (cycles, out-of-range sectors) and returns errors, not panics;
  T-4264 adds the fuzz target `cfb_container` to prove it on our inputs (SPEC-0026 decision 5's corpus).
- Classification, from the root storage's entries:
  - `EncryptionInfo` and `EncryptedPackage` streams: an encrypted package (D2).
  - `Workbook` or `Book` stream: a legacy binary workbook, `XlsxError::LegacyBinaryWorkbook`.
  - Anything else: `XlsxError::OleCompoundFile`, as today.

  T-4264 replaces the single error with these three before any decryption exists, so callers can already tell an
  encrypted xlsx from an `.xls`.

### D2. Decryption (T-4265)
A new module `crates/rustsheet-xlsx/src/import/encryption.rs` with one entry point:

```rust
/// Decrypts an ECMA-376 encrypted package into the zip bytes it holds.
pub(crate) fn decrypt_package(
    info: &[u8],
    package: &[u8],
    password: &str,
    limits: &ImportLimits,
) -> Result<Vec<u8>, XlsxError>;
```

`EncryptionInfo` starts with a version (`major.minor`, u16 each, little-endian) and flags:
- **4.4: Agile.** After the 8-byte header comes UTF-8 XML (`<encryption>`), parsed with the quick-xml the crate
  already uses. Read `keyData` (`saltValue`, `hashAlgorithm`, `blockSize`, `keyBits`, `cipherAlgorithm`,
  `cipherChaining`), `dataIntegrity` (`encryptedHmacKey`, `encryptedHmacValue`) and the one `keyEncryptor` whose `uri`
  is the password key encryptor (`spinCount`, `saltValue`, `encryptedVerifierHashInput`,
  `encryptedVerifierHashValue`, `encryptedKeyValue`). Supported: AES-128/192/256 in CBC with SHA-1, SHA-256, SHA-384
  or SHA-512. Anything else (`cipherChaining` CFB, 3DES, RC2, a certificate key encryptor only) is
  `UnsupportedEncryption`.
  - Key derivation (MS-OFFCRYPTO 2.3.4.11): `H0 = H(salt || password_utf16le)`, then `spinCount` times
    `Hi = H(i_le32 || Hi-1)`. For each of the three block keys (verifier input `fe a7 d2 76 3b 4b 9e 79`, verifier
    hash `d7 aa 0f 6d 30 61 34 4e`, key value `14 6e 0b e7 ab ac d0 d6`), `K = H(Hn || block)`, truncated to
    `keyBits/8` or padded with `0x36`.
  - Verify the password: decrypt the verifier input and the verifier hash (IV = the key encryptor's salt). A hash of
    the input that is not the decrypted hash value is `WrongPassword`.
  - Decrypt the intermediate key, then the package: 4,096-byte segments, segment i with IV =
    `H(keyData.salt || i_le32)` truncated to `blockSize`. The first 8 bytes of `EncryptedPackage` are the plain size
    (u64 LE), and the output is truncated to it.
  - Integrity (D5): HMAC over the whole `EncryptedPackage` stream, with the HMAC key and value decrypted under the
    block keys `5f b2 ad 01 0c b9 e1 f6` and `a0 67 7f 02 b2 2c 84 33`.
- **x.2 with `fCryptoAPI` and `fAES` set (x in 2, 3, 4): Standard.** A binary `EncryptionHeader` (AES-128/192/256,
  SHA-1 only) and `EncryptionVerifier`. Key derivation (2.3.4.7): `H0 = SHA1(salt || password_utf16le)`, 50,000
  rounds of `Hi = SHA1(i_le32 || Hi-1)`, `Hfinal = SHA1(Hn || 0_le32)`, then the `0x36` / `0x5c` X1/X2 expansion,
  truncated to the key size. The verifier is checked in AES-ECB. The package is AES-ECB after its 8-byte size. Standard
  has no integrity data.
- **Any other version or flag set:** `UnsupportedEncryption`.

Crypto dependencies come from RustCrypto: `aes`, `cbc`, `ecb`, `sha1`, `sha2`, `hmac` (all Apache-2.0 OR MIT,
already accepted by deny.toml's license list) and `base64` for the XML values. The password is kept in a `Zeroizing`
buffer (`zeroize`, same licence) while its UTF-16 form and the derived keys live. They add no build scripts and no
`unsafe` beyond what `aes` uses for its CPU intrinsics, which cargo-deny and the security reviewer check in T-4265.
Hand-rolled crypto is out (see Alternatives).

### D3. Limits (directive 5)
- **`spinCount`:** MS-OFFCRYPTO caps it at 10,000,000, and a larger value is refused as a malformed file
  (`UnsupportedEncryption`), so no deviation from Excel is needed. At 10,000,000 SHA-512 rounds the derivation costs
  about 2 to 4 s of CPU, once per load (not per segment). Excel writes 100,000. `LoadOptions::max_spin_count`
  (default 10,000,000) lets a server lower it. A file above a lowered cap is `LoadError::Options`, like the other
  limits.
- **Package size:** the declared plain size and the stream are both bounded by `max_uncompressed_bytes`, before any
  allocation. The decrypted zip then goes through the existing zip limits unchanged, so an encrypted bomb is caught
  where a plain one is.
- **Block sizes:** `blockSize` must be 16 (AES), `keyBits` 128, 192 or 256, and a salt 1 to 64 bytes. Base64 fields
  are length-checked before decoding.
- **Memory:** at most 2 × the package size at once (ciphertext and plain text). The 4,096-byte segments are
  decrypted in place into the output buffer.

### D4. Load API (T-4266)
- `LoadOptions` gains `pub password: Option<String>` (default `None`). `LoadOptions` is not `#[non_exhaustive]`, so a
  caller building it with a struct literal and no `..Default::default()` breaks. This is acceptable before the API v1
  freeze (SPEC-0026 decision 7), and the CHANGELOG notes it.
- `LoadError` (already `#[non_exhaustive]`) gains:
  - `PasswordRequired`: the file is encrypted and no password was given. If E2 holds, it applies only after
    `VelvetSweatshop` also fails.
  - `WrongPassword`;
  - `Unsupported(String)`: the encryption kind, or a legacy `.xls`, named in the message.
- `LoadError::Format` keeps everything else, including a failed HMAC (D5) and a malformed `EncryptionInfo`.
- Python: `Workbook.open(path, recalc=True, clock=None, name=None, password=None)` and the same keyword on
  `from_xlsx_bytes`. The errors map to new `PasswordRequiredError` and `WrongPasswordError` subclasses of the existing
  `LoadError`, so `except rustsheet.LoadError` still catches them. The `.pyi` stub and README document them.
- The password is never logged, never put in an error message and never stored on the `Workbook`.
- Saving a workbook opened from an encrypted file writes it unencrypted. The doc comment on `save_xlsx` says so.

### D5. Integrity
An Agile file whose HMAC does not match is `LoadError::Format("encrypted package failed its integrity check")`,
unless T-4263 shows Excel opens such a file. In that case T-4265 follows Excel and records it in xlsx-storage.md.
Verification runs before the zip is parsed, so the parser never reads ciphertext that was tampered with.

### The `.xls` plan
Excel opens BIFF8 workbooks, so not reading them is a gap, not a deviation. A BIFF8 reader is a second importer: the
record stream, SST, formats, BIFF formula tokens to text, shared and array formulas, names and external references.
Size L+, several specs of its own, and fixtures with no oracle source in eval/ today. The proposal:
- Until API v1, RustSheet does not read `.xls`. T-4264 refuses it with its own typed error, `LegacyBinaryWorkbook`
  (`LoadError::Unsupported` on the facade), which says to convert the file to xlsx.
- Whether RustSheet ever reads `.xls`, and in which milestone, is a scope decision for the owner (T-4268, blocked on
  the owner). Until the ruling the xlsx-storage.md entry keeps it as a known gap.

## Alternatives considered
- **Hand-rolled AES/SHA/HMAC.** No dependency, but writing crypto ourselves is a correctness and side-channel risk
  for no gain: these are reading, not secret-keeping, primitives, and RustCrypto's crates are the audited standard.
  Rejected.
- **An all-in-one crate (`office-crypto`, `ms-offcrypto-writer`).** Smaller diff, but the ones available are young,
  pull `anyhow` or `rand`, panic on malformed input or do not bound `spinCount` and stream sizes. We would still have
  to wrap and fuzz them. Using RustCrypto primitives under our own bounded parser keeps every limit in our code.
  Rejected, and T-4265 rechecks the crates.io state when it starts.
- **Our own MS-CFB reader.** About 400 lines of sector-chain walking, which `cfb` already does with cycle checks. We
  use `cfb` and fuzz it (D1). If the fuzz target finds a panic it cannot fix upstream, T-4264 falls back to a minimal
  reader of our own: only the root directory and two streams are needed.
- **A password callback instead of a string.** It lets a UI ask lazily, but the facade is headless and synchronous,
  and a caller can call `open` again after `PasswordRequired`. Rejected for v0. The error makes the retry loop easy.

## Test plan
- **Unit (T-4265):** the MS-OFFCRYPTO worked examples. Known-answer tests for the Standard and Agile key derivation
  with a small `spinCount`, and round trips on fixtures encrypted by an independent tool (msoffcrypto-tool, run once
  by the author, with its output committed as a test fixture and its command recorded in the test's doc comment).
- **Malformed inputs (T-4264, T-4265):** a truncated `EncryptedPackage`, a declared size past the stream, `spinCount`
  10,000,001, `keyBits` 7, a salt of 0 and 65 bytes, a CFB directory cycle, an `EncryptionInfo` of 2 MiB, a
  certificate-only key encryptor, version 3.3 and RC4 flags. Each gives its typed error, with no panic, and the fuzz
  target `cfb_container` and a new `encrypted_package` target run in the fuzz job.
- **poi suite (T-4267):** suite.toml gains a `[passwords]` table (file path to password) that the runner passes as
  `LoadOptions::password`. `58616.xlsx` and `protected_passtika.xlsx` leave `!InvalidCase`, and their cells are
  scored on cached values like every other poi file. The password values come from POI's public test code, so they
  are test data, not secrets.
- **Wrong and missing password:** facade and Python tests for `PasswordRequired`, `WrongPassword` and `Unsupported`
  (a `.xls`), and that the error text never contains the password.
- **Cost:** a bench that opens the passtika fixture. The derivation at its `spinCount` stays under 1 s on the CI
  runner.

## Rollout
1. **T-4263 (research, opus, S):** probe E1–E4 on desktop Excel or excel-graph; record the findings in
   xlsx-storage.md.
2. **T-4264 (api, opus, S):** add `cfb`, classify OLE files (encrypted, legacy binary, other) into typed errors, and
   add the fuzz target. With no decryption yet, the baseline does not change.
3. **T-4265 (api, opus, M; depends on T-4264):** the `encryption` module with Agile and Standard, the limits and the
   unit tests.
4. **T-4266 (api, opus, S; depends on T-4265):** `LoadOptions::password` and `max_spin_count`, the `LoadError`
   variants, Python keywords and exceptions, README and CHANGELOG.
5. **T-4267 (harness, sonnet, S; depends on T-4266):** the poi suite's `[passwords]` table. The two fixtures leave
   the baseline (`make eval-baseline`), and xlsx-storage.md drops "known gap" for encrypted xlsx.
6. **T-4268 (api, opus, S, blocked on the owner):** a ruling on whether and when RustSheet reads `.xls`. Until then,
   T-4264's `LegacyBinaryWorkbook` is the behavior.
