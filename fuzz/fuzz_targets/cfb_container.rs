//! Imports an untrusted OLE compound file through the public facade (SPEC-0031 D1, T-4264).
//!
//! A password-protected xlsx or a legacy `.xls` is an MS-CFB container, parsed by the `cfb` crate
//! and classified into a typed error. The input always starts with the OLE signature, so every run
//! reaches the container parser rather than the zip reader `xlsx_import` covers. Nothing may
//! panic, and the byte budget bounds what the parser is given.
#![no_main]

use libfuzzer_sys::fuzz_target;
use rustsheet::{LoadOptions, Workbook};

/// MS-CFB 2.2 `_abSig`.
const OLE_SIGNATURE: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

fuzz_target!(|data: &[u8]| {
    let opts = LoadOptions {
        recalc: false,
        max_uncompressed_bytes: 16 << 20,
        ..LoadOptions::default()
    };
    let bytes = if data.starts_with(&OLE_SIGNATURE) {
        data.to_vec()
    } else {
        [&OLE_SIGNATURE[..], data].concat()
    };
    // Every OLE file is refused today; only that it returns matters.
    let _ = Workbook::from_xlsx_bytes(&bytes, &opts);
});
