//! Loading a [`Workbook`] from an xlsx file or bytes (moved out of workbook.rs, T-4669).

use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use rustsheet_xlsx::import::{decrypt_ole_package, is_ole_compound_file, ImportLimits};
use zeroize::Zeroizing;

use super::{import_load_error, LoadOptions, Workbook};
use crate::error::LoadError;
use crate::source::Source;

impl Workbook {
    /// Opens an xlsx file.
    ///
    /// # Errors
    /// [`LoadError::Io`] if the file cannot be read, otherwise as [`Self::from_xlsx_bytes`].
    pub fn open(path: impl AsRef<Path>, opts: &LoadOptions) -> Result<Self, LoadError> {
        // Bound the read itself (special files report length 0): a larger file cannot be a
        // workbook within `max_uncompressed_bytes` anyway.
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(opts.max_uncompressed_bytes.saturating_add(1))
            .read_to_end(&mut bytes)?;
        if u64::try_from(bytes.len()).map_or(true, |n| n > opts.max_uncompressed_bytes) {
            return Err(LoadError::Options(
                "file larger than max_uncompressed_bytes".into(),
            ));
        }
        Self::from_xlsx_bytes(&bytes, opts)
    }

    /// Loads an xlsx workbook from memory. Never touches the filesystem.
    ///
    /// A password-protected xlsx is decrypted with [`LoadOptions::password`] first (ECMA-376 Agile
    /// and Standard encryption, SPEC-0031); the password is not kept on the workbook.
    ///
    /// # Errors
    /// - [`LoadError::Options`] when the zip exceeds the limits (including
    ///   [`LoadOptions::max_spin_count`]) or the locale is unknown;
    /// - [`LoadError::PasswordRequired`] / [`LoadError::WrongPassword`] for a password-protected
    ///   xlsx opened without its password;
    /// - [`LoadError::Unsupported`] for a legacy `.xls`, another OLE file or an encryption kind
    ///   not read;
    /// - [`LoadError::Format`] when the bytes are not a valid workbook (including an encrypted
    ///   package that is malformed or fails its integrity check).
    pub fn from_xlsx_bytes(bytes: &[u8], opts: &LoadOptions) -> Result<Self, LoadError> {
        let w = &opts.workbook;
        // The importer enforces the same limits on the parts it reads, so a caller raising
        // them here is not stopped by the importer's defaults.
        let limits = ImportLimits {
            max_uncompressed_bytes: opts.max_uncompressed_bytes,
            max_zip_entries: opts.max_zip_entries,
            max_spin_count: opts.max_spin_count,
            ..ImportLimits::default()
        };
        // With a password, an OLE file is decrypted here (one key derivation, SPEC-0031 D3); one
        // that is not an encrypted xlsx gets the error the importer would give it. Without one,
        // the importer refuses it as encrypted, which is `PasswordRequired`.
        let decrypted = match &opts.password {
            Some(password) if is_ole_compound_file(bytes) => Some(Zeroizing::new(
                decrypt_ole_package(bytes, password.expose(), &limits)
                    .map_err(import_load_error)?,
            )),
            _ => None,
        };
        let was_decrypted = decrypted.is_some();
        // The retained package is wiped when dropped: for an encrypted file it is the plaintext
        // (SPEC-0031), so it is moved in, never copied.
        let package = decrypted.unwrap_or_else(|| Zeroizing::new(bytes.to_vec()));
        let source = Source {
            bytes: package,
            name: w.name.clone().unwrap_or_default(),
            locale: w.locale.clone(),
            timezone: w.timezone.clone(),
            language: w.language.clone(),
            clock: w.clock,
            limits,
        };
        let model = source.import(was_decrypted)?;
        let mut workbook = Self::from_model(model, w.max_read_cells);
        workbook.source = Some(Arc::new(source));
        let caches = &workbook.model.workbook.pivot_caches;
        workbook.pivots_due = caches.iter().any(|c| c.refresh_on_load);
        if opts.recalc {
            workbook.recalculate();
        }
        Ok(workbook)
    }

    /// Pins the workbook clock (`NOW`, `TODAY`) to the timestamp in the workbook's `METADATA`
    /// sheet, as Excel-authored test workbooks carry it. Returns whether it pinned: a workbook
    /// without a timestamp, or with one outside Excel's calendar, keeps its clock. Harness only
    /// (SPEC-0008); it does not recalculate.
    #[cfg(feature = "metadata_clock")]
    pub fn pin_clock_from_metadata(&mut self) -> bool {
        rustsheet_xlsx::metadata_clock::pin_clock_from_metadata(&mut self.model)
    }
}
