//! Saving a [`Workbook`] as xlsx (moved out of workbook.rs, T-4669).

use std::io::Cursor;
use std::path::Path;

use rustsheet_xlsx::error::XlsxError;
use rustsheet_xlsx::export::SaveReport;

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
    pub fn to_xlsx_bytes(&self, opts: SaveOptions) -> Result<Vec<u8>, SaveError> {
        self.to_xlsx_bytes_with_report(opts).map(|(bytes, _)| bytes)
    }

    /// [`Self::to_xlsx_bytes`], also saying how the package was written and what it left out
    /// (SPEC-0033 D7, SPEC-0034 D4).
    ///
    /// # Errors
    /// As [`Self::to_xlsx_bytes`].
    pub fn to_xlsx_bytes_with_report(
        &self,
        _opts: SaveOptions,
    ) -> Result<(Vec<u8>, SaveReport), SaveError> {
        let engine = |e: XlsxError| SaveError::Engine(e.to_string());
        let Some(source) = &self.source else {
            return rustsheet_xlsx::export::save_xlsx_to_writer(
                &self.model,
                Cursor::new(Vec::new()),
            )
            .map(|w| (w.into_inner(), SaveReport::without_source()))
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
            &source.limits,
            Cursor::new(Vec::new()),
        )
        .map(|(w, report)| (w.into_inner(), report))
        .map_err(engine)
    }

    /// Writes an xlsx file, replacing an existing one.
    ///
    /// The file is never encrypted, also for a workbook opened from a password-protected one.
    ///
    /// # Errors
    /// [`SaveError::Io`] if the file cannot be written, otherwise as [`Self::to_xlsx_bytes`].
    pub fn save_xlsx(&self, path: impl AsRef<Path>, opts: SaveOptions) -> Result<(), SaveError> {
        self.save_xlsx_with_report(path, opts).map(|_| ())
    }

    /// [`Self::save_xlsx`], also returning the [`SaveReport`].
    ///
    /// # Errors
    /// As [`Self::save_xlsx`].
    pub fn save_xlsx_with_report(
        &self,
        path: impl AsRef<Path>,
        opts: SaveOptions,
    ) -> Result<SaveReport, SaveError> {
        let (bytes, report) = self.to_xlsx_bytes_with_report(opts)?;
        std::fs::write(path, bytes)?;
        Ok(report)
    }
}
