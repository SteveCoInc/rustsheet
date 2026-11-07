//! Label sorts (T-4768): a field whose `sortType` is `ascending` or `descending` and that has no
//! `autoSortScope` shows its items ordered by their own values, whatever its `<items>` order
//! (docs/excel-semantics/pivots.md "Pivot sorts and filters as Excel for the web renders them",
//! label-sort bullet, and "A field shows its items in the `<items>` order, not the cache's, until a
//! sort applies"):
//! - `ascending` orders the texts as Excel compares them (`functions::util::collation::cmp_text`,
//!   the collation the caption filters compare by): Ant, Bee, Cat over a cache Bee, Ant, Cat
//!   (probes/t4130-pivot-sort SA) and over items written Cat, Bee, Ant (t4164-pivot-item-order
//!   IOA);
//! - `descending` is the reverse (SD; t4128-pivot-spike K, an inner field under each region);
//!   `nonAutoSortDefault` changes nothing visible (NA) and is not read;
//! - with the definition's `customListSort` on (the default) a text that is a name of a built-in
//!   custom list sorts by the list: the day names Mon, Wed, Fri ascending (CL1); with it off, by
//!   the alphabet, Fri, Mon, Wed (CL0). The lists are [`BUILT_IN_CUSTOM_LISTS`].
//!
//! A field with an `autoSortScope` is a value sort's (`super::value_sorts`), even when its scope
//! sorts nothing, as before T-4768; `manual` keeps the `<items>` order (SM).
//!
//! UNVERIFIED (unprobed, T-4799), the engine's own readings: a field of mixed item types sorts
//! numbers and dates (by value) before texts, then booleans (FALSE first), then errors, as the
//! worksheet's SORT does, and descending reverses them all; a blank (`(blank)`) item sorts last in
//! both directions; texts the collation ties (`apple`, `Apple`) and errors keep the cache order
//! ascending and the reverse descending; a custom-list name matches ignoring ASCII case, the listed
//! names sort before every other text (by list, in [`BUILT_IN_CUSTOM_LISTS`] order, then by
//! position) and every list but the day names (the abbreviated day names are evidenced, CL1) is
//! unprobed; a field grouped by ranges of numbers or dates sorts its groups in their own order
//! (the cache's), reversed descending, not by their labels' text; and a value filter's Top-N
//! breaks ties by this order, the field's display order, which is its `<items>` order unsorted
//! (pivots.md "Top-N filters at their edges").
//!
//! Every pass charges its work before doing it (CLAUDE.md directive 5).

use std::cmp::Ordering;

use crate::area_budget::compared_units;
use crate::functions::util::collation::{
    cmp_text, restore_compared_ticks, take_compared_ticks, BYTE_TICKS, CHAR_TICKS,
};
use crate::locale::excel_date_names::BUILT_IN_CUSTOM_LISTS;
use crate::text::Text;

use super::super::result::{charge, sort_units, units, Order, PivotComputeError};
use super::super::{CacheField, CacheItem, PivotField, PivotTable, SortType};

/// Field `field`'s display order before any value sort: its `<items>` order ([`Order::of`]), or
/// under a label sort the order of its items' values (see the module documentation). Charges
/// its work.
pub(in crate::pivot) fn display_order(
    table: &PivotTable,
    cache_field: &CacheField,
    field: u32,
    work: &mut u64,
) -> Result<Order, PivotComputeError> {
    let pivot_field = table.fields.get(field as usize);
    let listed = pivot_field.map_or(0, |f| f.items.len());
    let item_count = cache_field.items.len();
    charge(work, units(item_count.saturating_add(listed)))?;
    match label_sort(pivot_field) {
        Some(descending) => {
            let lists = table.options.custom_list_sort;
            Order::permuted(positions(cache_field, descending, lists, work)?)
        }
        None => Ok(Order::of(pivot_field, item_count)),
    }
}

/// Whether `field` is label-sorted, and if so whether descending.
fn label_sort(field: Option<&PivotField>) -> Option<bool> {
    let field = field.filter(|f| f.auto_sort.is_none())?;
    match field.sort_type {
        SortType::Ascending => Some(false),
        SortType::Descending => Some(true),
        SortType::Manual => None,
    }
}

/// What an item sorts by, in ascending class order.
enum Key<'a> {
    /// A number or a date, by value.
    Number(f64),
    /// A name of a built-in custom list: the list, then its position there.
    Listed(usize, usize),
    /// Any other text, by the collation.
    Text(&'a Text),
    Boolean(bool),
    Error,
    /// A blank (`<m/>`): last both ways.
    Missing,
}

impl Key<'_> {
    /// The class: a number before a listed name, before a text, a boolean and an error.
    fn class(&self) -> u8 {
        match self {
            Key::Number(_) => 0,
            Key::Listed(..) => 1,
            Key::Text(_) => 2,
            Key::Boolean(_) => 3,
            Key::Error => 4,
            Key::Missing => 5,
        }
    }

    /// The ascending order of two keys; equal ones are left to the cache order.
    fn cmp(&self, other: &Key<'_>) -> Ordering {
        match (self, other) {
            (Key::Number(a), Key::Number(b)) => (a + 0.0).total_cmp(&(b + 0.0)),
            (Key::Listed(la, pa), Key::Listed(lb, pb)) => la.cmp(lb).then(pa.cmp(pb)),
            (Key::Text(a), Key::Text(b)) => cmp_text(a, b),
            (Key::Boolean(a), Key::Boolean(b)) => a.cmp(b),
            _ => self.class().cmp(&other.class()),
        }
    }
}

/// The position of the built-in custom list name `text` matches, ignoring ASCII case: its list
/// and its place in it.
fn listed(text: &str) -> Option<(usize, usize)> {
    (BUILT_IN_CUSTOM_LISTS.iter().enumerate()).find_map(|(list, names)| {
        let at = names.iter().position(|n| n.eq_ignore_ascii_case(text))?;
        Some((list, at))
    })
}

/// Per cache item of `field` its position under the label sort (a permutation of the items),
/// `lists` saying whether the built-in custom lists apply.
fn positions(
    field: &CacheField,
    descending: bool,
    lists: bool,
    work: &mut u64,
) -> Result<Vec<u32>, PivotComputeError> {
    let count = field.items.len();
    let mut order: Vec<u32> =
        (0..u32::try_from(count).map_err(|_| PivotComputeError::Limit)?).collect();
    let grouped = field.number_groups.is_some() || field.date_groups.is_some();
    if !grouped {
        let prepared = prepare(field, lists, work)?;
        let keys: Vec<Key<'_>> = (field.items.iter().zip(&prepared))
            .map(|(item, prepared)| key(item, prepared))
            .collect();
        // Each comparison of two texts reads each at most twice as ASCII, once a character at
        // the full collation's weight otherwise (`collation::cmp_str`); the bound is charged
        // first, and the ticks the collation notes are dropped rather than left for a formula, as
        // the caption filters do (`filter::item_filter`).
        let (longest, ascii) = (prepared.iter()).fold((0, true), |(n, a), p| match p {
            Prepared::Text(_, len, is_ascii) => (n.max(*len), a && *is_ascii),
            _ => (n, a),
        });
        let per_text = if ascii { 2 * BYTE_TICKS } else { CHAR_TICKS };
        let ticks = units(longest)
            .saturating_mul(2 * per_text)
            .saturating_mul(sort_units(count));
        charge(
            work,
            sort_units(count).saturating_add(compared_units(ticks)),
        )?;
        let pending = take_compared_ticks();
        order.sort_by(|&a, &b| {
            let (ka, kb) = (keys.get(a as usize), keys.get(b as usize));
            let by_key = match (ka, kb) {
                (Some(ka), Some(kb)) => ka.cmp(kb),
                _ => Ordering::Equal,
            };
            by_key.then(a.cmp(&b))
        });
        take_compared_ticks();
        restore_compared_ticks(pending);
    }
    charge(work, units(count).saturating_mul(2))?;
    if descending {
        order.reverse();
        // A blank stays last (the field holds at most one).
        if let Some(at) = order
            .iter()
            .position(|&i| matches!(field.items.get(i as usize), Some(CacheItem::Missing)))
        {
            let blank = order.remove(at);
            order.push(blank);
        }
    }
    let mut positions = vec![0; count];
    for (position, &item) in (0u32..).zip(&order) {
        if let Some(slot) = positions.get_mut(item as usize) {
            *slot = position;
        }
    }
    Ok(positions)
}

/// An item prepared for the sort: a text the collation orders (its engine text, its length in
/// bytes and whether it is ASCII), a name of a built-in custom list (its list and position), or
/// any other item.
enum Prepared {
    Text(Text, usize, bool),
    Listed(usize, usize),
    Other,
}

/// Per item of `field`, what it is [`Prepared`] as; with `lists`, a name of a built-in custom
/// list is [`Prepared::Listed`]. Charged.
fn prepare(
    field: &CacheField,
    lists: bool,
    work: &mut u64,
) -> Result<Vec<Prepared>, PivotComputeError> {
    let names = BUILT_IN_CUSTOM_LISTS
        .iter()
        .map(|l| l.len())
        .fold(0, usize::saturating_add);
    let bytes = (field.items.iter())
        .map(|item| match item {
            CacheItem::Text(text) => text.len(),
            _ => 0,
        })
        .fold(0, usize::saturating_add);
    let lookups = if lists {
        units(field.items.len()).saturating_mul(units(names))
    } else {
        0
    };
    charge(work, units(bytes).saturating_add(lookups))?;
    Ok((field.items.iter())
        .map(|item| match item {
            CacheItem::Text(text) => match listed(text).filter(|_| lists) {
                Some((list, at)) => Prepared::Listed(list, at),
                None => Prepared::Text(Text::from(text.as_str()), text.len(), text.is_ascii()),
            },
            _ => Prepared::Other,
        })
        .collect())
}

/// What `item`, [`Prepared`] as `prepared`, sorts by.
fn key<'a>(item: &CacheItem, prepared: &'a Prepared) -> Key<'a> {
    match (prepared, item) {
        (Prepared::Text(text, ..), _) => Key::Text(text),
        (Prepared::Listed(list, at), _) => Key::Listed(*list, *at),
        (Prepared::Other, CacheItem::Number(v) | CacheItem::Date(v)) => Key::Number(*v),
        (Prepared::Other, CacheItem::Boolean(b)) => Key::Boolean(*b),
        (Prepared::Other, CacheItem::Missing) => Key::Missing,
        // An error (a text is always prepared as one of the others).
        (Prepared::Other, CacheItem::Error(_) | CacheItem::Text(_)) => Key::Error,
    }
}

#[cfg(test)]
mod tests;
