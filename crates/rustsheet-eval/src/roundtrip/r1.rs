//! Scenario R1 (SPEC-0033 D1, D5): load, recalculate, apply the standard edits, save, and keep
//! only the items the edits explain no other way. Every item of the edited save that the plain
//! save (R0) did not have must be in `Δ(E)`: the cells the edits wrote, the cells whose values
//! they changed, the style records they appended, and the derived counts (N9, N10). Anything
//! else is a delta violation.

use std::collections::{BTreeSet, HashMap, HashSet};

use rustsheet::{CellRef, ChangeSet, Input, RangeRef, SheetId, Value, Workbook};

use super::{Item, Verdict};

/// One edit of the standard set (D1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Edit {
    /// The first numeric constant gets its value plus 1.
    E1,
    /// The first formula cell becomes `=1+1`.
    E2,
    /// The first empty cell inside the used range gets 42.
    E3,
    /// Column A, one row below the used range, gets 42.
    E4,
    /// E4's row, column B, gets a text the shared-string table cannot hold yet.
    E5,
    /// E4's row, column C, gets 0.5 in the number format `0.000`, a style the workbook may lack.
    E6,
}

/// The edits of the R1 scenario `name` (`R1/E1` … `R1/E6`, `R1/all`), or `None` for another name.
pub(super) fn edits_for(name: &str) -> Option<&'static [Edit]> {
    Some(match name {
        "R1/E1" => &[Edit::E1],
        "R1/E2" => &[Edit::E2],
        "R1/E3" => &[Edit::E3],
        "R1/E4" => &[Edit::E4],
        "R1/E5" => &[Edit::E5],
        "R1/E6" => &[Edit::E6],
        "R1/all" => &[Edit::E1, Edit::E2, Edit::E3, Edit::E4, Edit::E5, Edit::E6],
        _ => return None,
    })
}

/// E5's text: no workbook holds it, so the shared-string table must grow.
pub(super) const E5_TEXT: &str = "rustsheet-roundtrip-E5";

/// Most cells E3 visits looking for an empty one before reporting the edit `n/a` (directive 5).
const MAX_SCAN: u64 = 1 << 20;

/// The cells `Δ(E)` names, as the semantic facts spell them (`Sheet!A1`).
pub(super) type Delta = BTreeSet<String>;

/// Applies `edits` to the first sheet of `workbook` (already recalculated, as R0 saves it), each in
/// its own transaction. Returns the cells the edits wrote or whose values they changed, or
/// `None` when no edit had a target (the scenario is `n/a` for the case). An edit the engine
/// refuses (a target inside a spill, say) has no valid target either and is skipped, so one
/// refused edit does not cost the case its other scenarios.
///
/// # Errors
/// When the sheet cannot be read.
pub(super) fn apply(workbook: &mut Workbook, edits: &[Edit]) -> Result<Option<Delta>, String> {
    let sheets = workbook.sheets();
    // The first sheet in tab order. D1 says the first visible worksheet; the facade does not
    // expose visibility yet, and a hidden first sheet is rare in the corpora (T-4514 Outcome).
    let Some(first) = sheets.iter().min_by_key(|s| s.index) else {
        return Ok(None);
    };
    let targets = Targets::find(workbook, first.id)?;
    let names: HashMap<SheetId, String> = sheets.iter().map(|s| (s.id, s.name.clone())).collect();
    workbook.set_track_changes(true);
    let mut delta: Option<Delta> = None;
    for (cell, input, format) in edits.iter().filter_map(|e| targets.plan(*e)) {
        let applied = workbook.edit(|t| {
            t.set(cell, input.clone())?;
            if let Some(format) = format {
                t.set_number_format(cell, format)?;
            }
            Ok(())
        });
        if let Ok(((), changes)) = applied {
            delta
                .get_or_insert_with(Delta::new)
                .extend(cells_of(&changes, &names));
        }
    }
    Ok(delta)
}

/// The cells a change set names, as `Sheet!A1`.
fn cells_of(changes: &ChangeSet, names: &HashMap<SheetId, String>) -> Delta {
    let at = |cell: &CellRef| names.get(&cell.sheet).map(|name| format!("{name}!{cell}"));
    changes
        .inputs
        .iter()
        .map(|c| &c.cell)
        .chain(changes.values.iter().map(|c| &c.cell))
        .filter_map(at)
        .collect()
}

/// Where each edit of the standard set writes on one sheet.
struct Targets {
    sheet: SheetId,
    first_number: Option<(CellRef, f64)>,
    first_formula: Option<CellRef>,
    first_empty: Option<CellRef>,
    /// The row below the used range (0 on an empty sheet).
    below: u32,
}

impl Targets {
    fn find(workbook: &Workbook, sheet: SheetId) -> Result<Targets, String> {
        let mut cells = workbook.stored_cells(sheet).map_err(|e| e.to_string())?;
        cells.sort_by_key(|c| (c.row, c.col));
        let mut first_number = None;
        let mut first_formula = None;
        for &cell in &cells {
            if first_number.is_some() && first_formula.is_some() {
                break;
            }
            let Ok(info) = workbook.cell(cell) else {
                continue;
            };
            match (&info.formula, &info.value) {
                (Some(_), _) if first_formula.is_none() => first_formula = Some(cell),
                (None, Value::Number(n)) if first_number.is_none() => {
                    first_number = Some((cell, *n));
                }
                _ => {}
            }
        }
        let used = workbook.used_range(sheet).map_err(|e| e.to_string())?;
        let stored: HashSet<(u32, u32)> = cells.iter().map(|c| (c.row, c.col)).collect();
        Ok(Targets {
            sheet,
            first_number,
            first_formula,
            first_empty: used.and_then(|range| first_empty(range, &stored)),
            below: used.map_or(0, |range| range.bottom.saturating_add(1)),
        })
    }

    /// The cell, input and number format of `edit`, or `None` when it has no target.
    fn plan(&self, edit: Edit) -> Option<(CellRef, Input, Option<&'static str>)> {
        let below = |col: u32| CellRef::new(self.sheet, self.below, col).ok();
        match edit {
            Edit::E1 => self
                .first_number
                .map(|(cell, n)| (cell, Input::from(n + 1.0), None)),
            Edit::E2 => self
                .first_formula
                .map(|cell| (cell, Input::Formula("=1+1".into()), None)),
            Edit::E3 => self.first_empty.map(|cell| (cell, Input::from(42.0), None)),
            Edit::E4 => below(0).map(|cell| (cell, Input::from(42.0), None)),
            Edit::E5 => below(1).map(|cell| (cell, Input::Value(Value::from(E5_TEXT)), None)),
            Edit::E6 => below(2).map(|cell| (cell, Input::from(0.5), Some("0.000"))),
        }
    }
}

/// The first cell of `range`, in row-major order, that holds nothing, within [`MAX_SCAN`] cells.
fn first_empty(range: RangeRef, stored: &HashSet<(u32, u32)>) -> Option<CellRef> {
    let mut visited = 0u64;
    for row in range.top..=range.bottom {
        for col in range.left..=range.right {
            if !stored.contains(&(row, col)) {
                return CellRef::new(range.sheet, row, col).ok();
            }
            visited += 1;
            if visited >= MAX_SCAN {
                return None;
            }
        }
    }
    None
}

/// The delta violations of an R1 run (D5): the failing items of `edited` that the plain save's
/// failing items (`plain`) do not hold and `delta` does not explain, relabelled `scenario`.
pub(super) fn violations(
    scenario: &'static str,
    plain: &[Item],
    edited: Vec<Item>,
    delta: &Delta,
) -> Vec<Item> {
    let known: BTreeSet<(&str, &str, Verdict)> = plain
        .iter()
        .filter(|i| !i.verdict.passes())
        .map(|i| (i.kind, i.key.as_str(), i.verdict))
        .collect();
    edited
        .into_iter()
        .filter(|i| !i.verdict.passes())
        .filter(|i| !known.contains(&(i.kind, i.key.as_str(), i.verdict)))
        .filter(|i| !explained(i, delta))
        .map(|i| Item { scenario, ..i })
        .collect()
}

/// Whether `Δ(E)` explains `item`: a fact about a cell the edits touched, a style record they
/// appended, or a derived count or range (N9, N10).
fn explained(item: &Item, delta: &Delta) -> bool {
    match item.kind {
        "semantic" => fact_cell(&item.key).is_some_and(|at| delta.contains(at)),
        "element" | "attribute" => {
            let Some((part, path)) = item.key.split_once(':') else {
                return false;
            };
            derived(path)
                || (item.verdict == Verdict::Added
                    && part.ends_with("/styles[0]")
                    && style_record(path))
        }
        _ => false,
    }
}

/// The `Sheet!A1` a cell fact names: `cell <at> <what>`, `value <at>` or `link <at>`.
fn fact_cell(key: &str) -> Option<&str> {
    if let Some(rest) = key.strip_prefix("cell ") {
        return ["computed", "style", "input", "formula"]
            .iter()
            .find_map(|what| rest.strip_suffix(&format!(" {what}")[..]));
    }
    key.strip_prefix("value ")
        .or_else(|| key.strip_prefix("link "))
}

/// N9's `dimension@ref` and N10's counts, which any edit changes.
fn derived(path: &str) -> bool {
    path.ends_with("@count")
        || path.ends_with("@uniqueCount")
        || path.ends_with("/dimension[0]@ref")
}

/// An appended number format or cell format (E6), or the `numFmts` list E6 creates.
fn style_record(path: &str) -> bool {
    path.starts_with("/styleSheet/numFmts[0]") || path.starts_with("/styleSheet/cellXfs[0]/xf[")
}
