//! [`roll_back`]: how a failed [`Workbook::edit`](crate::Workbook::edit) undoes its ops (moved
//! out of edit.rs, T-4828).

use rustsheet_engine::Model;

use super::Undo;
use crate::refs::SheetId;
use crate::workbook::sheet_index;

/// Replays `undo` in reverse. Failures cannot be reported past the original error and the
/// entries only restore state they themselves captured, so release builds ignore them and debug
/// builds assert.
pub(super) fn roll_back(model: &mut Model, undo: Vec<Undo>, re_evaluate: bool) {
    for entry in undo.into_iter().rev() {
        match entry {
            Undo::Cells {
                sheet_id,
                cells,
                link: (link_key, link),
            } => {
                let Some(ws) = model
                    .workbook
                    .worksheets
                    .iter_mut()
                    .find(|ws| ws.get_sheet_id() == sheet_id)
                else {
                    continue;
                };
                for (row, col, cell) in cells {
                    match cell {
                        Some(cell) => {
                            ws.sheet_data.set_cell(row, col, cell);
                        }
                        None => {
                            ws.sheet_data.remove_cell(row, col);
                        }
                    }
                }
                match link {
                    Some(link) => {
                        ws.links.insert(link_key, link);
                    }
                    None => {
                        ws.links.remove(&link_key);
                    }
                }
            }
            Undo::AddedSheet(id) => {
                if let Some(index) = model
                    .workbook
                    .worksheets
                    .iter()
                    .position(|w| w.get_sheet_id() == id)
                {
                    let undone = model.delete_sheet(u32::try_from(index).unwrap_or(u32::MAX));
                    debug_assert!(undone.is_ok(), "undo of add_sheet failed: {undone:?}");
                }
            }
            Undo::Moved { sheet_id, index } => {
                if let Some(now) = model
                    .workbook
                    .worksheets
                    .iter()
                    .position(|w| w.get_sheet_id() == sheet_id)
                {
                    let undone = model.move_sheet(u32::try_from(now).unwrap_or(u32::MAX), index);
                    debug_assert!(undone.is_ok(), "undo of move_sheet failed: {undone:?}");
                }
            }
            Undo::Renamed { sheet_id, old_name } => {
                if let Some(index) = model
                    .workbook
                    .worksheets
                    .iter()
                    .position(|w| w.get_sheet_id() == sheet_id)
                {
                    let undone = model
                        .rename_sheet_by_index(u32::try_from(index).unwrap_or(u32::MAX), &old_name);
                    debug_assert!(undone.is_ok(), "undo of rename_sheet failed: {undone:?}");
                }
            }
            Undo::DeletedSheet { index, sheet } => {
                let undone = model.restore_sheet(index, *sheet);
                debug_assert!(undone.is_ok(), "undo of delete_sheet failed: {undone:?}");
            }
            Undo::DefinedName { name, sheet_id } => {
                // Undo runs in reverse, so the name's sheet still exists here.
                let scope = sheet_id
                    .map(|id| sheet_index(model, SheetId::new(id)))
                    .transpose();
                let undone = scope
                    .map_err(|e| e.to_string())
                    .and_then(|scope| model.delete_defined_name(&name, scope));
                debug_assert!(undone.is_ok(), "undo of define_name failed: {undone:?}");
            }
            Undo::Pivot(undo) => undo.roll_back(model),
            Undo::Table(name) => {
                let undone = model.remove_table(&name);
                debug_assert!(undone.is_ok(), "undo of add_table failed: {undone:?}");
            }
            Undo::TableBefore(table) => model.restore_table(*table),
            Undo::DataTable { sheet_id, range } => {
                let sheet = (model.workbook.worksheets.iter())
                    .position(|ws| ws.get_sheet_id() == sheet_id)
                    .and_then(|i| u32::try_from(i).ok());
                let undone = sheet.is_some_and(|i| model.remove_data_table(i, range));
                debug_assert!(undone, "undo of add_data_table failed");
            }
            Undo::RowHidden {
                sheet_id,
                row,
                hidden,
            } => {
                if let Ok(index) = sheet_index(model, SheetId::new(sheet_id)) {
                    let undone = model.set_row_hidden(index, row, hidden);
                    debug_assert!(undone.is_ok(), "undo of set_row_hidden failed: {undone:?}");
                }
            }
            Undo::Clock(clock) => model.set_clock(clock),
        }
    }
    if re_evaluate {
        model.evaluate();
    }
}
