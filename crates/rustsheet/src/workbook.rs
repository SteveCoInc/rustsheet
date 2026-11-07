//! [`Workbook`]: the facade over the engine `Model` (open/new/save and the read path).

use std::sync::Arc;

use rustsheet_engine::clock::Clock;
use rustsheet_engine::types::DateSystem;
use rustsheet_engine::Model;
use rustsheet_xlsx::error::XlsxError;
use rustsheet_xlsx::import::ImportLimits;
use zeroize::Zeroizing;

use crate::error::{LoadError, RefError, UnsupportedKind};
use crate::refs::{
    parse_cell, parse_range, resolve_sheet_by_name, CellRef, RangeRef, SheetId, Target,
};
use crate::source::Source;
use crate::value::Value;
mod active_sheet;
mod calculation;
#[cfg(feature = "harness_row_insert")]
mod harness;
mod load;
mod read;
mod save;

/// Options for [`Workbook::new`] and the limits shared by all reads.
#[derive(Clone, Debug)]
pub struct WorkbookOptions {
    /// Most cells a single [`Workbook::values`] call may return (checked against the
    /// rectangle area before anything is allocated).
    pub max_read_cells: u64,
    /// Engine locale id (`"en"` is the engine's en-US locale).
    pub locale: String,
    /// IANA timezone name used by date functions.
    pub timezone: String,
    /// Engine language id for function names and error literals.
    pub language: String,
    /// Date system of a new workbook ([`Workbook::new`]). A loaded workbook keeps the one its
    /// xlsx stores (`workbookPr@date1904`), so this field does not apply to loading.
    pub date_system: DateSystem,
    /// The clock NOW, TODAY and the other readers of the current time use: the system time
    /// (the default) or a fixed instant (SPEC-0008). Applied before the first recalc, also when
    /// loading.
    pub clock: Clock,
    /// The workbook's file name without its extension (`Budget` for `Budget.xlsx`), which
    /// `CELL("address")` of a reference on another sheet
    /// reports as `[Budget.xlsx]`. `None` (the default) is an unnamed workbook, for which that
    /// `CELL("address")` is `#N/IMPL!`: Excel's form for an unsaved workbook is unprobed (T-2042). Applies to loading
    /// too.
    pub name: Option<String>,
}

impl Default for WorkbookOptions {
    fn default() -> Self {
        Self {
            max_read_cells: 1_000_000,
            locale: "en".to_string(),
            timezone: "UTC".to_string(),
            language: "en".to_string(),
            date_system: DateSystem::default(),
            clock: Clock::system(),
            name: None,
        }
    }
}

/// The password of a password-protected xlsx (SPEC-0031 D4).
///
/// Its text is zeroized when it is dropped, and its `Debug` prints `Password(***)`. It implements
/// neither `Display` nor `Serialize`, so neither it nor a [`LoadOptions`] holding it can print or
/// store the password.
#[derive(Clone)]
pub struct Password(Zeroizing<String>);

impl Password {
    /// Wraps `password`, taking ownership of its buffer (no other copy is made).
    #[must_use]
    pub fn new(password: impl Into<String>) -> Self {
        Self(Zeroizing::new(password.into()))
    }

    /// The password text, for the decryption only.
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Password(***)")
    }
}

/// Options for [`Workbook::open`] and [`Workbook::from_xlsx_bytes`].
///
/// The size limits are enforced on the bytes actually inflated, before any part is parsed.
///
/// The struct is not `#[non_exhaustive]`: build it with `..LoadOptions::default()` so a field added
/// later (as `password` and `max_spin_count` were) does not break the caller.
#[derive(Clone, Debug)]
pub struct LoadOptions {
    /// Recalculate once after loading.
    pub recalc: bool,
    /// Most bytes all zip entries may declare in total. A password-protected xlsx is bounded by it
    /// both as a file and as the zip it decrypts to.
    pub max_uncompressed_bytes: u64,
    /// Most entries the zip directory may list.
    pub max_zip_entries: u32,
    /// The password of a password-protected xlsx; `None` (the default) refuses one with
    /// [`LoadError::PasswordRequired`]. Ignored for a file that is not encrypted. Only this password
    /// is tried, so a load runs at most one key derivation.
    pub password: Option<Password>,
    /// Most key derivation rounds (`spinCount`) a password-protected xlsx may ask for, checked
    /// before any hashing; a file asking for more is [`LoadError::Options`]. The default,
    /// 1,000,000, refuses files Excel opens (it allows up to 10,000,000, several seconds of CPU
    /// any untrusted file could demand): a DEVIATION, docs/excel-semantics/xlsx-storage.md. Raise
    /// it, up to 10,000,000, only for files you trust.
    pub max_spin_count: u32,
    /// Options of the loaded workbook (read limit, locale, timezone, language).
    pub workbook: WorkbookOptions,
}

impl Default for LoadOptions {
    fn default() -> Self {
        let import = ImportLimits::default();
        Self {
            recalc: true,
            max_uncompressed_bytes: import.max_uncompressed_bytes,
            max_zip_entries: import.max_zip_entries,
            password: None,
            max_spin_count: import.max_spin_count,
            workbook: WorkbookOptions::default(),
        }
    }
}

/// Options for saving; reserved for future use.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct SaveOptions {}

/// A worksheet's identity and position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetMeta {
    /// Stable identifier.
    pub id: SheetId,
    /// Current name.
    pub name: String,
    /// Current 0-based position.
    pub index: u32,
}

/// A cell's value together with its stored input and number format.
#[derive(Clone, Debug, PartialEq)]
pub struct CellInfo {
    /// The computed value.
    pub value: Value,
    /// The formula (with leading `=`), if the cell holds one.
    pub formula: Option<String>,
    /// The number format code, `None` for `General`.
    pub number_format: Option<String>,
}

/// An open workbook. Owns its engine model; reads never mutate it.
pub struct Workbook {
    pub(crate) model: Model,
    max_read_cells: u64,
    /// Whether `edit` diffs formula values into [`crate::ChangeSet::values`].
    pub(crate) track_changes: bool,
    /// The package the workbook was loaded from, kept so a save can write back what did not change
    /// (SPEC-0034 D1). `None` for a workbook created with [`Workbook::new`].
    source: Option<Arc<Source>>,
    /// Whether the first [`Workbook::recalculate`] refreshes the `refreshOnLoad` pivots.
    pub(crate) pivots_due: bool,
}

pub(crate) fn engine_load_error(message: impl std::fmt::Display) -> LoadError {
    LoadError::Format(message.to_string())
}

/// The facade error for an importer error (SPEC-0031 D4). None of them carries the password.
pub(crate) fn import_load_error(e: XlsxError) -> LoadError {
    match e {
        XlsxError::Limit { .. } => LoadError::Options(e.to_string()),
        XlsxError::EncryptedPackage => LoadError::PasswordRequired,
        XlsxError::WrongPassword => LoadError::WrongPassword,
        XlsxError::LegacyBinaryWorkbook => {
            LoadError::Unsupported(UnsupportedKind::LegacyBinaryWorkbook)
        }
        XlsxError::OleCompoundFile => LoadError::Unsupported(UnsupportedKind::OtherOleFile),
        XlsxError::UnsupportedEncryption(kind) => {
            LoadError::Unsupported(UnsupportedKind::Encryption(kind))
        }
        e => engine_load_error(e),
    }
}

/// All sheets of a model in workbook order.
pub(crate) fn sheet_metas(model: &Model) -> Vec<SheetMeta> {
    model
        .workbook
        .worksheets
        .iter()
        .enumerate()
        .map(|(i, ws)| SheetMeta {
            id: SheetId::new(ws.get_sheet_id()),
            name: ws.get_name(),
            index: u32::try_from(i).unwrap_or(u32::MAX),
        })
        .collect()
}

/// A sheet by case-insensitive name; `None` selects the first sheet.
pub(crate) fn lookup_sheet(model: &Model, name: Option<&str>) -> Option<SheetId> {
    let sheets = sheet_metas(model);
    let pairs: Vec<(SheetId, &str)> = sheets.iter().map(|s| (s.id, s.name.as_str())).collect();
    resolve_sheet_by_name(&pairs, name)
}

/// The positional index of a sheet id.
pub(crate) fn sheet_index(model: &Model, id: SheetId) -> Result<u32, RefError> {
    model
        .workbook
        .worksheets
        .iter()
        .position(|ws| ws.get_sheet_id() == id.get())
        .and_then(|i| u32::try_from(i).ok())
        .ok_or_else(|| RefError::UnknownSheet(id.get().to_string()))
}

/// A target as a rectangle, resolving A1 text against the model's sheets.
pub(crate) fn resolve_range(model: &Model, target: Target<'_>) -> Result<RangeRef, RefError> {
    match target {
        Target::Cell(c) => RangeRef::new(c.sheet, c.row, c.col, c.row, c.col),
        Target::Range(r) => Ok(r),
        Target::A1(text) => parse_range(text, |name| lookup_sheet(model, name)),
    }
}

/// Engine coordinates are 1-based `i32`; the facade's are 0-based `u32` already bounds-checked.
pub(crate) fn engine_rc(row: u32, col: u32) -> Option<(i32, i32)> {
    Some((
        i32::try_from(row).ok()?.checked_add(1)?,
        i32::try_from(col).ok()?.checked_add(1)?,
    ))
}

impl Workbook {
    /// A new workbook with one empty sheet.
    ///
    /// # Errors
    /// [`LoadError::Options`] when the locale, timezone or language is unknown.
    pub fn new(opts: &WorkbookOptions) -> Result<Self, LoadError> {
        let mut model = Model::new_empty_with_clock(
            opts.name.as_deref().unwrap_or_default(),
            &opts.locale,
            &opts.timezone,
            &opts.language,
            opts.clock,
        )
        .map_err(LoadError::Options)?;
        model.workbook.settings.date_system = opts.date_system;
        Ok(Self::from_model(model, opts.max_read_cells))
    }

    /// The workbook's date system: the one chosen at [`Workbook::new`], or the one a loaded
    /// xlsx stores.
    #[must_use]
    pub fn date_system(&self) -> DateSystem {
        self.model.workbook.settings.date_system
    }

    fn from_model(model: Model, max_read_cells: u64) -> Self {
        Self {
            model,
            max_read_cells,
            track_changes: true,
            source: None,
            pivots_due: false,
        }
    }

    /// All sheets in workbook order.
    #[must_use]
    pub fn sheets(&self) -> Vec<SheetMeta> {
        sheet_metas(&self.model)
    }

    /// Looks a sheet up by name, case-insensitively.
    ///
    /// # Errors
    /// [`RefError::UnknownSheet`] when no sheet has that name.
    pub fn sheet_id(&self, name: &str) -> Result<SheetId, RefError> {
        self.lookup_sheet(Some(name))
            .ok_or_else(|| RefError::UnknownSheet(name.to_string()))
    }

    fn lookup_sheet(&self, name: Option<&str>) -> Option<SheetId> {
        lookup_sheet(&self.model, name)
    }

    /// Parses an A1 cell reference (`Sheet1!A1`, `'My Sheet'!$B$2`); unqualified references
    /// use the first sheet.
    ///
    /// # Errors
    /// As [`parse_cell`].
    pub fn cell_ref(&self, a1: &str) -> Result<CellRef, RefError> {
        parse_cell(a1, |name| self.lookup_sheet(name))
    }

    /// Parses an A1 range reference.
    ///
    /// # Errors
    /// As [`parse_range`].
    pub fn range_ref(&self, a1: &str) -> Result<RangeRef, RefError> {
        parse_range(a1, |name| self.lookup_sheet(name))
    }

    fn resolve_range(&self, target: Target<'_>) -> Result<RangeRef, RefError> {
        resolve_range(&self.model, target)
    }

    fn sheet_index(&self, id: SheetId) -> Result<u32, RefError> {
        sheet_index(&self.model, id)
    }
}

#[cfg(test)]
mod tests;
