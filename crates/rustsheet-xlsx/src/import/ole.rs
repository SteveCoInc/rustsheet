//! What an OLE compound file (MS-CFB) handed to the xlsx importer holds (SPEC-0031 D1, T-4264).
//!
//! A file that starts with the OLE signature is not a zip package. It is a password-protected
//! xlsx (ECMA-376 Part 2 / MS-OFFCRYPTO: `EncryptionInfo` and `EncryptedPackage` streams), a
//! legacy binary workbook (BIFF: a `Workbook` or, before BIFF8, `Book` stream) or something else.
//! [`classify_ole_compound_file`] names which one it is, so callers get a typed error;
//! [`decrypt_ole_package`] reads the two streams of an encrypted xlsx and decrypts it (T-4265).
//!
//! The container is untrusted (CLAUDE.md directive 5). It is parsed by the `cfb` crate over the
//! bytes in memory, which the caller has already bounded by `max_uncompressed_bytes`. `cfb` parses
//! the header, allocation tables and directory eagerly and rejects broken sector chains and
//! directory trees with an error. Classification reads no stream: it looks only at the names,
//! kinds and declared lengths of the root storage's entries. Decryption reads the two encrypted
//! xlsx streams, each bounded before it is read (SPEC-0031 D1).

use std::io::{Cursor, Read};

use super::encryption::{decrypt_package, MAX_ENCRYPTION_INFO_BYTES};
use super::package::ImportLimits;
use crate::error::XlsxError;

/// A parsed container over the bytes in memory.
type Container<'a> = cfb::CompoundFile<Cursor<&'a [u8]>>;

/// Most directory entries a container may hold (SPEC-0031 D1). An encrypted xlsx has about ten,
/// an `.xls` a few dozen; the cap bounds the importer's walk of the directory. It does not bound
/// `cfb`'s own eager parse of it when the container is opened: the byte budget does.
pub(crate) const MAX_DIRECTORY_ENTRIES: usize = 4_096;

/// The error that refuses `bytes`, an OLE compound file (it starts with the OLE signature):
/// [`XlsxError::EncryptedPackage`], [`XlsxError::LegacyBinaryWorkbook`] or, for any other or
/// unreadable container, [`XlsxError::OleCompoundFile`]. A container larger than
/// `limits.max_uncompressed_bytes`, or one whose `EncryptedPackage` declares more, is the
/// uncompressed-bytes [`XlsxError::Limit`]: decrypting it would inflate past the budget.
#[must_use]
pub fn classify_ole_compound_file(bytes: &[u8], limits: &ImportLimits) -> XlsxError {
    match open_container(bytes, limits) {
        Ok(file) => classify(&file, limits),
        Err(error) => error,
    }
}

/// The container in `bytes`, bounded by the byte budget and [`MAX_DIRECTORY_ENTRIES`].
fn open_container<'a>(bytes: &'a [u8], limits: &ImportLimits) -> Result<Container<'a>, XlsxError> {
    if u64::try_from(bytes.len()).map_or(true, |n| n > limits.max_uncompressed_bytes) {
        return Err(limits.bytes_exceeded());
    }
    // A malformed container (bad header, sector chain cycle, chain past the end, broken
    // directory tree) is still an OLE file, just not one anybody can read: the same error as an
    // unknown one.
    let file =
        cfb::CompoundFile::open(Cursor::new(bytes)).map_err(|_| XlsxError::OleCompoundFile)?;
    if file.walk().nth(MAX_DIRECTORY_ENTRIES).is_some() {
        return Err(XlsxError::OleCompoundFile);
    }
    Ok(file)
}

/// The error [`classify_ole_compound_file`] gives for an opened container.
fn classify(file: &Container<'_>, limits: &ImportLimits) -> XlsxError {
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

/// The zip package of the password-protected xlsx in `bytes`, an OLE compound file, decrypted
/// with `password` (SPEC-0031 D2).
///
/// `EncryptionInfo` is read up to 1 MiB and `EncryptedPackage` up to
/// `limits.max_uncompressed_bytes`; one byte more is refused before the rest is read. A stream
/// that ends before its declared size (`cfb` checks a sector chain against the size only when it
/// reads it) is malformed, not a shorter package.
///
/// # Errors
/// The error [`classify_ole_compound_file`] gives for a container that is not an encrypted xlsx,
/// and otherwise those of [`decrypt_package`].
pub fn decrypt_ole_package(
    bytes: &[u8],
    password: &str,
    limits: &ImportLimits,
) -> Result<Vec<u8>, XlsxError> {
    let mut file = open_container(bytes, limits)?;
    let not_encrypted = classify(&file, limits);
    if not_encrypted != XlsxError::EncryptedPackage {
        return Err(not_encrypted);
    }
    let info = read_stream(
        &mut file,
        "EncryptionInfo",
        MAX_ENCRYPTION_INFO_BYTES,
        || XlsxError::MalformedEncryption("EncryptionInfo is larger than 1 MiB".to_string()),
    )?;
    let package = read_stream(
        &mut file,
        "EncryptedPackage",
        limits.max_uncompressed_bytes,
        || limits.bytes_exceeded(),
    )?;
    decrypt_package(&info, &package, password, limits)
}

/// The content of the root storage's stream `name` (compared case-insensitively), at most
/// `limit` bytes; `over` is the error for a longer one.
fn read_stream(
    file: &mut Container<'_>,
    name: &str,
    limit: u64,
    over: impl Fn() -> XlsxError,
) -> Result<Vec<u8>, XlsxError> {
    let short = || XlsxError::MalformedEncryption(format!("{name} is shorter than it declares"));
    let entry = file
        .read_root_storage()
        .find(|e| e.is_stream() && e.name().eq_ignore_ascii_case(name))
        .ok_or_else(short)?;
    if entry.len() > limit {
        return Err(over());
    }
    let mut stream = file.open_stream(entry.path()).map_err(|_| short())?;
    let mut content = Vec::new();
    stream
        .by_ref()
        .take(limit.saturating_add(1))
        .read_to_end(&mut content)
        .map_err(|_| short())?;
    if u64::try_from(content.len()).map_or(true, |n| n > limit) {
        return Err(over());
    }
    if u64::try_from(content.len()).ok() != Some(entry.len()) {
        return Err(short());
    }
    Ok(content)
}
