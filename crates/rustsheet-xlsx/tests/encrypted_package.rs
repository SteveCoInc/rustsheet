//! Decrypting a password-protected xlsx (SPEC-0031 D2, D3, D5; T-4265): round trips on files
//! encrypted by independent tools, and a typed error, never a panic, for each malformed input of
//! SPEC-0031's test plan.
//!
//! Fixtures (crates/rustsheet-xlsx/tests/encrypted):
//! - `agile_sha512.xlsx`: `basic_text.xlsx` (next to this file) encrypted by msoffcrypto-tool
//!   6.1.0 with Agile encryption (AES-256-CBC, SHA-512, spinCount 1,000 to keep the test fast),
//!   password `Pässwörd🔑` (non-ASCII and outside the BMP, so the UTF-16LE hashing is tested):
//!   `ECMA376Agile.encrypt("Pässwörd🔑", io.BytesIO(open("basic_text.xlsx", "rb").read()),
//!   spin_count=1000)` from `msoffcrypto.method.ecma376_agile`, written as is.
//! - `standard_aes128_passtika.xlsx`: Apache POI's test-data/spreadsheet/protected_passtika.xlsx
//!   (Apache-2.0, POI rev 9958a2a2, the poi suite's pin), Standard encryption (`EncryptionInfo`
//!   4.2, AES-128). POI's tests publish its password, `tika`. msoffcrypto-tool 6.1.0 decrypts it
//!   (`msoffcrypto-tool -p tika`) to a zip whose SHA-256 is [`PASSTIKA_ZIP_SHA256`].

use std::error::Error;
use std::io::{Cursor, Read};

use rustsheet_xlsx::error::{ImportLimit, XlsxError};
use rustsheet_xlsx::import::{
    decrypt_ole_package, decrypt_package, load_from_xlsx_bytes, ImportLimits,
};
use sha2::{Digest, Sha256};

type TestResult = Result<(), Box<dyn Error>>;

const AGILE: &[u8] = include_bytes!("encrypted/agile_sha512.xlsx");
const AGILE_PASSWORD: &str = "Pässwörd🔑";
const AGILE_PLAIN: &[u8] = include_bytes!("basic_text.xlsx");
const STANDARD: &[u8] = include_bytes!("encrypted/standard_aes128_passtika.xlsx");
const STANDARD_PASSWORD: &str = "tika";
const PASSTIKA_ZIP_SHA256: &str =
    "2bf260af678ac1b93b5dc618e0892aff98062f4566b3b6f5f3c9cba4e02b8d68";

/// The `EncryptionInfo` and `EncryptedPackage` streams of a fixture.
fn streams(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let mut file = cfb::CompoundFile::open(Cursor::new(bytes))?;
    let mut read = |name: &str| -> Result<Vec<u8>, Box<dyn Error>> {
        let mut out = Vec::new();
        file.open_stream(name)?.read_to_end(&mut out)?;
        Ok(out)
    };
    Ok((read("/EncryptionInfo")?, read("/EncryptedPackage")?))
}

/// The Agile streams with `from` replaced by `to` in the descriptor.
fn agile_with(from: &str, to: &str) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let (info, package) = streams(AGILE)?;
    let xml = std::str::from_utf8(&info[8..])?;
    assert!(xml.contains(from), "the fixture has no {from}");
    let mut edited = info[..8].to_vec();
    edited.extend_from_slice(xml.replacen(from, to, 1).as_bytes());
    Ok((edited, package))
}

fn decrypt(info: &[u8], package: &[u8]) -> Result<Vec<u8>, XlsxError> {
    decrypt_package(info, package, AGILE_PASSWORD, &ImportLimits::default())
}

fn bytes_limit(max: u64) -> XlsxError {
    XlsxError::Limit {
        limit: ImportLimit::UncompressedBytes,
        max,
    }
}

fn malformed(result: &Result<Vec<u8>, XlsxError>) -> bool {
    matches!(result, Err(XlsxError::MalformedEncryption(_)))
}

fn unsupported(result: &Result<Vec<u8>, XlsxError>) -> bool {
    matches!(result, Err(XlsxError::UnsupportedEncryption(_)))
}

#[test]
fn agile_round_trip() -> TestResult {
    let zip = decrypt_ole_package(AGILE, AGILE_PASSWORD, &ImportLimits::default())?;
    assert_eq!(zip, AGILE_PLAIN);
    // The plain package loads like any xlsx.
    assert!(load_from_xlsx_bytes(&zip, "case", "en", "UTC").is_ok());
    Ok(())
}

#[test]
fn standard_round_trip() -> TestResult {
    let zip = decrypt_ole_package(STANDARD, STANDARD_PASSWORD, &ImportLimits::default())?;
    let digest = Sha256::digest(&zip);
    let hex = format!("{digest:x}");
    assert_eq!(hex, PASSTIKA_ZIP_SHA256);
    assert!(zip.starts_with(b"PK\x03\x04"));
    Ok(())
}

#[test]
fn a_wrong_password_is_its_own_error() {
    let limits = ImportLimits::default();
    for (bytes, password) in [
        (AGILE, "pässwörd🔑"), // case-sensitive
        (AGILE, ""),
        (STANDARD, "Tika"),
        (STANDARD, ""),
    ] {
        assert_eq!(
            decrypt_ole_package(bytes, password, &limits),
            Err(XlsxError::WrongPassword),
            "{password}"
        );
    }
}

#[test]
fn the_error_text_never_holds_the_password() -> TestResult {
    let (info, package) = agile_with("keyBits=\"256\"\n", "keyBits=\"7\"\n")?;
    for error in [
        decrypt_ole_package(AGILE, "secret-tail-1", &ImportLimits::default()),
        decrypt_package(&info, &package, "secret-tail-1", &ImportLimits::default()),
    ]
    .into_iter()
    .filter_map(Result::err)
    {
        assert!(!error.to_string().contains("secret-tail-1"));
        assert!(!format!("{error:?}").contains("secret-tail-1"));
    }
    Ok(())
}

#[test]
fn a_container_that_is_not_an_encrypted_xlsx_keeps_its_classification() {
    let limits = ImportLimits::default();
    assert_eq!(
        decrypt_ole_package(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1", "x", &limits),
        Err(XlsxError::OleCompoundFile)
    );
    let small = ImportLimits {
        max_uncompressed_bytes: 1024,
        ..limits
    };
    assert_eq!(
        decrypt_ole_package(AGILE, AGILE_PASSWORD, &small),
        Err(bytes_limit(1024))
    );
}

#[test]
fn spin_counts_past_the_limits_are_refused_before_hashing() -> TestResult {
    // Past MS-OFFCRYPTO's maximum: malformed for the format, whatever the caller allows.
    let (info, package) = agile_with("spinCount=\"1000\"", "spinCount=\"10000001\"")?;
    let generous = ImportLimits {
        max_spin_count: u32::MAX,
        ..ImportLimits::default()
    };
    assert!(unsupported(&decrypt_package(
        &info, &package, "x", &generous
    )));
    let (info, package) = agile_with("spinCount=\"1000\"", "spinCount=\"99999999999999999999\"")?;
    assert!(unsupported(&decrypt(&info, &package)));
    // Past the caller's limit (default 1,000,000): the spin-count limit error.
    let (info, package) = agile_with("spinCount=\"1000\"", "spinCount=\"1000001\"")?;
    assert_eq!(
        decrypt(&info, &package),
        Err(XlsxError::Limit {
            limit: ImportLimit::SpinCount,
            max: 1_000_000
        })
    );
    // Neither derived anything: a 10,000,001-round SHA-512 would take seconds, and a password
    // that cannot be right gives the same refusal.
    let (info, package) = agile_with("spinCount=\"1000\"", "spinCount=\"999\"")?;
    assert_eq!(decrypt(&info, &package), Err(XlsxError::WrongPassword));
    Ok(())
}

#[test]
fn package_shape_checks() -> TestResult {
    let (info, package) = streams(AGILE)?;
    // Shorter than its size field.
    assert!(malformed(&decrypt(&info, &package[..7])));
    // A truncated ciphertext that is not whole AES blocks.
    assert!(malformed(&decrypt(&info, &package[..package.len() - 1])));
    // Whole blocks, but fewer bytes than the declared size.
    assert!(malformed(&decrypt(&info, &package[..package.len() - 4096])));
    // A declared size past the byte budget is the limit error, before anything is allocated.
    let mut huge = package.clone();
    huge[..8].copy_from_slice(&u64::MAX.to_le_bytes());
    let limits = ImportLimits::default();
    assert_eq!(
        decrypt_package(&info, &huge, AGILE_PASSWORD, &limits),
        Err(bytes_limit(limits.max_uncompressed_bytes))
    );
    // A declared size past the ciphertext but inside the budget is malformed.
    let mut over = package.clone();
    let past = u64::try_from(package.len())?;
    over[..8].copy_from_slice(&past.to_le_bytes());
    assert!(malformed(&decrypt(&info, &over)));
    Ok(())
}

#[test]
fn a_tampered_package_fails_its_integrity_check() -> TestResult {
    // SPEC-0031 E4 / D5: the last ciphertext block altered. The password is right, so only the
    // HMAC catches it, and no plain text comes back.
    let (info, mut package) = streams(AGILE)?;
    let last = package.len() - 1;
    package[last] ^= 1;
    let error = decrypt(&info, &package).err();
    assert_eq!(
        error,
        Some(XlsxError::MalformedEncryption(
            "the encrypted package failed its integrity check".to_string()
        ))
    );
    // The size field is covered too.
    let (info, mut package) = streams(AGILE)?;
    package[0] ^= 1;
    assert!(malformed(&decrypt(&info, &package)));
    Ok(())
}

#[test]
fn descriptor_parameter_checks() -> TestResult {
    let cases: [(&str, &str); 9] = [
        ("keyBits=\"256\"\n", "keyBits=\"7\"\n"),
        ("blockSize=\"16\" keyBits=\"256\"\n", "blockSize=\"8\" keyBits=\"256\"\n"),
        // A salt of 0 and of 65 bytes.
        ("saltValue=\"PqCYXZbMM6ZhfuBQkmqzCw==\"", "saltValue=\"\""),
        (
            "saltSize=\"16\" blockSize=\"16\" keyBits=\"256\"\n",
            "saltSize=\"65\" blockSize=\"16\" keyBits=\"256\"\n",
        ),
        // hashSize that does not match the algorithm.
        ("hashSize=\"64\" cipherAlgorithm", "hashSize=\"20\" cipherAlgorithm"),
        // A short hash output: the verifier hash shorter than SHA-512's 64 bytes.
        (
            "encryptedVerifierHashValue=\"f+pF+gwg6uDVOwM51ZOebiip9eogQbg9v2q4tsdRRUgQ7tiTOVQVMN7YQKGLbDhcGcC3XxMbTFMECqeZ8rUhhw==\"",
            "encryptedVerifierHashValue=\"f+pF+gwg6uDVOwM51ZOebg==\"",
        ),
        ("encryptedKeyValue=\"t9Gg", "encryptedKeyValue=\"!!!!"),
        ("<dataIntegrity ", "<notIntegrity "),
        ("spinCount=\"1000\"", "spinCount=\"-1\""),
    ];
    for (from, to) in cases {
        let (info, package) = agile_with(from, to)?;
        let result = decrypt(&info, &package);
        assert!(malformed(&result), "{to}: {result:?}");
    }
    let mut salt65 = "A".repeat(88);
    salt65.push_str("==");
    let (info, package) = agile_with("PqCYXZbMM6ZhfuBQkmqzCw==", &salt65)?;
    assert!(malformed(&decrypt(&info, &package)));
    // A base64 field past its length cap is refused before it is decoded.
    let (info, package) = agile_with("t9Gg", &"A".repeat(400))?;
    assert!(malformed(&decrypt(&info, &package)));
    Ok(())
}

#[test]
fn unsupported_kinds_are_named() -> TestResult {
    for (from, to) in [
        ("cipherAlgorithm=\"AES\"", "cipherAlgorithm=\"3DES\""),
        ("ChainingModeCBC", "ChainingModeCFB"),
        ("hashAlgorithm=\"SHA512\"", "hashAlgorithm=\"MD5\""),
        // A certificate key encryptor only.
        ("keyEncryptor/password\">", "keyEncryptor/certificate\">"),
    ] {
        let (info, package) = agile_with(from, to)?;
        let result = decrypt(&info, &package);
        assert!(unsupported(&result), "{to}: {result:?}");
    }
    let (info, package) = streams(AGILE)?;
    for version in [[3u8, 0, 3, 0], [4, 0, 3, 0], [1, 0, 1, 0], [5, 0, 5, 0]] {
        let mut edited = info.clone();
        edited[..4].copy_from_slice(&version);
        assert!(unsupported(&decrypt(&edited, &package)), "{version:?}");
    }
    // Standard flags without fAES: RC4 CryptoAPI.
    let (info, package) = streams(STANDARD)?;
    let mut rc4 = info.clone();
    rc4[4] &= !0x20;
    assert!(unsupported(&decrypt_package(
        &rc4,
        &package,
        "tika",
        &ImportLimits::default()
    )));
    // fExternal: Extensible.
    let mut external = info.clone();
    external[4] |= 0x10;
    assert!(unsupported(&decrypt_package(
        &external,
        &package,
        "tika",
        &ImportLimits::default()
    )));
    Ok(())
}

#[test]
fn standard_header_checks() -> TestResult {
    let (info, package) = streams(STANDARD)?;
    let limits = ImportLimits::default();
    for cut in [3, 8, 12, 40, info.len() - 1] {
        let result = decrypt_package(&info[..cut], &package, "tika", &limits);
        assert!(malformed(&result), "cut at {cut}: {result:?}");
    }
    // HeaderSize past the stream.
    let mut long = info.clone();
    long[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(malformed(&decrypt_package(
        &long, &package, "tika", &limits
    )));
    Ok(())
}

#[test]
fn descriptor_shape_is_bounded() -> TestResult {
    // 64 elements is the cap; Excel's descriptor has 6.
    let many = "<x/>".repeat(64);
    let (info, package) = agile_with("<keyEncryptors>", &format!("{many}<keyEncryptors>"))?;
    assert!(malformed(&decrypt(&info, &package)));
    let deep = "<x>".repeat(8);
    let close = "</x>".repeat(8);
    let (info, package) = agile_with("<keyEncryptors>", &format!("{deep}{close}<keyEncryptors>"))?;
    assert!(malformed(&decrypt(&info, &package)));
    // Not XML, not UTF-8, an undeclared entity.
    // (Text is never read, so an entity there is harmless; one in a read attribute is refused.)
    let (info, package) = agile_with("saltValue=\"IEEL", "saltValue=\"&bomb;IEEL")?;
    assert!(malformed(&decrypt(&info, &package)));
    let mut bad_utf8 = info.clone();
    bad_utf8.push(0xff);
    assert!(malformed(&decrypt(&bad_utf8, &package)));
    // An EncryptionInfo of 2 MiB.
    let mut big = info;
    big.resize(2 << 20, b' ');
    assert!(malformed(&decrypt(&big, &package)));
    Ok(())
}

#[test]
fn a_stream_shorter_than_its_declared_size_is_malformed() -> TestResult {
    // `cfb` checks a sector chain against the stream's declared size only when the stream is
    // read (docs/excel-semantics/xlsx-storage.md): a directory entry that declares 64 KiB more
    // than its chain holds must not read as a shorter package.
    let mut bytes = AGILE.to_vec();
    let name: Vec<u8> = "EncryptedPackage"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let entry = bytes
        .windows(name.len())
        .position(|w| w == name.as_slice())
        .ok_or("no EncryptedPackage entry")?;
    // MS-CFB 2.6.3: the stream size is the u64 at offset 120 of the 128-byte entry.
    let size_at = entry + 120;
    let size = u32::from_le_bytes(bytes[size_at..size_at + 4].try_into()?);
    bytes[size_at..size_at + 4].copy_from_slice(&(size + 65_536).to_le_bytes());
    let result = decrypt_ole_package(&bytes, AGILE_PASSWORD, &ImportLimits::default());
    assert!(
        matches!(
            result,
            Err(XlsxError::MalformedEncryption(_) | XlsxError::OleCompoundFile)
        ),
        "{result:?}"
    );
    Ok(())
}
