//! The source-preserving export (SPEC-0034): a workbook loaded from a package is saved by keeping
//! that package wherever nothing changed, instead of rebuilding every part from the model.
//!
//! Three tiers: `Verbatim` when the live model equals the pristine re-import of the source,
//! `Patched` when only cells changed (`patch`, T-4560), `Regenerated` otherwise, which keeps the
//! source's `PivotChart` drawings (`pivot_drawings`, T-4950).

use std::io::{Cursor, Seek, Write};

use rustsheet_engine::Model;

use super::report::{self, SaveReport};
use super::save_regenerated;
use crate::error::XlsxError;
use crate::import::ImportLimits;

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
/// keeping the source's `PivotChart` drawings where it can (T-4950). The report says which tier
/// wrote the package and which source parts it left out (SPEC-0033 D7, T-4562); `limits` are the
/// ones `source` was loaded with, which bound the report's own read of it.
///
/// # Errors
/// When writing fails, or the regenerating export does.
pub fn save_preserving<W: Write + Seek>(
    model: &Model,
    pristine: &Model,
    source: &[u8],
    limits: &ImportLimits,
    mut writer: W,
) -> Result<(W, SaveReport), XlsxError> {
    if model.workbook == pristine.workbook {
        writer.write_all(source)?;
        return Ok((writer, SaveReport::new(SaveTier::Verbatim, Vec::new())));
    }
    let reason = match super::patch::patch(model, pristine, source)? {
        Ok((bytes, sheets)) => {
            writer.write_all(&bytes)?;
            let dropped = report::patched(source, &bytes, limits);
            return Ok((
                writer,
                SaveReport::new(SaveTier::Patched { sheets }, dropped),
            ));
        }
        Err(reason) => reason,
    };
    // Written to memory first: the report compares the saved package's own inventory with the
    // source's (SPEC-0034 D5.1, T-4999).
    let (buffer, kept) = save_regenerated(model, pristine, source, Cursor::new(Vec::new()))?;
    let bytes = buffer.into_inner();
    writer.write_all(&bytes)?;
    let dropped = report::regenerated(source, &bytes, &kept, limits);
    Ok((
        writer,
        SaveReport::new(SaveTier::Regenerated { reason }, dropped),
    ))
}
