//! The spelling of a name called like a function that is none, `=foo(1)`, and of a LET or LAMBDA
//! name's uses.
//!
//! A use of a LET or LAMBDA name, called or not, prints in the spelling of the innermost
//! declaration that binds it: `LET(x,5,X*2)` reads `LET(x,5,x*2)`, and `LET(x,1,LET(X,5,x))`
//! reads `LET(x,1,LET(X,5,X))` (probes/let-name-use-spelling, T-1720).
//!
//! Excel keeps one spelling per such name, called or used as a value (T-1760), for the whole
//! workbook, the first it meets in load order and then in edit order, case-insensitively: after
//! `=foo(1)`, a typed `=Foo(1)` or `=FOO(1)` prints `=foo(1)` (probes/unknown-function-name-spelling,
//! docs/excel-semantics/references.md "Unknown function call names", T-1719).

use std::collections::HashMap;

use super::stringify::bound_variables::{walk_names, NameSite};
use super::Node;
use crate::expressions::utils::spelled_like_a1_cell;
use crate::functions::logical::same_variable;
use crate::types::DefinedName;

/// Characters Excel keeps apart from their upper-case form in a name, though `to_uppercase` merges them.
const UNFOLDED: [char; 11] = [
    'ı', 'ſ', '\u{b5}', 'ϐ', 'ϑ', 'ϕ', 'ϖ', 'ϰ', 'ϱ', '\u{fb05}', 'ŉ',
];

/// The registry key of a name: upper-cased, except that the `UNFOLDED` characters keep their own
/// key and the capital sharp s ẞ is `SS` like ß. Excel folds ß/SS/ẞ, ς/σ/Σ, ϵ/ε, ﬁ/FI, ﬀ, ﬃ, ﬄ, ﬆ and
/// ǆ/Ǆ but not ı/I, ſ/S, µ/μ/Μ, ϐ/β (likewise ϑ ϕ ϖ ϰ ϱ), ﬅ/ST or ŉ
/// (probes/unknown-function-name-folds{,-2}, T-1818, T-1825).
fn spelling_key(name: &str) -> String {
    let mut key = String::with_capacity(name.len());
    for c in name.chars() {
        match c {
            'ẞ' => key.push_str("SS"),
            c if UNFOLDED.contains(&c) => key.push(c),
            c => key.extend(c.to_uppercase()),
        }
    }
    key
}

/// Upper-cased name (ß folds with SS, T-1762; see `spelling_key`) → the spelling every call or value use of it prints with.
#[derive(Clone, Default)]
pub(crate) struct CallNameSpellings {
    first: HashMap<String, String>,
    /// Upper-cased defined name → its spelling, refreshed with every reparse of the workbook's
    /// formulas (every defined-name edit reparses them), so a lookup costs one allocation.
    defined: HashMap<String, String>,
}

impl CallNameSpellings {
    /// Registers the spellings of the workbook's defined names; the first of equal names wins.
    pub(crate) fn set_defined_names(&mut self, defined_names: &[DefinedName]) {
        self.defined.clear();
        for d in defined_names {
            self.defined
                .entry(spelling_key(&d.name))
                .or_insert_with(|| d.name.clone());
        }
    }

    /// Gives every callee and value-used name of `node` that no LET or LAMBDA binds the workbook's spelling, which a
    /// name met for the first time becomes. A bound name, called or used, takes the spelling of
    /// its declaration. A callee that is a defined name takes the defined name's spelling, whatever
    /// the first typed one (probes/unknown-function-name-reach, T-1763). Returns whether a name
    /// changed.
    pub(crate) fn apply(&mut self, node: &mut Node) -> bool {
        let mut changed = false;
        walk_names(node, &mut Vec::new(), &mut |site, name, declared| {
            let spelling = match (declared, site) {
                (Some(declared), _) => declared.to_string(),
                (None, NameSite::Declaration | NameSite::Parameter) => return,
                (None, NameSite::Callee | NameSite::Use) => {
                    let key = spelling_key(name);
                    match self.defined.get(&key) {
                        Some(defined) if site == NameSite::Callee => defined.clone(),
                        _ => self
                            .first
                            .entry(key)
                            .or_insert_with(|| name.clone())
                            .clone(),
                    }
                }
            };
            if *name != spelling {
                *name = spelling;
                changed = true;
            }
        });
        changed
    }
}

/// Drops the `_xlpm.` prefix the parser keeps on every use (used or called) from the ones a LET
/// or LAMBDA binds, which evaluation finds by their bare name. An unbound one keeps it: it is
/// Excel's undefined name (`#NAME?`), and FORMULATEXT shows the prefix, for any name, from a
/// file or typed in either mode: `=_xlpm.f1`, `=_xlpm.abc`, `=SUM(_xlpm.xy1,2)`, `=_xlpm.f1(2)`,
/// `=LET(x,1,x+_xlpm.f1)` (probes/t4040-r1c1-unbound-cell-names A14:C16,
/// probes/t4099-file-xlpm-unbound-names, excel-graph, T-4099). The prefixed name is no defined
/// name either: the defined-name lookup of `name_operand` never matches it.
///
/// Only the mark goes: a use spelled like its declaration is that name already, the file's
/// `_xlpm._xlpm.x` of a declared `_xlpm.x` drops one prefix, and `_xlpm._xlpm.x` where only `x`
/// is declared binds nothing (`#NAME?`, FORMULATEXT `=LET(x,2,_xlpm._xlpm.x*3)`,
/// probes/t4224-doubled-xlpm-prefix A8, A13, T-4224).
pub(super) fn settle_lambda_prefixes(node: &mut Node) {
    walk_names(node, &mut Vec::new(), &mut |site, name, declared| {
        let Some(declared) = declared else { return };
        if matches!(site, NameSite::Use | NameSite::Callee) && !same_variable(name, declared) {
            if let Some(bare) = name.strip_prefix("_xlpm.") {
                *name = bare.to_string();
            }
        }
    });
}

/// Gives every name spelled like an A1 cell that no LET or LAMBDA binds (used or called) the
/// quoted spelling `'f1'`. Typed in R1C1 mode, `=f1`, `=F1+1` and `=SUM(f1,2)` read such a
/// name, an undefined one (`#NAME?`), and FORMULATEXT quotes it like a sheet name: `='f1'`,
/// `='AB12'*2`, `=SUM('f1',2)`, `='f1'(2)` (probes/t3897-r1c1-let-cell-names A8:C10,
/// probes/t4040-r1c1-unbound-cell-names A7:C13, excel-graph, T-4040); a name that is no cell
/// stays bare (`=Xfe1`). The quoted spelling is no name a LET can declare and no defined name,
/// so it evaluates to `#NAME?`; the R1C1 storage form writes it `_xlpm.f1` (`variable_text`),
/// which this reads back quoted, and a name written with its prefix `_xlpm._xlpm.x` (T-4099).
pub(super) fn quote_unbound_cell_shaped_names(node: &mut Node) {
    walk_names(node, &mut Vec::new(), &mut |site, name, declared| {
        if declared.is_none()
            && matches!(site, NameSite::Use | NameSite::Callee)
            && spelled_like_a1_cell(name)
        {
            *name = format!("'{name}'");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::spelling_key;

    /// Whitespace-separated name pairs: whether the two keys of each pair are equal.
    fn pairs_fold(pairs: &str, folds: bool) {
        let names: Vec<&str> = pairs.split_whitespace().collect();
        for pair in names.chunks(2) {
            assert_eq!(
                spelling_key(pair[0]) == spelling_key(pair[1]),
                folds,
                "{pair:?}"
            );
        }
    }

    #[test]
    fn folds_match_excel() {
        pairs_fold(
            "ςw σw ςu Σu ﬁq FIq ǆd Ǆd straße STRASSE ϵg εg ﬀj FFj ﬆn STn ẞr ßr ẞs SSs",
            true,
        );
        pairs_fold(
            "ıx Ix ſy Sy \u{b5}z μz \u{b5}v Μv ϐa βa ϑb θb ϕc φc ϖd πd ϰe κe ϱf ρf ﬅm STm ŉo ʼNo",
            false,
        );
    }
}
