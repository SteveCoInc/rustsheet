//! A defined pivot's `<pivotField>`s (`_pivot_field` and `_field_ext` of
//! `tools/src/rustsheet_tools/pivot_definition.py`), in their own module to keep
//! `pivot_definition.rs` within the file-size limit.

use std::collections::HashMap;
use std::fmt::Write as _;

use serde_json::Value as Json;

use super::{
    attrs, bounded, cached_json, key, Cached, DefinedPivot, Field, Key, MAX_LIST, SUBTOTALS,
};

impl DefinedPivot<'_> {
    pub(super) fn pivot_field(f: &Field<'_>) -> Result<String, String> {
        let extra = attrs(f.entry("attrs"))?;
        let mut head = String::new();
        if let Some(axis) = f.axis {
            let _ = write!(head, r#" axis="{axis}""#);
        }
        if f.data {
            head.push_str(r#" dataField="1""#);
        }
        if f.formula.is_some() {
            head.push_str(r#" dragToRow="0" dragToCol="0" dragToPage="0""#);
        }
        let subtotals: Option<Vec<&str>> = f
            .entry("subtotals")
            .and_then(Json::as_array)
            .map(|s| s.iter().filter_map(Json::as_str).collect());
        if let Some(subtotals) = &subtotals {
            if subtotals.len() > MAX_LIST || subtotals.iter().any(|s| !SUBTOTALS.contains(s)) {
                return Err(format!("subtotals of {:?}", f.name));
            }
            head.push_str(r#" defaultSubtotal="0""#);
            for s in subtotals {
                let _ = write!(head, r#" {s}Subtotal="1""#);
            }
        }
        let hidden = f.entry("hidden").and_then(Json::as_array);
        if hidden.is_some_and(|h| !h.is_empty()) && f.axis == Some("axisPage") {
            head.push_str(r#" multipleItemSelectionAllowed="1""#);
        }
        let ext = Self::field_ext(f);
        // A grouped field lists its items even off the axes, as Excel writes it (`pivot_groups.py`).
        if f.axis.is_none() && f.labels.is_none() {
            return Ok(if ext.is_empty() {
                format!(r#"<pivotField{head} showAll="0"{extra}/>"#)
            } else {
                format!(r#"<pivotField{head} showAll="0"{extra}>{ext}</pivotField>"#)
            });
        }
        let values: Vec<Option<Cached>> = if let Some(labels) = &f.labels {
            labels.clone()
        } else if f.group().is_some() {
            (f.group_labels()?.into_iter())
                .map(|label| Some(Cached::Text(label)))
                .collect()
        } else {
            f.items.clone()
        };
        let keys: HashMap<Key, usize> = (values.iter().enumerate())
            .map(|(k, v)| (key(v.as_ref()), k))
            .collect();
        let mut flags: HashMap<usize, String> = HashMap::new();
        for (kind, attr) in [("hidden", r#" h="1""#), ("collapsed", r#" sd="0""#)] {
            let listed = f.entry(kind).and_then(Json::as_array);
            for raw in bounded(listed.map_or(&[], Vec::as_slice), kind)? {
                let k = keys
                    .get(&key(cached_json(raw)?.as_ref()))
                    .ok_or_else(|| format!("{kind} item {raw} is not in field {:?}", f.name))?;
                flags.entry(*k).or_default().push_str(attr);
            }
        }
        // `item_attrs`: item text to its other `<item>` attributes.
        if let Some(listed) = f.entry("item_attrs").and_then(Json::as_object) {
            if listed.len() > MAX_LIST {
                return Err(format!("item_attrs of {:?}", f.name));
            }
            for (text, extra) in listed {
                let wanted = key(Some(&Cached::Text(text.clone())));
                let k = keys.get(&wanted).ok_or_else(|| {
                    format!("item_attrs item {text:?} is not in field {:?}", f.name)
                })?;
                flags.entry(*k).or_default().push_str(&attrs(Some(extra))?);
            }
        }
        // A calculated item is `f="1"` unless its item_attrs set `f` (`own_f`).
        let own_f: std::collections::HashSet<Key> =
            (f.entry("item_attrs").and_then(Json::as_object))
                .into_iter()
                .flatten()
                .filter(|(_, extra)| extra.get("f").is_some())
                .map(|(text, _)| key(Some(&Cached::Text(text.clone()))))
                .collect();
        for (k, v) in values.iter().enumerate() {
            let k_key = key(v.as_ref());
            if f.calculated.contains(&k_key) && !own_f.contains(&k_key) {
                flags.entry(k).or_default().push_str(r#" f="1""#);
            }
        }
        let order = super::item_order::item_order(f, &keys, values.len())?;
        let mut items = order.into_iter().fold(String::new(), |mut out, k| {
            let flag = flags.get(&k).map_or("", String::as_str);
            let _ = write!(out, r#"<item x="{k}"{flag}/>"#);
            out
        });
        let tails = subtotals.unwrap_or_else(|| vec!["default"]);
        for t in &tails {
            let _ = write!(items, r#"<item t="{t}"/>"#);
        }
        Ok(format!(
            r#"<pivotField{head} showAll="0"{extra}><items count="{}">{items}</items>{ext}</pivotField>"#,
            values.len() + tails.len()
        ))
    }

    /// The field's raw children after `items` (`_field_ext`: `autoSortScope`, then `extLst`), as
    /// written.
    fn field_ext(f: &Field<'_>) -> String {
        ["autoSortScope", "extLst"]
            .iter()
            .filter_map(|slot| f.entry("xml").and_then(|x| x.get(*slot)))
            .filter_map(Json::as_str)
            .map(|text| text.trim().to_string())
            .collect()
    }
}
