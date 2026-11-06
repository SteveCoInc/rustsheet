#![allow(clippy::panic, reason = "tests")]

//! Malformed ZIP input stays a zip error, not a panic or another error kind.

use rustsheet_xlsx::error::XlsxError;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

/// Empty bytes, non-ZIP bytes and a truncated local-file header all come back as
/// `XlsxError::Zip` (ported from ironcalc 4e6db1e6).
#[test]
fn malformed_zip_inputs_remain_zip_errors() {
    for bytes in [&b""[..], b"not a ZIP archive", b"PK\x03\x04"] {
        match load_from_xlsx_bytes(bytes, "book", "en", "UTC") {
            Err(XlsxError::Zip(_)) => {}
            Err(other) => panic!("{bytes:?}: expected a zip error, got {other:?}"),
            Ok(_) => panic!("{bytes:?}: malformed input loaded"),
        }
    }
}
