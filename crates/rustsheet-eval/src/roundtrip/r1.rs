//! Scenario R1 (SPEC-0033 D1, D5): load, recalculate, apply the standard edits, save, and keep
//! only the items the edits explain no other way. Every failing item of the edited save that the
//! plain save (R0) did not have must be in `Δ(E)`: the cells the edits wrote, the cached values
//! they changed, the records E5 and E6 append, and the derived counts and range (N9, N10).
//! Anything else is a delta violation.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use rustsheet::{CellRef, ChangeSet, EditError, Input, RangeRef, SheetId, Value, Workbook};

use super::{Item, Verdict};

/// One edit of the standard set (D1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Edit {
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

/// E5's text: no workbook holds it, so the shared-string table must grow.
pub(super) const E5_TEXT: &str = "rustsheet-roundtrip-E5";

/// Most cells E3 visits looking for an empty one before reporting the edit `n/a` (directive 5).
const MAX_SCAN: u64 = 1 << 20;

/// Most candidates E1 and E2 try in turn when the engine refuses one (a cell inside a spill or
/// an array, which is no constant and no formula of its own).
const MAX_TRIES: usize = 64;

/// What `Δ(E)` holds (D5), by cell as the semantic facts spell them (`Sheet!A1`).
#[derive(Debug, Default)]
pub(super) struct Delta {
    /// The cells the edits wrote: their input, formula, value and computed flag may change.
    pub(super) written: BTreeSet<String>,
    /// The cells whose cached value the edits changed: only their value may change.
    pub(super) changed: BTreeSet<String>,
    /// The written cells that were empty before: their style fact is new, as every stored cell
    /// has one.
    pub(super) created: BTreeSet<String>,
    /// E6's cells, whose style E6 changes.
    pub(super) restyled: BTreeSet<String>,
    /// The edited sheet, whose `dimension` the edits may extend (N9).
    pub(super) sheet: String,
    /// Whether E5 ran: only it appends shared strings.
    pub(super) strings: bool,
    /// Whether E6 ran: only it appends style records and restyles a cell.
    pub(super) styles: bool,
}

/// What `apply` did beyond `Δ(E)`, for the report: the writes the engine refused, and the edits
/// that had no target.
#[derive(Debug, Default, Clone, Copy)]
pub(super) struct Outcome {
    pub(super) refused: usize,
    pub(super) missing: usize,
}

/// Applies `edits` to the first sheet of `workbook` (already recalculated, as R0 saves it), each
/// in its own transaction. Returns `Δ(E)`, or `None` when no edit applied (the scenario is `n/a`
/// for the case), with how many writes were refused and edits had no target.
///
/// A write the engine refuses because of its target (`EditError::Ref` or `InvalidInput`, such as a
/// cell inside a spill) moves E1 and E2 on to their next candidate and leaves the other edits
/// without one.
///
/// # Errors
/// When the sheet cannot be read, or the engine fails an edit for any other reason.
pub(super) fn apply(
    workbook: &mut Workbook,
    edits: &[Edit],
) -> Result<(Option<Delta>, Outcome), String> {
    let sheets = workbook.sheets();
    // The first sheet in tab order. D1 says the first visible worksheet; the facade does not
    // expose visibility yet (T-4868).
    let Some(first) = sheets.iter().min_by_key(|s| s.index) else {
        return Ok((None, Outcome::default()));
    };
    let targets = Targets::find(workbook, first.id)?;
    let names: HashMap<SheetId, String> = sheets.iter().map(|s| (s.id, s.name.clone())).collect();
    workbook.set_track_changes(true);
    let mut delta: Option<Delta> = None;
    let mut outcome = Outcome::default();
    for edit in edits {
        let candidates = targets.plan(*edit);
        if candidates.is_empty() {
            outcome.missing += 1;
            continue;
        }
        let mut applied = None;
        for (cell, input, format) in candidates {
            let result = workbook.edit(|t| {
                t.set(cell, input.clone())?;
                if let Some(format) = format {
                    t.set_number_format(cell, format)?;
                }
                Ok(())
            });
            match result {
                Ok(((), changes)) => {
                    applied = Some(changes);
                    break;
                }
                Err(EditError::Ref(_) | EditError::InvalidInput { .. }) => outcome.refused += 1,
                Err(e) => return Err(format!("the engine failed edit {edit:?}: {e}")),
            }
        }
        let Some(changes) = applied else {
            outcome.missing += 1;
            continue;
        };
        let d = delta.get_or_insert_with(|| Delta {
            sheet: first.name.clone(),
            ..Delta::default()
        });
        d.strings |= *edit == Edit::E5;
        d.styles |= *edit == Edit::E6;
        record(&changes, &names, *edit == Edit::E6, d);
    }
    Ok((delta, outcome))
}

/// Adds a change set's written, created and value-changed cells to `delta`, and with `restyle`
/// (E6) its written cells to the restyled ones.
pub(super) fn record(
    changes: &ChangeSet,
    names: &HashMap<SheetId, String>,
    restyle: bool,
    delta: &mut Delta,
) {
    let at = |cell: &CellRef| names.get(&cell.sheet).map(|name| format!("{name}!{cell}"));
    for input in &changes.inputs {
        let Some(cell) = at(&input.cell) else {
            continue;
        };
        if input.before.is_none() {
            delta.created.insert(cell.clone());
        }
        if restyle {
            delta.restyled.insert(cell.clone());
        }
        delta.written.insert(cell);
    }
    delta
        .changed
        .extend(changes.values.iter().filter_map(|c| at(&c.cell)));
}

/// One planned write: the cell, its input, and the number format E6 gives it.
type Planned = (CellRef, Input, Option<&'static str>);

/// Where each edit of the standard set writes on one sheet.
struct Targets {
    sheet: SheetId,
    /// Numeric constants and formula cells in row-major order, at most [`MAX_TRIES`] each.
    numbers: Vec<(CellRef, f64)>,
    formulas: Vec<CellRef>,
    first_empty: Option<CellRef>,
    /// The row below the used range (0 on an empty sheet).
    below: u32,
}

impl Targets {
    fn find(workbook: &Workbook, sheet: SheetId) -> Result<Targets, String> {
        let mut cells = workbook.stored_cells(sheet).map_err(|e| e.to_string())?;
        cells.sort_by_key(|c| (c.row, c.col));
        let mut numbers = Vec::new();
        let mut formulas = Vec::new();
        for &cell in &cells {
            if numbers.len() >= MAX_TRIES && formulas.len() >= MAX_TRIES {
                break;
            }
            // A spill or array member holds the anchor's result, no constant or formula of its own.
            if workbook.in_spill(cell).unwrap_or(true) {
                continue;
            }
            let Ok(info) = workbook.cell(cell) else {
                continue;
            };
            match (&info.formula, &info.value) {
                (Some(_), _) if formulas.len() < MAX_TRIES => formulas.push(cell),
                (None, Value::Number(n)) if numbers.len() < MAX_TRIES => {
                    numbers.push((cell, *n));
                }
                _ => {}
            }
        }
        let used = workbook.used_range(sheet).map_err(|e| e.to_string())?;
        let stored: HashSet<(u32, u32)> = cells.iter().map(|c| (c.row, c.col)).collect();
        Ok(Targets {
            sheet,
            numbers,
            formulas,
            first_empty: used.and_then(|range| first_empty(range, &stored)),
            below: used.map_or(0, |range| range.bottom.saturating_add(1)),
        })
    }

    /// The writes `edit` tries in turn (the first the engine accepts is the edit); empty when it
    /// has no target.
    fn plan(&self, edit: Edit) -> Vec<Planned> {
        let below = |col: u32| CellRef::new(self.sheet, self.below, col).ok();
        let one = |cell: Option<CellRef>, input: Input, format: Option<&'static str>| {
            cell.map(|cell| (cell, input, format)).into_iter().collect()
        };
        match edit {
            Edit::E1 => self
                .numbers
                .iter()
                .map(|&(cell, n)| (cell, Input::from(n + 1.0), None))
                .collect(),
            Edit::E2 => self
                .formulas
                .iter()
                .map(|&cell| (cell, Input::Formula("=1+1".into()), None))
                .collect(),
            Edit::E3 => one(self.first_empty, Input::from(42.0), None),
            Edit::E4 => one(below(0), Input::from(42.0), None),
            Edit::E5 => one(below(1), Input::Value(Value::from(E5_TEXT)), None),
            Edit::E6 => one(below(2), Input::from(0.5), Some("0.000")),
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

/// How an item is matched across the plain and the edited save: a validator item by its detail
/// too, since its key is only the check's id.
type Identity<'a> = (&'a str, &'a str, Verdict, Option<&'a str>);

fn identity(item: &Item) -> Identity<'_> {
    let detail = (item.kind == "package")
        .then_some(item.detail.as_deref())
        .flatten();
    (item.kind, item.key.as_str(), item.verdict, detail)
}

/// The delta violations of an R1 or R2 run (D5): the failing items of `edited` that the plain
/// save's failing items (`plain`) do not hold and the run's delta does not explain (`explained`),
/// relabelled `scenario`.
///
/// An item R0 also has is excused only up to R0's weight. A `<truncated>` item that hides more
/// differences than R0's stays a violation weighing the excess, so a worse loss behind the item
/// cap cannot pass.
pub(super) fn violations(
    scenario: &'static str,
    plain: &[Item],
    edited: Vec<Item>,
    explained: impl Fn(&Item) -> bool,
) -> Vec<Item> {
    let mut known: BTreeMap<Identity<'_>, usize> = BTreeMap::new();
    for i in plain.iter().filter(|i| !i.verdict.passes()) {
        *known.entry(identity(i)).or_default() += i.weight;
    }
    let mut out = Vec::new();
    for item in edited.into_iter().filter(|i| !i.verdict.passes()) {
        if explained(&item) {
            continue;
        }
        match known.get(&identity(&item)).copied() {
            None => out.push(Item { scenario, ..item }),
            Some(weight) if item.weight > weight => {
                let excess = item.weight - weight;
                out.push(Item {
                    scenario,
                    weight: excess,
                    ..item
                });
            }
            Some(_) => {}
        }
    }
    out
}

/// Whether `Δ(E)` explains `item` (D5). Each kind of change has the one edit that can cause it:
/// - a written cell's input, formula, computed flag and value; its style when the edit created
///   the cell or E6 restyled it;
/// - a value-changed cell's value, and nothing else about it;
/// - a number format or cell format appended under E6, with the style counts (N10);
/// - the shared-string counts under E5 (N10);
/// - the edited sheet's `dimension@ref` (N9).
pub(super) fn explained(item: &Item, delta: &Delta) -> bool {
    match item.kind {
        "semantic" => fact_explained(&item.key, item.verdict, delta),
        "element" | "attribute" => {
            let Some((part, path)) = item.key.split_once(':') else {
                return false;
            };
            if part.ends_with("/styles[0]") {
                delta.styles
                    && (STYLE_COUNTS.contains(&path)
                        || (item.verdict == Verdict::Added && style_record(path)))
            } else if part.ends_with("/sharedStrings[0]") {
                delta.strings && SST_COUNTS.contains(&path)
            } else {
                path == "/worksheet/dimension[0]@ref"
                    && part.contains(&format!("/worksheet[{}]", delta.sheet))
            }
        }
        _ => false,
    }
}

/// The derived counts of the styles part E6 changes (N10).
const STYLE_COUNTS: &[&str] = &[
    "/styleSheet/numFmts[0]@count",
    "/styleSheet/cellXfs[0]@count",
];
/// The derived counts of the shared-string part E5 changes (N10).
const SST_COUNTS: &[&str] = &["/sst@count", "/sst@uniqueCount"];

/// Whether `delta` explains a semantic fact: `cell <at> input|formula|computed|style` or
/// `value <at>`. A style fact is explained when the edit created the cell (its new style fact is
/// `added`) or E6 restyled it. A link is never explained: no edit writes one.
fn fact_explained(key: &str, verdict: Verdict, delta: &Delta) -> bool {
    if let Some(at) = key.strip_prefix("value ") {
        return delta.written.contains(at) || delta.changed.contains(at);
    }
    let Some(rest) = key.strip_prefix("cell ") else {
        return false;
    };
    for what in ["input", "formula", "computed"] {
        if let Some(at) = rest.strip_suffix(&format!(" {what}")[..]) {
            return delta.written.contains(at);
        }
    }
    rest.strip_suffix(" style").is_some_and(|at| {
        delta.restyled.contains(at) || (verdict == Verdict::Added && delta.created.contains(at))
    })
}

/// An appended number format or cell format (E6), or the `numFmts` list E6 creates.
fn style_record(path: &str) -> bool {
    path.starts_with("/styleSheet/numFmts[0]") || path.starts_with("/styleSheet/cellXfs[0]/xf[")
}
