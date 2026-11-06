//! A loaded workbook's source package, kept so a save can write back what did not change
//! (SPEC-0034 D1).

use std::fmt;

use rustsheet_engine::clock::Clock;
use rustsheet_engine::Model;
use rustsheet_xlsx::error::XlsxError;
use rustsheet_xlsx::import::ImportLimits;
use zeroize::Zeroizing;

use crate::error::LoadError;
use crate::workbook::{engine_load_error, import_load_error};

/// A loaded workbook's source package and the options it was imported with (SPEC-0034 D1).
pub(crate) struct Source {
    /// The package, after decryption for a password-protected file: the plaintext then, so it is
    /// zeroized when dropped (SPEC-0031). A copy of the caller's bytes otherwise, the size of the
    /// input; a decrypted package is bounded by the load's `max_uncompressed_bytes`.
    pub(crate) bytes: Zeroizing<Vec<u8>>,
    pub(crate) name: String,
    pub(crate) locale: String,
    pub(crate) timezone: String,
    pub(crate) language: String,
    pub(crate) clock: Clock,
    pub(crate) limits: ImportLimits,
}

/// `Debug` never shows the package: for an encrypted file it is the plaintext.
impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Source")
            .field("bytes", &format_args!("<{} bytes>", self.bytes.len()))
            .field("name", &self.name)
            .field("locale", &self.locale)
            .field("timezone", &self.timezone)
            .field("language", &self.language)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

impl Source {
    /// Imports the package into a model, as [`Workbook::from_xlsx_bytes`] does (one definition
    /// for the load and the save's pristine copy). `decrypted` says the bytes came out of an
    /// encrypted file, whose payload must not itself be an OLE file.
    pub(crate) fn import(&self, decrypted: bool) -> Result<Model, LoadError> {
        let book = rustsheet_xlsx::import::load_from_xlsx_bytes_with_limits(
            &self.bytes,
            &self.name,
            &self.locale,
            &self.timezone,
            &self.limits,
        )
        .map_err(|e| match e {
            // A decrypted package that is itself an OLE file is not a workbook, whatever it holds.
            XlsxError::EncryptedPackage
            | XlsxError::LegacyBinaryWorkbook
            | XlsxError::OleCompoundFile
                if decrypted =>
            {
                engine_load_error("the decrypted package is not a zip")
            }
            e => import_load_error(e),
        })?;
        Model::from_workbook_with_clock(book, &self.language, self.clock).map_err(LoadError::Format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_shows_the_package() {
        let source = Source {
            bytes: Zeroizing::new(b"secret plaintext".to_vec()),
            name: "w".into(),
            locale: "en".into(),
            timezone: "UTC".into(),
            language: "en".into(),
            clock: Clock::default(),
            limits: ImportLimits::default(),
        };
        let text = format!("{source:?}");
        assert!(!text.contains("secret"), "{text}");
        assert!(text.contains("<16 bytes>"), "{text}");
    }
}
