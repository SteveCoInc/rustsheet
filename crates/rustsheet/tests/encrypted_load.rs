//! Opening a password-protected xlsx through the facade (SPEC-0031 D3, D4; T-4266): the right
//! password opens it, a wrong or missing one and an unsupported OLE file are typed errors, and no
//! error or `Debug` output ever contains the password.
//!
//! The encrypted fixtures are T-4265's (crates/rustsheet-xlsx/tests/encrypted; their provenance
//! is documented in crates/rustsheet-xlsx/tests/encrypted_package.rs): `agile_sha512.xlsx` is
//! `basic_text.xlsx` encrypted with Agile encryption (spinCount 1,000), and
//! `standard_aes128_passtika.xlsx` is Apache POI's `protected_passtika.xlsx` (Standard).

use std::error::Error;
use std::io::{Cursor, Write};

use rustsheet::{LoadError, LoadOptions, Password, UnsupportedKind, Workbook};

type TestResult = Result<(), Box<dyn Error>>;

const AGILE: &[u8] = include_bytes!("../../rustsheet-xlsx/tests/encrypted/agile_sha512.xlsx");
const AGILE_PASSWORD: &str = "Pässwörd🔑";
const AGILE_PLAIN: &[u8] = include_bytes!("../../rustsheet-xlsx/tests/basic_text.xlsx");
const STANDARD: &[u8] =
    include_bytes!("../../rustsheet-xlsx/tests/encrypted/standard_aes128_passtika.xlsx");
const STANDARD_PASSWORD: &str = "tika";

fn with_password(password: &str) -> LoadOptions {
    LoadOptions {
        password: Some(Password::new(password)),
        ..LoadOptions::default()
    }
}

/// A version 3 compound file holding one empty stream `stream` in its root storage.
fn ole_file(stream: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file =
        cfb::CompoundFile::create_with_version(cfb::Version::V3, Cursor::new(Vec::new()))?;
    file.create_stream(format!("/{stream}"))?
        .write_all(&[0; 16])?;
    file.flush()?;
    Ok(file.into_inner().into_inner())
}

/// Neither the error's `Display` nor its `Debug` names the password.
fn assert_hides(error: &LoadError, password: &str) {
    assert!(!error.to_string().contains(password), "{error}");
    assert!(!format!("{error:?}").contains(password), "{error:?}");
}

#[test]
fn the_right_password_opens_an_agile_workbook() -> TestResult {
    let opened = Workbook::from_xlsx_bytes(AGILE, &with_password(AGILE_PASSWORD))?;
    let plain = Workbook::from_xlsx_bytes(AGILE_PLAIN, &LoadOptions::default())?;
    assert_eq!(opened.sheets(), plain.sheets());
    assert_eq!(opened.value("A1")?, plain.value("A1")?);
    Ok(())
}

#[test]
fn the_right_password_opens_a_standard_workbook_from_a_file() -> TestResult {
    let dir = std::env::temp_dir().join(format!("rustsheet-t4266-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("passtika.xlsx");
    std::fs::write(&path, STANDARD)?;
    let opened = Workbook::open(&path, &with_password(STANDARD_PASSWORD));
    std::fs::remove_dir_all(&dir)?;
    assert!(!opened?.sheets().is_empty());
    Ok(())
}

#[test]
fn a_missing_password_is_password_required() {
    for bytes in [AGILE, STANDARD] {
        let result = Workbook::from_xlsx_bytes(bytes, &LoadOptions::default());
        assert!(matches!(result, Err(LoadError::PasswordRequired)));
    }
}

#[test]
fn a_wrong_password_is_wrong_password_and_never_echoed() {
    for (bytes, password) in [
        (AGILE, "pässwörd🔑-not-it"),
        (AGILE, ""),
        (STANDARD, "Tika-secret"),
    ] {
        let Err(error) = Workbook::from_xlsx_bytes(bytes, &with_password(password)) else {
            panic!("{password:?} opened the workbook");
        };
        assert!(matches!(error, LoadError::WrongPassword), "{error:?}");
        if !password.is_empty() {
            assert_hides(&error, password);
        }
    }
}

#[test]
fn a_password_on_a_plain_workbook_is_ignored() -> TestResult {
    let opened = Workbook::from_xlsx_bytes(AGILE_PLAIN, &with_password("unused"))?;
    assert!(!opened.sheets().is_empty());
    Ok(())
}

#[test]
fn a_legacy_xls_is_unsupported_with_or_without_a_password() -> TestResult {
    // `ole_file("Workbook")` written once: a version 3 container whose root storage holds a
    // 16-byte `Workbook` stream, the BIFF8 record stream's name. The Python tests read it too.
    let xls = include_bytes!("fixtures/biff_container.xls");
    for opts in [LoadOptions::default(), with_password("s3cr3t-xls")] {
        let Err(error) = Workbook::from_xlsx_bytes(xls, &opts) else {
            panic!("an .xls loaded");
        };
        assert!(
            matches!(
                error,
                LoadError::Unsupported(UnsupportedKind::LegacyBinaryWorkbook)
            ),
            "{error:?}"
        );
        assert_hides(&error, "s3cr3t-xls");
    }
    let other = Workbook::from_xlsx_bytes(&ole_file("Other")?, &with_password("x"));
    assert!(matches!(
        other,
        Err(LoadError::Unsupported(UnsupportedKind::OtherOleFile))
    ));
    Ok(())
}

#[test]
fn spin_count_past_the_limit_is_refused_before_hashing() {
    // The Agile fixture asks for 1,000 rounds.
    let opts = LoadOptions {
        max_spin_count: 999,
        ..with_password(AGILE_PASSWORD)
    };
    let result = Workbook::from_xlsx_bytes(AGILE, &opts);
    assert!(
        matches!(result, Err(LoadError::Options(_))),
        "{:?}",
        result.err()
    );
    let opts = LoadOptions {
        max_spin_count: 1_000,
        ..with_password(AGILE_PASSWORD)
    };
    assert!(Workbook::from_xlsx_bytes(AGILE, &opts).is_ok());
}

#[test]
fn the_defaults_and_debug_never_show_the_password() {
    let defaults = LoadOptions::default();
    assert!(defaults.password.is_none());
    assert_eq!(defaults.max_spin_count, 1_000_000);
    let opts = with_password("hunter2-secret");
    let debug = format!("{opts:?}");
    assert!(!debug.contains("hunter2-secret"), "{debug}");
    assert!(debug.contains("Password(***)"), "{debug}");
}

#[test]
fn an_oversized_encrypted_file_is_a_limit_error() {
    let opts = LoadOptions {
        max_uncompressed_bytes: 1024,
        ..with_password(AGILE_PASSWORD)
    };
    let result = Workbook::from_xlsx_bytes(AGILE, &opts);
    assert!(
        matches!(result, Err(LoadError::Options(_))),
        "{:?}",
        result.err()
    );
}
