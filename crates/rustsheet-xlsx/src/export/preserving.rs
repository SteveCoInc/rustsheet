//! The source-preserving export (SPEC-0034): a workbook loaded from a package is saved by keeping
//! that package wherever nothing changed, instead of rebuilding every part from the model.
//!
//! Three tiers: `Verbatim` when the live model equals the pristine re-import of the source,
//! `Patched` when only cells changed (`patch`, T-4560), `Regenerated` otherwise, which keeps the
//! source's `PivotChart` drawings (`pivot_drawings`, T-4950).

use std::io::{Seek, Write};

use rustsheet_engine::Model;

use super::save_regenerated;
use crate::error::XlsxError;

/// How a save wrote the package (SPEC-0034 D4).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SaveTier {
    /// Every part is the source's, byte for byte.
    Verbatim,
    /// The source with these sheets' changed cells patched in, and new shared strings appended;
    /// every other part byte for byte (SPEC-0034 D3).
    Patched {
        /// The sheets whose cells were patched.
        sheets: Vec<String>,
    },
    /// Rebuilt from the model, for this reason.
    Regenerated {
        /// Why the source could not be kept.
        reason: String,
    },
}

/// Writes `model`, keeping `source` where SPEC-0034 allows. `pristine` is `source` imported again
/// the way `model` was first loaded, so comparing the two typed models says what changed since the
/// load (D2). An unchanged workbook is the source's bytes, untouched; a change the patch cannot
/// write regenerates, as [`save_xlsx_to_writer`](super::save_xlsx_to_writer) always did,
/// keeping the source's `PivotChart` drawings where it can (T-4950).
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
    let reason = match super::patch::patch(model, pristine, source)? {
        Ok((bytes, sheets)) => {
            writer.write_all(&bytes)?;
            return Ok((writer, SaveTier::Patched { sheets }));
        }
        Err(reason) => reason,
    };
    let writer = save_regenerated(model, pristine, source, writer)?;
    Ok((writer, SaveTier::Regenerated { reason }))
}
