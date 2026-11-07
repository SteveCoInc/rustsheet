//! A pivot field's `autoSortScope` typed (T-4827): read from the element the table reader keeps
//! verbatim for export, as `<filters>` is, so the round trip writes it back as it was.

use rustsheet_engine::pivot::{AutoSortScope, SortReference, DATA_FIELD_REFERENCE};

use super::{at, attribute, local_name, walk, Skip, Tag};

/// The path of a scope's references.
const REFERENCES: [&str; 3] = ["autoSortScope", "pivotArea", "references"];

/// The path of a reference's items.
const ITEMS: [&str; 4] = ["autoSortScope", "pivotArea", "references", "reference"];

/// The scope of a kept `autoSortScope` element `xml`: its `pivotArea`'s references.
///
/// The first reference whose `field` is [`DATA_FIELD_REFERENCE`] and that has an `x` names the
/// data field by its first `x`; every other reference is kept with its `x`s. A reference whose
/// `field` is no number, and an `x` with no `v` that is a number, are not read (UNVERIFIED,
/// T-4799: Excel writes neither).
///
/// The area's own attributes (`axis`, `field`, `type`, ...) and the references' subtotal flags
/// change nothing visible (probes/t4130-pivot-sort SXA, SXR; docs/excel-semantics/pivots.md
/// "Pivot sorts and filters as Excel for the web renders them") and are not read. The element is
/// bounded by the opaque budget it was kept under, and so is what is read from it.
pub(super) fn read_auto_sort(xml: &str) -> Result<AutoSortScope, Skip> {
    // Per reference, its field (`None` for one not read) and its items.
    let mut references: Vec<(Option<u32>, Vec<u32>)> = Vec::new();
    walk(xml.as_bytes(), |path, tag| {
        let Tag::Open(element) = tag else {
            return Ok(());
        };
        let name = local_name(element);
        if at(path, &REFERENCES) && name == "reference" {
            let field = attribute(element, "field")?.and_then(|f| f.parse::<u32>().ok());
            references.push((field, Vec::new()));
        } else if at(path, &ITEMS) && name == "x" {
            let item = attribute(element, "v")?.and_then(|v| v.parse::<u32>().ok());
            if let (Some(item), Some((_, items))) = (item, references.last_mut()) {
                items.push(item);
            }
        }
        Ok(())
    })?;
    let mut scope = AutoSortScope::default();
    for (field, items) in references {
        match field {
            Some(DATA_FIELD_REFERENCE) => {
                if scope.data_field.is_none() {
                    scope.data_field = items.first().copied();
                }
            }
            Some(field) => scope.references.push(SortReference { field, items }),
            None => {}
        }
    }
    Ok(scope)
}
