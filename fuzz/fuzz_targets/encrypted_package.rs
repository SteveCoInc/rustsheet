//! Decrypts an untrusted password-protected xlsx (SPEC-0031 D2, D3, D5; T-4265).
//!
//! The input is an OLE compound file handed to `decrypt_ole_package`: the container, the
//! `EncryptionInfo` descriptor (Agile XML or the Standard binary header) and the
//! `EncryptedPackage` stream are all untrusted. Nothing may panic, and every size, count and
//! the spin count are bounded before they are used. The seeds are the encrypted fixtures of
//! crates/rustsheet-xlsx/tests/encrypted (`seed_corpus.py`), so mutations start from containers
//! that reach the descriptor parser and the key derivation. The spin-count limit is kept low so
//! a mutated `spinCount` refuses fast instead of spending each run on hashing.
#![no_main]

use libfuzzer_sys::fuzz_target;
use rustsheet_xlsx::import::{decrypt_ole_package, ImportLimits};

fuzz_target!(|data: &[u8]| {
    let limits = ImportLimits {
        max_uncompressed_bytes: 16 << 20,
        max_spin_count: 10_000,
        ..ImportLimits::default()
    };
    // The fixtures' own passwords, so a mutation that keeps the verifier reaches the package.
    for password in ["Pässwörd🔑", "tika"] {
        let _ = decrypt_ole_package(data, password, &limits);
    }
});
