//! A loaded workbook's source package, kept so a save can write back what did not change
//! (SPEC-0034 D1).

use std::sync::Arc;

use rustsheet_engine::clock::Clock;
use rustsheet_engine::Model;
use rustsheet_xlsx::error::XlsxError;
use rustsheet_xlsx::import::ImportLimits;

use crate::error::LoadError;
use crate::workbook::{engine_load_error, import_load_error};

/// A loaded workbook's source package and the options it was imported with (SPEC-0034 D1).
#[derive(Debug)]
pub(crate) struct Source {
    /// The package, after decryption for a password-protected file. Bounded by the load's
    /// `max_uncompressed_bytes`.
    pub(crate) bytes: Arc<[u8]>,
    pub(crate) name: String,
    pub(crate) locale: String,
    pub(crate) timezone: String,
    pub(crate) language: String,
    pub(crate) clock: Clock,
    pub(crate) limits: ImportLimits,
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
