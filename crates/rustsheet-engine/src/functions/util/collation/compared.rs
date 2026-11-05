//! What text comparisons read, for the formula work budget (SPEC-0024 section 1, "text values";
//! T-4233; docs/excel-semantics/arrays.md "One formula evaluation does at most 268,435,456 work
//! units").
//!
//! The count is in *ticks*, each about the time of one byte of the ASCII first pass (about 1.6 ns,
//! release build): the paths differ by two orders of magnitude a character, so each notes what it
//! read times its own weight below, fitted by time (T-4233; the timings are in arrays.md). The
//! evaluator charges one work unit per `area_budget::COMPARED_TICKS_PER_UNIT` ticks.

use std::cell::Cell;
use std::cmp::Ordering;

use super::{cmp_chars, cmp_prefix, cmp_str, AccentOrder};
use crate::text::Text;

/// Ticks of one byte a byte-wise pass reads: the ASCII first pass, [`cmp_prefix`], the ASCII
/// folds and wildcard matches of the lookups and criteria (1.1 to 1.6 ns a byte).
pub(crate) const BYTE_TICKS: u64 = 1;

/// Ticks of one character the ASCII second pass reads (`elements`; about 14 ns, as two equal
/// ASCII texts in another case walk it to the end).
pub(super) const ELEMENT_TICKS: u64 = 32;

/// Ticks of one character the full collation reads (non-ASCII text or a lone surrogate: about 95
/// to 140 ns, `é`, `中`).
pub(super) const CHAR_TICKS: u64 = 128;

/// Ticks of one byte a [`super::CollationKey`] is built from (UNIQUE, the exact-match lookup
/// index, the *IF(S) cache). Its bound-array worst case reaches the limit in about 7 s, as the
/// other paths' do, while a UNIQUE of a million 27-byte texts still computes (T-4233).
pub(super) const KEY_TICKS: u64 = 128;

/// Ticks of one byte [`super::fold_lookup`] folds (non-ASCII: about 31 ns).
pub(super) const FOLD_TICKS: u64 = 32;

thread_local! {
    /// The ticks of the comparisons, keys and folds since the evaluator last took them
    /// ([`take_compared_ticks`]).
    static COMPARED_TICKS: Cell<u64> = const { Cell::new(0) };
}

/// Notes `count` bytes or characters read by a comparison, a key or a fold, at `ticks` each (the
/// weights above). The readers that compare texts (the lookups, MATCH, UNIQUE, SORT, the criteria,
/// GROUPBY) all go through this module or
/// [`folded_units`](crate::functions::criteria::folded_units), so the evaluator charges what they
/// read here, at one place, after each node (SPEC-0024 section 1, "text values"; T-4233): the
/// cell charge of a bound array does not cover comparing its 32,767-byte texts again and again.
#[inline]
pub(crate) fn note_compared(count: usize, ticks: u64) {
    let count = u64::try_from(count).unwrap_or(u64::MAX);
    add_ticks(count.saturating_mul(ticks));
}

/// Adds `ticks` to the pending count.
fn add_ticks(ticks: u64) {
    COMPARED_TICKS.with(|total| total.set(total.get().saturating_add(ticks)));
}

/// The ticks noted ([`note_compared`]) since the last call, and starts again from 0.
#[inline]
pub(crate) fn take_compared_ticks() -> u64 {
    COMPARED_TICKS.with(|total| total.replace(0))
}

/// Puts back `ticks` taken by [`take_compared_ticks`] (a dependency cell's run keeps the outer
/// run's pending ticks apart, `Model::with_area_budget`).
#[inline]
pub(crate) fn restore_compared_ticks(ticks: u64) {
    add_ticks(ticks);
}

/// `items`, adding `ticks` to `read` for each item taken, in every clone: a comparison's passes walk
/// clones of the two texts and stop at their first difference, so `read` ends as the work they did,
/// which the work budget charges ([`note_compared`], T-4233).
pub(super) fn counted<'a, I: Iterator + Clone + 'a>(
    items: I,
    read: &'a Cell<u64>,
    ticks: u64,
) -> impl Iterator<Item = I::Item> + Clone + 'a {
    items.inspect(move |_| read.set(read.get().saturating_add(ticks)))
}

/// [`super::cmp_text`] or [`super::cmp_text_pairwise`], noting the ticks it read.
pub(super) fn cmp_text_in(left: &Text, right: &Text, order: AccentOrder) -> Ordering {
    let read = Cell::new(0);
    let ordering = cmp_text_counted(left, right, order, &read);
    add_ticks(read.get());
    ordering
}

/// [`cmp_text_in`], counting the ticks its passes read in `read`.
fn cmp_text_counted(left: &Text, right: &Text, order: AccentOrder, read: &Cell<u64>) -> Ordering {
    if let AccentOrder::Pairwise = order {
        let prefix = cmp_prefix(
            counted(left.chars_lossless(), read, BYTE_TICKS),
            counted(right.chars_lossless(), read, BYTE_TICKS),
        );
        if let Some(order) = prefix {
            return order;
        }
    }
    if left.has_lone_surrogate() || right.has_lone_surrogate() {
        cmp_chars(
            counted(left.chars_lossless(), read, CHAR_TICKS),
            counted(right.chars_lossless(), read, CHAR_TICKS),
            order,
        )
    } else {
        cmp_str(left, right, order, read)
    }
}
