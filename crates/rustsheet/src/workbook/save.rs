//! Saving a [`Workbook`] as xlsx (moved out of workbook.rs, T-4669).

use std::io::Cursor;
use std::path::Path;

use rustsheet_xlsx::error::XlsxError;

use super::{SaveOptions, Workbook};
use crate::error::SaveError;

impl Workbook {
    /// Serializes to xlsx bytes.
    ///
    /// A loaded workbook unchanged since its load saves as its package, byte for byte (SPEC-0034;
    /// deciding re-imports it, about twice the model in memory). The save is never encrypted.
    ///
    /// # Errors
    /// [`SaveError::Engine`] when the engine cannot export the workbook.
    pub fn to_xlsx_bytes(&self, _opts: SaveOptions) -> Result<Vec<u8>, SaveError> {
        let engine = |e: XlsxError| SaveError::Engine(e.to_string());
        let Some(source) = &self.source else {
            return rustsheet_xlsx::export::save_xlsx_to_writer(
                &self.model,
                Cursor::new(Vec::new()),
            )
            .map(Cursor::into_inner)
            .map_err(engine);
        };
        // What changed since the load is the difference from the source imported again the same
        // way (SPEC-0034 D2). The source loaded once, so it loads again; if it somehow does not,
        // that is an engine error, never a silently different file.
        let pristine = source
            .import(false)
            .map_err(|e| SaveError::Engine(format!("the source no longer loads: {e}")))?;
        rustsheet_xlsx::export::save_preserving(
            &self.model,
            &pristine,
            &source.bytes,
            Cursor::new(Vec::new()),
        )
        .map(|(w, _tier)| w.into_inner())
        .map_err(engine)
    }

    /// Writes an xlsx file, replacing an existing one.
    ///
    /// The file is never encrypted, also for a workbook opened from a password-protected one.
    ///
    /// # Errors
    /// [`SaveError::Io`] if the file cannot be written, otherwise as [`Self::to_xlsx_bytes`].
    pub fn save_xlsx(&self, path: impl AsRef<Path>, opts: SaveOptions) -> Result<(), SaveError> {
        let bytes = self.to_xlsx_bytes(opts)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}
