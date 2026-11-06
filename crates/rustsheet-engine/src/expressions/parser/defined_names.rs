//! Defined-name lookups while parsing: one probe of `Parser::name_index` per lookup (T-3195).

use std::collections::HashMap;

use super::{DefinedNameS, Node, Parser};
use crate::expressions::utils::quote_name;
use crate::name_key::{get_name, NameKey};

/// The positions of `names` by scope and lower-cased name, the first of names that fold alike
/// (`Parser::name_index`).
pub(super) fn index_names(names: &[DefinedNameS]) -> HashMap<NameKey, usize> {
    let mut index = HashMap::with_capacity(names.len());
    for (position, (name, scope, _)) in names.iter().enumerate() {
        index.entry(NameKey::new(*scope, name)).or_insert(position);
    }
    index
}

impl Parser {
    // Returns:
    //  * None: If there is no defined name by that name
    //  * Some((Some(index), formula)): If there is a defined name local to that sheet
    //  * Some(None): If there is a global defined name
    pub(super) fn get_defined_name(&self, name: &str, sheet: u32) -> Option<(Option<u32>, String)> {
        if let Some(formula) = self.defined_name_formula(Some(sheet), name) {
            return Some((Some(sheet), formula.to_owned()));
        }
        self.defined_name_formula(None, name)
            .map(|formula| (None, formula.to_owned()))
    }

    /// The body of the name `name` (any case) in `scope`, the first of the names that fold to
    /// it; `None` when there is none.
    pub(super) fn defined_name_formula(&self, scope: Option<u32>, name: &str) -> Option<&str> {
        let index = *get_name(&self.name_index, scope, name)?;
        self.defined_names
            .get(index)
            .map(|(_, _, formula)| formula.as_str())
    }

    /// In an API-added name's body ([`Parser::set_api_name_body`]), the bare name `name` bound
    /// to its sheet-scoped definition on the first sheet by name order ([`sheet_name_order`]),
    /// written with that sheet's prefix so a sheet rename rewrites it; `None` outside such a body
    /// or when no sheet defines `name`, which then reads the workbook name as anywhere else.
    /// Tab order and the active sheet play no part (probes/t4554-api-name-scoped-name-choice,
    /// -first, -last: First < Last < S2 with tabs First, S2, Last; T-4587).
    pub(super) fn api_body_sheet_name(&self, name: &str) -> Option<Node> {
        if !self.api_name_body {
            return None;
        }
        let (sheet, sheet_name, formula) = self
            .worksheets
            .iter()
            .enumerate()
            .filter_map(|(index, sheet_name)| {
                let index = u32::try_from(index).ok()?;
                let formula = self.defined_name_formula(Some(index), name)?;
                Some((index, sheet_name, formula))
            })
            .min_by(|(_, a, _), (_, b, _)| sheet_name_order(a, b))?;
        Some(Node::DefinedNameKind((
            format!("{}!{name}", quote_name(sheet_name)),
            Some(sheet),
            formula.to_owned(),
        )))
    }
}

/// The order of sheet names in which an API-added name's body picks a sheet-scoped definition
/// ([`Parser::api_body_sheet_name`]): letter case is ignored (probes/t4587-api-name-sheet-order-case).
fn sheet_name_order(a: &str, b: &str) -> std::cmp::Ordering {
    a.to_lowercase().cmp(&b.to_lowercase())
}
