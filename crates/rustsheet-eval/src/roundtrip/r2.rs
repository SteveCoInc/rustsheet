//! Scenario R2's sheet-level edits (SPEC-0033 D1, SPEC-0034 D5.7, T-5005): rename, add, delete
//! or move a sheet, save, and keep only the items the edit explains no other way. As with R1
//! (D5), every failing item of the edited save that the plain save (R0) did not have must be in
//! the edit's expected delta:
//! - the cells whose input the edit rewrote (a reference to a renamed or deleted sheet) and the
//!   cells whose cached value it changed;
//! - the workbook part's sheet list and book views, the defined names that name the sheet or
//!   its position, and the extended properties' sheet titles ([`package_explained`]);
//! - a renamed sheet's name: its items are matched under the source name ([`Rename`]), so only
//!   what the rename itself changed differs;
//! - an added sheet's items, which the source does not have;
//! - a deleted sheet's semantic items, and the tab order a delete or move shifts.
//!
//! A deleted sheet's parts are not in the delta: they pass only as `dropped`, when the save report
//! names them `OwnerDeleted` ([`super::dropped`], D7.2), which stage 5 adds (T-5004).

use std::collections::{HashMap, HashSet};

use rustsheet::{CellRef, SheetId, SheetMeta, Workbook};

use super::package::SHEET_KINDS;
use super::r1::{self, Delta};
use super::semantic::Facts;
use super::{Item, Verdict};

/// One sheet-level edit of R2 (D1). Each acts on the first sheet in tab order, as R1's edits do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SheetEdit {
    /// The first sheet is renamed [`R2_SHEET`].
    Rename,
    /// A sheet called [`R2_SHEET`] is added at the end.
    Add,
    /// The first sheet is deleted (`n/a` for a workbook of one sheet).
    Delete,
    /// The first sheet moves to the end (`n/a` for a workbook of one sheet).
    Move,
}

/// The name a renamed or added sheet gets: no workbook holds it (a case that does is `n/a`).
pub(super) const R2_SHEET: &str = "rustsheet-roundtrip-R2";

/// A renamed sheet: `from` as the source names it, `to` as the output does. The output's items
/// are keyed back under `from`, so the sheet's items are matched with the source's.
#[derive(Debug)]
pub(super) struct Rename {
    from: String,
    to: String,
}

impl Rename {
    /// An output part key under the source's name of the sheet.
    pub(super) fn back_part(&self, key: &str) -> String {
        swap_part(key, &self.to, &self.from)
    }

    /// Output facts keyed under the source's name of the sheet.
    pub(super) fn back_facts(&self, facts: Facts) -> Facts {
        swap_facts(facts, &self.to, &self.from)
    }

    /// Source facts keyed under the output's name of the sheet.
    pub(super) fn forward_facts(&self, facts: &Facts) -> Facts {
        swap_facts(facts.clone(), &self.from, &self.to)
    }
}

/// What one R2 edit did: its expected delta.
#[derive(Debug)]
pub(super) struct Structural {
    edit: SheetEdit,
    /// The edited sheet as the source names it; for [`SheetEdit::Add`], the added sheet.
    sheet: String,
    /// For a rename, the names to match the output's items under.
    pub(super) renamed: Option<Rename>,
    /// The cells whose input the edit rewrote and whose cached value it changed.
    cells: Delta,
}

impl Structural {
    /// The sheet whose parts the save report may drop as `OwnerDeleted` (a delete) or
    /// `StaleAfterStructuralEdit` (D7.2), as the source and as the output name it.
    pub(super) fn edited(&self) -> Option<super::dropped::Edited<'_>> {
        (self.edit != SheetEdit::Add).then(|| super::dropped::Edited {
            sheet: &self.sheet,
            now: self.renamed.as_ref().map_or(&self.sheet, |r| &r.to),
            deleted: self.edit == SheetEdit::Delete,
        })
    }
}

/// Applies `edit` to `workbook` (already recalculated, as R0 saves it) in one transaction.
/// Returns the expected delta, or `None` when the edit does not apply to the case (`n/a`).
///
/// # Errors
/// When the engine fails the edit.
pub(super) fn apply(
    workbook: &mut Workbook,
    edit: SheetEdit,
) -> Result<Option<Structural>, String> {
    let sheets = workbook.sheets();
    let Some(first) = sheets.iter().min_by_key(|s| s.index).cloned() else {
        return Ok(None);
    };
    let last = u32::try_from(sheets.len().saturating_sub(1)).unwrap_or(u32::MAX);
    let alone = sheets.len() < 2 && matches!(edit, SheetEdit::Delete | SheetEdit::Move);
    if alone || sheets.iter().any(|s| s.name.eq_ignore_ascii_case(R2_SHEET)) {
        return Ok(None);
    }
    let names: HashMap<SheetId, String> = sheets.iter().map(|s| (s.id, s.name.clone())).collect();
    let before = formulas(workbook, &sheets);
    workbook.set_track_changes(true);
    let edited = workbook.edit(|t| match edit {
        SheetEdit::Rename => t.rename_sheet(first.id, R2_SHEET),
        SheetEdit::Add => t.add_sheet(R2_SHEET).map(|_| ()),
        SheetEdit::Delete => t.delete_sheet(first.id),
        SheetEdit::Move => t.move_sheet(first.id, last),
    });
    let ((), changes) = edited.map_err(|e| format!("the engine failed {edit:?}: {e}"))?;
    let mut cells = Delta::default();
    r1::record(&changes, &names, false, &mut cells);
    // The engine rewrites references to a renamed or deleted sheet without reporting the cells
    // whose input it rewrote, so they are found by comparing every formula.
    let after = formulas(workbook, &workbook.sheets());
    for cell in before.keys().chain(after.keys()).collect::<HashSet<_>>() {
        if before.get(cell) != after.get(cell) {
            if let Some(name) = names.get(&cell.sheet) {
                cells.written.insert(format!("{name}!{cell}"));
            }
        }
    }
    let (sheet, renamed) = match edit {
        SheetEdit::Add => (R2_SHEET.to_string(), None),
        SheetEdit::Rename => {
            let rename = Rename {
                from: first.name.clone(),
                to: R2_SHEET.to_string(),
            };
            (first.name, Some(rename))
        }
        SheetEdit::Delete | SheetEdit::Move => (first.name, None),
    };
    Ok(Some(Structural {
        edit,
        sheet,
        renamed,
        cells,
    }))
}

/// The formula of every stored formula cell of `sheets`.
fn formulas(workbook: &Workbook, sheets: &[SheetMeta]) -> HashMap<CellRef, String> {
    let mut out = HashMap::new();
    for meta in sheets {
        for cell in workbook.stored_cells(meta.id).unwrap_or_default() {
            if let Some(formula) = workbook.cell(cell).ok().and_then(|c| c.formula) {
                out.insert(cell, formula);
            }
        }
    }
    out
}

/// Whether the expected delta of `edit` explains `item` (module doc).
pub(super) fn explained(item: &Item, edit: &Structural) -> bool {
    if r1::explained(item, &edit.cells) {
        return true;
    }
    match item.kind {
        "semantic" => fact_explained(item, edit),
        "element" | "attribute" => package_explained(item, edit.edit),
        // An added sheet's parts are new in the output.
        "part" => {
            edit.edit == SheetEdit::Add
                && item.verdict == Verdict::Added
                && sheet_of_part(&item.key) == Some(edit.sheet.as_str())
        }
        _ => false,
    }
}

/// Whether the edit explains a semantic fact: an added sheet's facts, a deleted sheet's, the tab
/// order a delete or move shifts, and a defined name rewritten to name a renamed sheet or to
/// `#REF!` for a deleted one.
fn fact_explained(item: &Item, edit: &Structural) -> bool {
    let on_sheet = fact_sheet(&item.key) == Some(edit.sheet.as_str());
    let detail = item.detail.as_deref().unwrap_or_default();
    match (edit.edit, item.verdict) {
        (SheetEdit::Add, Verdict::Added) | (SheetEdit::Delete, Verdict::Lost) => on_sheet,
        (SheetEdit::Delete | SheetEdit::Move, Verdict::Changed) if item.key.ends_with(" order") => {
            item.key.starts_with("sheet ")
        }
        (SheetEdit::Rename, Verdict::Changed) => {
            item.key.starts_with("name ") && detail.contains(R2_SHEET)
        }
        (SheetEdit::Delete, Verdict::Changed) => {
            item.key.starts_with("name ") && detail.contains("#REF!")
        }
        _ => false,
    }
}

/// Whether an element or attribute item is one a sheet-list edit rewrites:
/// - the workbook's sheet list and book views (`activeTab`), and each sheet view's `tabSelected`
///   after a delete or move, which pass the selection to another sheet;
/// - a defined name: rewritten to name a renamed sheet, rewritten to `#REF!` by a delete, and,
///   under a delete or move, a sheet-scoped one, whose `localSheetId` (part of its key, D4.2)
///   counts tab positions;
/// - what the extended properties list inside their sheet titles (`TitlesOfParts`,
///   `HeadingPairs`); the lists themselves, lost whole, are the export's, not the edit's.
fn package_explained(item: &Item, edit: SheetEdit) -> bool {
    let Some((part, path)) = item.key.split_once(':') else {
        return false;
    };
    let detail = item.detail.as_deref().unwrap_or_default();
    let reorders = matches!(edit, SheetEdit::Delete | SheetEdit::Move);
    if part.ends_with("/extended-properties[0]") {
        return [
            "/Properties/ep:TitlesOfParts[0]",
            "/Properties/ep:HeadingPairs[0]",
        ]
        .iter()
        .any(|list| path.strip_prefix(list).is_some_and(|rest| !rest.is_empty()));
    }
    if sheet_of_part(part).is_some() {
        return reorders && path.ends_with("@tabSelected");
    }
    if part != "/officeDocument[0]" {
        return false;
    }
    if path.starts_with("/workbook/sheets[0]") || path.starts_with("/workbook/bookViews[0]") {
        return true;
    }
    let Some(name) = path.strip_prefix("/workbook/definedNames[0]/definedName[") else {
        return false;
    };
    let scoped = name
        .split_once(']')
        .and_then(|(key, _)| key.rsplit_once(','))
        .is_some_and(|(_, local)| !local.is_empty());
    let changed = item.verdict == Verdict::Changed;
    match edit {
        SheetEdit::Rename => changed && detail.contains(R2_SHEET),
        SheetEdit::Delete => scoped || (changed && detail.contains("#REF!")),
        SheetEdit::Move => scoped,
        SheetEdit::Add => false,
    }
}

/// The sheet a part key names: the label of a sheet relationship of the workbook part, which is
/// the sheet's name (SPEC-0033 D2; `/officeDocument[0]/worksheet[Sheet1]/drawing[0]`).
pub(super) fn sheet_of_part(key: &str) -> Option<&str> {
    let segment = key.split('/').nth(2)?;
    let (kind, rest) = segment.split_once('[')?;
    SHEET_KINDS
        .contains(&kind)
        .then(|| rest.strip_suffix(']'))
        .flatten()
}

/// `key` with the sheet `a` of its sheet segment renamed `b`.
fn swap_part(key: &str, a: &str, b: &str) -> String {
    match sheet_of_part(key) {
        Some(sheet) if sheet == a => {
            let mut segments: Vec<String> = key.split('/').map(str::to_string).collect();
            if let Some(segment) = segments.get_mut(2) {
                let kind = segment.split_once('[').map_or("", |(k, _)| k).to_string();
                *segment = format!("{kind}[{b}]");
            }
            segments.join("/")
        }
        _ => key.to_string(),
    }
}

/// Where the sheet name sits in a semantic fact's key (`semantic::model_facts`,
/// `stored_facts`, `value_facts`): before the last space of `sheet <name> <what>`, before the last
/// `!` of the others, as nothing after a sheet's `!` (an A1 cell, a range, a name) holds one.
fn sheet_span(key: &str) -> Option<(usize, usize)> {
    let (kind, rest) = key.split_once(' ')?;
    let end = match kind {
        "sheet" => rest.rfind(' ')?,
        "cell" | "value" | "merge" | "link" | "cf" | "name" => rest.rfind('!')?,
        _ => return None,
    };
    let start = kind.len() + 1;
    Some((start, start + end))
}

/// The sheet a semantic fact is about, or `None` for a workbook-wide one.
fn fact_sheet(key: &str) -> Option<&str> {
    sheet_span(key).map(|(start, end)| &key[start..end])
}

/// `facts` with the sheet `a` renamed `b` in their keys, and in a table's value, which starts
/// with its sheet.
fn swap_facts(facts: Facts, a: &str, b: &str) -> Facts {
    let table_prefix = format!("{a}!");
    facts
        .into_iter()
        .map(|(key, value)| {
            let key = match sheet_span(&key) {
                Some((start, end)) if &key[start..end] == a => {
                    format!("{}{b}{}", &key[..start], &key[end..])
                }
                _ => key,
            };
            let value = match value.strip_prefix(&table_prefix) {
                Some(rest) if key.starts_with("table ") => format!("{b}!{rest}"),
                _ => value,
            };
            (key, value)
        })
        .collect()
}

#[cfg(test)]
mod tests;
