//! What an OLE compound file (MS-CFB) handed to the xlsx importer holds (SPEC-0031 D1, T-4264).
//!
//! A file that starts with the OLE signature is not a zip package. It is a password-protected
//! xlsx (ECMA-376 Part 2 / MS-OFFCRYPTO: `EncryptionInfo` and `EncryptedPackage` streams), a
//! legacy binary workbook (BIFF: a `Workbook` or, before BIFF8, `Book` stream) or something else.
//! None is read yet; this module only names which one it is, so callers get a typed error.
//!
//! The container is untrusted (CLAUDE.md directive 5). It is parsed by the `cfb` crate over the
//! bytes in memory, which the caller has already bounded by `max_uncompressed_bytes`. `cfb` parses
//! the header, allocation tables and directory eagerly and rejects broken sector chains and
//! directory trees with an error. No stream is read here: classification looks only at the
//! names, kinds and declared lengths of the root storage's entries.

use std::io::Cursor;

use super::package::ImportLimits;
use crate::error::XlsxError;

/// Most directory entries a container may hold (SPEC-0031 D1). An encrypted xlsx has about ten,
/// an `.xls` a few dozen; the cap keeps a crafted directory from being walked at length.
pub(crate) const MAX_DIRECTORY_ENTRIES: usize = 4_096;

/// The error that refuses `bytes`, an OLE compound file (it starts with the OLE signature):
/// [`XlsxError::EncryptedPackage`], [`XlsxError::LegacyBinaryWorkbook`] or, for any other or
/// unreadable container, [`XlsxError::OleCompoundFile`]. A container larger than
/// `limits.max_uncompressed_bytes`, or one whose `EncryptedPackage` declares more, is the
/// uncompressed-bytes [`XlsxError::Limit`]: decrypting it would inflate past the budget.
#[must_use]
pub fn classify_ole_compound_file(bytes: &[u8], limits: &ImportLimits) -> XlsxError {
    if u64::try_from(bytes.len()).map_or(true, |n| n > limits.max_uncompressed_bytes) {
        return limits.bytes_exceeded();
    }
    // A malformed container (bad header, sector chain cycle, chain past the end, broken
    // directory tree) is still an OLE file, just not one anybody can read: the same error as an
    // unknown one.
    let Ok(file) = cfb::CompoundFile::open(Cursor::new(bytes)) else {
        return XlsxError::OleCompoundFile;
    };
    if file.walk().nth(MAX_DIRECTORY_ENTRIES).is_some() {
        return XlsxError::OleCompoundFile;
    }
    let mut encryption_info = false;
    let mut encrypted_package = None;
    let mut binary_workbook = false;
    for entry in file.read_root_storage().filter(cfb::Entry::is_stream) {
        // Directory entry names compare case-insensitively (MS-CFB 2.6.4).
        let name = entry.name();
        if name.eq_ignore_ascii_case("EncryptionInfo") {
            encryption_info = true;
        } else if name.eq_ignore_ascii_case("EncryptedPackage") {
            encrypted_package = Some(entry.len());
        } else if name.eq_ignore_ascii_case("Workbook") || name.eq_ignore_ascii_case("Book") {
            binary_workbook = true;
        }
    }
    match (encryption_info, encrypted_package) {
        (true, Some(len)) if len > limits.max_uncompressed_bytes => limits.bytes_exceeded(),
        (true, Some(_)) => XlsxError::EncryptedPackage,
        _ if binary_workbook => XlsxError::LegacyBinaryWorkbook,
        _ => XlsxError::OleCompoundFile,
    }
}
