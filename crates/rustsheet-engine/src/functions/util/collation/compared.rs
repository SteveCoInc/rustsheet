//! What text comparisons read, for the formula work budget (SPEC-0024 section 1, "text values";
//! T-4233; docs/excel-semantics/arrays.md "One formula evaluation does at most 268,435,456 work
//! units").

use std::cell::Cell;
use std::cmp::Ordering;

use super::{cmp_chars, cmp_prefix, cmp_str, AccentOrder};
use crate::text::Text;

thread_local! {
    /// The text bytes compared, keyed or folded since the evaluator last took them
    /// ([`take_compared_bytes`]).
    static COMPARED_BYTES: Cell<u64> = const { Cell::new(0) };
}

/// Notes `bytes` text bytes read by a comparison, a key or a fold. The readers that compare texts
/// (the lookups, MATCH, UNIQUE, SORT, the criteria, GROUPBY) all go through this module or
/// [`folded_units`](crate::functions::criteria::folded_units), so the evaluator charges what they
/// read here, at one place, after each node (SPEC-0024 section 1, "text values"; T-4233): the
/// cell charge of a bound array does not cover comparing its 32,767-byte texts again and again.
#[inline]
pub(crate) fn note_compared_bytes(bytes: usize) {
    let bytes = u64::try_from(bytes).unwrap_or(u64::MAX);
    COMPARED_BYTES.with(|total| total.set(total.get().saturating_add(bytes)));
}

/// The bytes noted ([`note_compared_bytes`]) since the last call, and starts again from 0.
#[inline]
pub(crate) fn take_compared_bytes() -> u64 {
    COMPARED_BYTES.with(|total| total.replace(0))
}

/// `items`, adding one to `read` for each item taken, in every clone: a comparison's passes walk
/// clones of the two texts and stop at their first difference, so `read` ends as the work they did,
/// which the work budget charges ([`note_compared_bytes`], T-4233).
pub(super) fn counted<'a, I: Iterator + Clone + 'a>(
    items: I,
    read: &'a Cell<u64>,
) -> impl Iterator<Item = I::Item> + Clone + 'a {
    items.inspect(move |_| read.set(read.get().saturating_add(1)))
}

/// [`super::cmp_text`] or [`super::cmp_text_pairwise`], noting the characters it read.
pub(super) fn cmp_text_in(left: &Text, right: &Text, order: AccentOrder) -> Ordering {
    let read = Cell::new(0);
    let ordering = cmp_text_counted(left, right, order, &read);
    note_compared_bytes(usize::try_from(read.get()).unwrap_or(usize::MAX));
    ordering
}

/// [`cmp_text_in`], counting the characters its passes read in `read`.
fn cmp_text_counted(left: &Text, right: &Text, order: AccentOrder, read: &Cell<u64>) -> Ordering {
    if let AccentOrder::Pairwise = order {
        let prefix = cmp_prefix(
            counted(left.chars_lossless(), read),
            counted(right.chars_lossless(), read),
        );
        if let Some(order) = prefix {
            return order;
        }
    }
    if left.has_lone_surrogate() || right.has_lone_surrogate() {
        cmp_chars(
            counted(left.chars_lossless(), read),
            counted(right.chars_lossless(), read),
            order,
        )
    } else {
        cmp_str(left, right, order, read)
    }
}
