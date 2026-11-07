//! The order of a defined pivot field's `<item>` elements (`_item_order` of
//! `tools/src/rustsheet_tools/pivot_definition.py`, T-4164): a case's `item_order` writes them in
//! an order of its own while the cache keeps its shared items, so a probe can ask whether Excel
//! shows the `<items>` order or the cache's (docs/excel-semantics/pivots.md, "A field shows its
//! items in the `<items>` order").

use std::collections::{HashMap, HashSet};

use serde_json::Value as Json;

use super::{bounded, cached_json, key, Field, Key};

/// The order of the field's `<item>` elements (`_item_order`): the items `item_order` lists
/// first, in that order, then the rest by index, or for a field a group adds in its own order
/// (`Field::item_order`, `groups.rs`, T-4170). Unlike `order` it leaves the cache's shared items as
/// they are (T-4164). It refuses what the Python builder refuses: an `item_order` that
/// is not a list, an item not in the field, and a repeated item; and, as every list here, more
/// than `MAX_LIST` entries.
pub(super) fn item_order(
    f: &Field<'_>,
    keys: &HashMap<Key, usize>,
    count: usize,
) -> Result<Vec<usize>, String> {
    let listed: &[Json] = match f.entry("item_order") {
        None => &[],
        Some(Json::Array(listed)) => listed,
        Some(_) => return Err(format!("item_order of {:?} is a list of items", f.name)),
    };
    let mut first: Vec<usize> = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    for raw in bounded(listed, "item_order")? {
        let k = *keys
            .get(&key(cached_json(raw)?.as_ref()))
            .ok_or_else(|| format!("item_order item {raw} is not in field {:?}", f.name))?;
        if !seen.insert(k) {
            return Err(format!("item_order of {:?} repeats an item", f.name));
        }
        first.push(k);
    }
    let rest: Vec<usize> = f.item_order.clone().unwrap_or_else(|| (0..count).collect());
    first.extend(rest.into_iter().filter(|k| !seen.contains(k)));
    Ok(first)
}
