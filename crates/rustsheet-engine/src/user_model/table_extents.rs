//! Table extents in the history (T-3634): a row delete records the extents of the tables it
//! resized, and its undo puts them back once the rows are back.

use crate::{
    constants::LAST_ROW,
    table_edits::bounds,
    user_model::history::{Diff, TableExtent},
    UserModel,
};

impl UserModel {
    /// The extents of the tables on `sheet`, by name.
    pub(super) fn table_extents(&self, sheet: u32) -> Result<Vec<TableExtent>, String> {
        let sheet_name = self.model.workbook.worksheet(sheet)?.get_name();
        Ok(self
            .model
            .tables_on_sheet(&sheet_name)
            .into_iter()
            .map(|(name, table)| TableExtent {
                name,
                reference: table.reference,
                totals_row_count: table.totals_row_count,
            })
            .collect())
    }

    /// A [`Diff::SetTableExtents`] of the tables in `old_extents` (taken before a structural
    /// edit of `sheet`) whose extent the edit changed; none when it changed none. It goes before
    /// the edit's own diff in the list, so that undo, which runs the list backwards, puts the
    /// extents back after the edit is undone (T-3634).
    pub(super) fn table_extents_snapshot_diffs(
        &self,
        sheet: u32,
        old_extents: Vec<TableExtent>,
    ) -> Result<Vec<Diff>, String> {
        let now = self.table_extents(sheet)?;
        let changed: Vec<TableExtent> = old_extents
            .into_iter()
            .filter(|old| now.iter().any(|new| new.name == old.name && new != old))
            .collect();
        if changed.is_empty() {
            Ok(vec![])
        } else {
            Ok(vec![Diff::SetTableExtents {
                sheet,
                old_value: changed,
            }])
        }
    }

    /// Undoes [`Diff::SetTableExtents`](crate::user_model::history::Diff::SetTableExtents): gives
    /// each table of `sheet` in `old_value` its extent back, then reparses the formulas against
    /// the tables (T-3634). It runs after the rows are back, so a genuine extent keeps the
    /// table's columns and top row and only moves its bottom edge; anything else (an extent from
    /// an untrusted peer that would move or widen a table, overlap another, or leave it no data
    /// row) is refused before any table changes.
    pub(super) fn undo_table_extents(
        &mut self,
        sheet: u32,
        old_value: &[TableExtent],
    ) -> Result<(), String> {
        let sheet_name = self.model.workbook.worksheet(sheet)?.get_name();
        let tables = &self.model.workbook.tables;
        for extent in old_value {
            let refused = || {
                format!(
                    "Table '{}': invalid extent '{}'",
                    extent.name, extent.reference
                )
            };
            let table = tables
                .get(&extent.name)
                .filter(|t| t.is_on_sheet(&sheet_name))
                .ok_or_else(|| format!("Table '{}' is not on sheet {sheet}", extent.name))?;
            let mut restored = table.clone();
            restored.reference.clone_from(&extent.reference);
            restored.totals_row_count = extent.totals_row_count;
            let (Some(now), Some(new)) = (bounds(table), bounds(&restored)) else {
                return Err(refused());
            };
            let keeps_place = (new.left, new.right, new.top) == (now.left, now.right, now.top);
            if !keeps_place
                || extent.totals_row_count > 1
                || new.bottom > LAST_ROW
                || new.data.0 > new.data.1
            {
                return Err(refused());
            }
            let overlaps = tables.iter().any(|(name, other)| {
                name != &extent.name
                    && other.is_on_sheet(&sheet_name)
                    && bounds(other).is_some_and(|o| {
                        o.left <= new.right
                            && new.left <= o.right
                            && o.top <= new.bottom
                            && new.top <= o.bottom
                    })
            });
            if overlaps {
                return Err(refused());
            }
        }
        for extent in old_value {
            if let Some(table) = self.model.workbook.tables.get_mut(&extent.name) {
                table.reference.clone_from(&extent.reference);
                table.totals_row_count = extent.totals_row_count;
            }
        }
        self.model.reset_tables();
        Ok(())
    }
}
