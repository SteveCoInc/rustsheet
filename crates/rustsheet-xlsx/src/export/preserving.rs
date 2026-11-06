//! The source-preserving export (SPEC-0034): a workbook loaded from a package is saved by keeping
//! that package wherever nothing changed, instead of rebuilding every part from the model.
//!
//! This slice (T-4559) has two tiers: `Verbatim` when the live model equals the pristine
//! re-import of the source, and `Regenerated` otherwise. Patching changed cells (T-4560) fills
//! in the tier between them.

use std::io::{Seek, Write};

use rustsheet_engine::Model;

use super::save_xlsx_to_writer;
use crate::error::XlsxError;

/// How a save wrote the package (SPEC-0034 D4).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SaveTier {
    /// Every part is the source's, byte for byte.
    Verbatim,
    /// Rebuilt from the model, for this reason.
    Regenerated {
        /// Why the source could not be kept.
        reason: String,
    },
}

/// Writes `model`, keeping `source` where SPEC-0034 allows. `pristine` is `source` imported again
/// the way `model` was first loaded, so comparing the two typed models says what changed since the
/// load (D2). An unchanged workbook is the source's bytes, untouched; any change regenerates, as
/// [`save_xlsx_to_writer`] always did.
///
/// # Errors
/// When writing fails, or the regenerating export does.
pub fn save_preserving<W: Write + Seek>(
    model: &Model,
    pristine: &Model,
    source: &[u8],
    mut writer: W,
) -> Result<(W, SaveTier), XlsxError> {
    if model.workbook == pristine.workbook {
        writer.write_all(source)?;
        return Ok((writer, SaveTier::Verbatim));
    }
    let writer = save_xlsx_to_writer(model, writer)?;
    // Patching changed cells into the source is T-4560 (SPEC-0034 D3).
    let reason = "the workbook changed since it was loaded".to_string();
    Ok((writer, SaveTier::Regenerated { reason }))
}
