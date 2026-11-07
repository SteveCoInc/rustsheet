//! Calculated items of a defined pivot (`_calculated_items` and `_calculated_item` of
//! `tools/src/rustsheet_tools/pivot_definition.py`, T-4130): each is appended to its field's
//! shared items, written `f="1"` in the pivotField unless its `item_attrs` set `f`, and listed in
//! the cache's `<calculatedItems>`. Excel for the web refuses `f="1"`
//! (docs/excel-semantics/pivots.md, "Excel for the web refuses calculated items").

use quick_xml::escape::escape;
use serde_json::{Map, Value as Json};

use super::checks::{managed_attrs, raw_element};
use super::{key, Cached, DefinedPivot};

/// A calculated item's keys (`_CALCULATED_ITEM_KEYS`).
const KEYS: [&str; 5] = ["field", "name", "formula", "attrs", "pivot_area"];

impl DefinedPivot<'_> {
    /// `_calculated_items`: each item is appended to its field's shared items.
    pub(super) fn read_calculated_items(&mut self) -> Result<(), String> {
        for entry in self.list("calculated_items")? {
            let raw = entry.as_object().ok_or("a calculated item is an object")?;
            if let Some(key) = raw.keys().find(|k| !KEYS.contains(&k.as_str())) {
                return Err(format!(
                    "pivot {}: a calculated item has the unknown key {key:?}",
                    self.name
                ));
            }
            let text = |k: &str| raw.get(k).and_then(Json::as_str);
            let (Some(field), Some(item), Some(_)) = (text("field"), text("name"), text("formula"))
            else {
                return Err(format!(
                    "pivot {}: a calculated item is {{field, name, formula}}",
                    self.name
                ));
            };
            let i = self.index(field)?;
            let f = &mut self.fields[i];
            let value = Some(Cached::Text(item.to_string()));
            let k = key(value.as_ref());
            if f.formula.is_some()
                || item.is_empty()
                || f.items.iter().any(|v| key(v.as_ref()) == k)
            {
                return Err(format!(
                    "pivot {}: calculated item {item:?} needs a source field without that item",
                    self.name
                ));
            }
            f.items.push(value);
            f.calculated.insert(k);
            let x = f.items.len() - 1;
            self.calculated_items.push((raw, i, x));
        }
        Ok(())
    }

    /// One `<calculatedItem>` (`_calculated_item`).
    pub(super) fn calculated_item(
        raw: &Map<String, Json>,
        field: usize,
        x: usize,
    ) -> Result<String, String> {
        let formula = raw
            .get("formula")
            .and_then(Json::as_str)
            .unwrap_or_default();
        let extra = managed_attrs(raw.get("attrs"), &["formula"])?;
        let area = match raw.get("pivot_area") {
            None => format!(
                r#"<pivotArea cacheIndex="1" outline="0" fieldPosition="0"><references count="1"><reference field="{field}" count="1" selected="0"><x v="{x}"/></reference></references></pivotArea>"#
            ),
            Some(text) => raw_element(
                text.as_str()
                    .ok_or("a calculated item's pivot_area is text")?,
                "pivotArea",
            )?,
        };
        Ok(format!(
            r#"<calculatedItem formula="{}"{extra}>{area}</calculatedItem>"#,
            escape(formula)
        ))
    }
}
