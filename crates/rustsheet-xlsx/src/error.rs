use std::io;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;
use zip::result::ZipError;

/// A resource bound the importer enforces on untrusted workbooks (CLAUDE.md directive 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportLimit {
    /// Entries listed in the zip directory.
    ZipEntries,
    /// Bytes inflated from the parts the importer reads, in total.
    UncompressedBytes,
    /// Characters in one cell text or shared string.
    TextLength,
    /// Entity references beyond the five predefined ones and character references: xlsx parts
    /// declare no entities, so none is ever expanded (no "billion laughs").
    EntityExpansion,
    /// Rounds of an encrypted package's password key derivation (`spinCount`, SPEC-0031 D3).
    SpinCount,
}

impl std::fmt::Display for ImportLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ImportLimit::ZipEntries => "zip entries",
            ImportLimit::UncompressedBytes => "uncompressed bytes",
            ImportLimit::TextLength => "characters in one text",
            ImportLimit::EntityExpansion => "custom entity expansions",
            ImportLimit::SpinCount => "key derivation rounds",
        })
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum XlsxError {
    #[error("I/O Error: {0}")]
    IO(String),
    #[error("Zip Error: {0}")]
    Zip(String),
    #[error("XML Error: {0}")]
    Xml(String),
    #[error("{0}")]
    Workbook(String),
    #[error("Evaluation Error: {}", .0.join("; "))]
    Evaluation(Vec<String>),
    #[error("Comparison Error: {0}")]
    Comparison(String),
    #[error("Not Implemented Error: {0}")]
    NotImplemented(String),
    /// The workbook exceeds an import resource limit.
    #[error("Limit Error: more than {max} {limit}")]
    Limit { limit: ImportLimit, max: u64 },
    /// The file is an OLE compound file, not a zip package, and holds neither an encrypted xlsx
    /// nor a binary workbook, or its container is malformed (decision 0279, SPEC-0031 D1).
    #[error("OLE Compound File Error: the file is an OLE compound file that holds no workbook RustSheet can read, not an xlsx package")]
    OleCompoundFile,
    /// The file is a password-protected xlsx: an OLE compound file holding the ECMA-376
    /// `EncryptionInfo` and `EncryptedPackage` streams. Decryption is not implemented yet
    /// (SPEC-0031 D2, T-4265).
    #[error("Encrypted Workbook Error: the file is a password-protected xlsx, which cannot be opened yet")]
    EncryptedPackage,
    /// The file is a legacy binary Excel workbook (`.xls`, BIFF): an OLE compound file with a
    /// `Workbook` or `Book` stream. RustSheet reads only xlsx (SPEC-0031, "The .xls plan").
    #[error("Legacy Binary Workbook Error: the file is a binary Excel workbook (.xls); save it as .xlsx to open it")]
    LegacyBinaryWorkbook,
    /// The password does not open the encrypted package (SPEC-0031 D2), or is longer than the
    /// 255 UTF-16 code units a password may have (E3).
    #[error("Wrong Password Error: the password does not open this workbook")]
    WrongPassword,
    /// The package is encrypted in a way RustSheet does not read (SPEC-0031 D2): Extensible
    /// encryption, RC4, a certificate key encryptor, a cipher other than AES-CBC, or a
    /// `spinCount` past the format's 10,000,000. The text names the kind, never the password.
    #[error(
        "Unsupported Encryption Error: the workbook uses an encryption RustSheet cannot read: {0}"
    )]
    UnsupportedEncryption(String),
    /// The encryption data breaks the format (SPEC-0031 D3), or an Agile package failed its
    /// integrity check (D5). The text says which, never the password.
    #[error("Malformed Encryption Error: {0}")]
    MalformedEncryption(String),
}

impl From<io::Error> for XlsxError {
    fn from(error: io::Error) -> Self {
        XlsxError::IO(error.to_string())
    }
}

impl From<ZipError> for XlsxError {
    fn from(error: ZipError) -> Self {
        XlsxError::Zip(error.to_string())
    }
}

impl From<ParseIntError> for XlsxError {
    fn from(error: ParseIntError) -> Self {
        XlsxError::Xml(error.to_string())
    }
}

impl From<ParseFloatError> for XlsxError {
    fn from(error: ParseFloatError) -> Self {
        XlsxError::Xml(error.to_string())
    }
}

impl From<quick_xml::Error> for XlsxError {
    fn from(error: quick_xml::Error) -> Self {
        XlsxError::Xml(error.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for XlsxError {
    fn from(error: quick_xml::events::attributes::AttrError) -> Self {
        XlsxError::Xml(error.to_string())
    }
}

impl XlsxError {
    pub fn user_message(&self) -> String {
        match &self {
            XlsxError::IO(_)
            | XlsxError::Workbook(_)
            | XlsxError::Limit { .. }
            | XlsxError::OleCompoundFile
            | XlsxError::EncryptedPackage
            | XlsxError::LegacyBinaryWorkbook
            | XlsxError::WrongPassword
            | XlsxError::UnsupportedEncryption(_)
            | XlsxError::MalformedEncryption(_) => self.to_string(),
            XlsxError::Zip(_) | XlsxError::Xml(_) => {
                "IronCalc can only open workbooks created by Microsoft Excel. \
                Can you open this file with Excel, save it to a new file, \
                and then open that new file with IronCalc? If you've already tried this, \
                then send this workbook to support@ironcalc.com and our engineering team \
                will work with you to fix the issue."
                    .to_string()
            }
            XlsxError::NotImplemented(error) => format!(
                "IronCalc cannot open this workbook due to the following unsupported features: \
                {error}. You can either re-implement these parts of your workbook using features \
                supported by IronCalc, or you can send this workbook to support@ironcalc.com \
                and our engineering team will work with you to fix the issue.",
            ),
            XlsxError::Evaluation(errors) => format!(
                "IronCalc could not evaluate this workbook without errors. This may indicate a bug or missing feature \
                in the IronCalc spreadsheet calculation engine. Please contact support@ironcalc.com, share the entirety \
                of this error message and the relevant workbook, and we will work with you to resolve the issue. \
                Detailed error message:\n{}",
                errors.join("\n")
            ),
            XlsxError::Comparison(error) => format!(
                "IronCalc produces different results when evaluating the workbook \
                than those already present in the workbook. This may indicate a bug or missing \
                feature in the IronCalc spreadsheet calculation engine. Please contact \
                support@ironcalc.com, share the entirety of this error message and the relevant \
                workbook, and we will work with you to resolve the issue. \
                Detailed error message:\n{error}"
            ),
        }
    }
}
