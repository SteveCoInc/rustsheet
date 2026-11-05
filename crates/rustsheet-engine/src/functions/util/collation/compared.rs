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

/// Ticks of one byte a byte-wise pass reads: the ASCII first pass, the ASCII second pass (equal
/// or case-only-different texts, [`super::ascii_skipped`]), [`cmp_prefix`], the ASCII folds and
/// wildcard matches of the lookups and criteria (1.1 to 1.6 ns a byte; the second pass about 1 ns
/// a byte since T-4279, 14 ns a character before).
pub(crate) const BYTE_TICKS: u64 = 1;

/// Ticks of one character the full collation reads (non-ASCII text or a lone surrogate). Its first
/// pass reads each character once and keeps its elements for the later passes (T-4279), so this is
/// the whole comparison's cost a character: about 70 ns at the worst, two texts equal to the end
/// (`é`, `中`, release build, 2026-11-05; 95 to 140 ns a character for each of the six passes
/// before T-4279, weighed 128).
pub(super) const CHAR_TICKS: u64 = 48;

/// Ticks of one byte a [`super::CollationKey`] is built from (UNIQUE, the exact-match lookup
/// index, the *IF(S) cache): about 50 ns a byte for ASCII and 37 for `é` once the elements are
/// made once (T-4279; 224 ns and a weight of 128 before). The weight is twice that because a short
/// key's own costs (its vectors, the hashing) weigh as much again: so a UNIQUE of a million 27-byte
/// texts still computes, and one of a million 107-byte texts, which runs 13 s at 32 ticks, is
/// refused as before (T-4233); a bound-array UNIQUE of 32,767-byte texts reaches the limit in
/// about 3 s.
pub(super) const KEY_TICKS: u64 = 64;

/// Ticks of one byte [`super::fold_lookup`] folds (non-ASCII: about 31 ns).
pub(super) const FOLD_TICKS: u64 = 32;

thread_local! {
    /// The ticks of the comparisons, keys and folds since the evaluator last took them
    /// ([`take_compared_ticks`]).
    static COMPARED_TICKS: Cell<u64> = const { Cell::new(0) };
    /// The most ticks the formula run can still pay for at its next charge
    /// ([`set_compared_allowance`]); no bound outside a run.
    static COMPARED_ALLOWANCE: Cell<u64> = const { Cell::new(u64::MAX) };
    /// Whether a reader stopped reading because the pending ticks passed the allowance
    /// ([`out_of_ticks`]): the run's value is then the work budget's error, whatever the reader
    /// returned ([`take_cut_short`]).
    static CUT_SHORT: Cell<bool> = const { Cell::new(false) };
}

/// Sets the ticks the formula run can still pay for at its next charge (its work units left
/// times `area_budget::COMPARED_TICKS_PER_UNIT`) and returns the one it replaces. Pending ticks
/// past it make that charge fail, so a reader may stop reading as soon as they pass it
/// ([`out_of_ticks`], T-4278). It may be more than the run has left (units spent since it was set
/// lower what is left), never less: the evaluator sets it at each node's charge and at the start
/// of each run, and puts the interrupted run's back afterwards.
#[inline]
pub(crate) fn set_compared_allowance(ticks: u64) -> u64 {
    COMPARED_ALLOWANCE.with(|allowance| allowance.replace(ticks))
}

/// Whether the ticks noted since the last charge already pass what the run can pay for
/// ([`set_compared_allowance`]): the node will be the work budget's error whatever it computes, so
/// a reader returns at once, without reading, instead of reading to its end first (T-4278). Marks
/// the run cut short ([`take_cut_short`]), so its value is that error even if the charge that
/// would fail never comes.
#[inline]
pub(super) fn out_of_ticks() -> bool {
    let out = COMPARED_TICKS.with(Cell::get) > COMPARED_ALLOWANCE.with(Cell::get);
    if out {
        CUT_SHORT.with(|cut| cut.set(true));
    }
    out
}

/// Whether a reader of the run stopped short since the last call ([`out_of_ticks`]), and clears it.
#[inline]
pub(crate) fn take_cut_short() -> bool {
    CUT_SHORT.with(|cut| cut.replace(false))
}

/// Marks the run cut short again: an interrupted run's flag, put back after a dependency cell's
/// run ([`take_cut_short`]).
#[inline]
pub(crate) fn restore_cut_short(cut: bool) {
    if cut {
        CUT_SHORT.with(|flag| flag.set(true));
    }
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
    read_ticks(count.saturating_mul(ticks));
}

/// Adds `ticks` to the pending count.
fn add_ticks(ticks: u64) {
    COMPARED_TICKS.with(|total| total.set(total.get().saturating_add(ticks)));
}

/// Adds `ticks` a reader read to the pending count ([`add_ticks`]), and, in tests, to the count
/// of every tick read ([`take_read_ticks`]).
fn read_ticks(ticks: u64) {
    add_ticks(ticks);
    #[cfg(test)]
    READ_TICKS.with(|total| total.set(total.get().saturating_add(ticks)));
}

#[cfg(test)]
thread_local! {
    /// Every tick the readers read since [`take_read_ticks`] last took them, charged or not.
    static READ_TICKS: Cell<u64> = const { Cell::new(0) };
}

/// The ticks the readers read since the last call, for the tests of T-4278: how far a reader
/// read before it stopped.
#[cfg(test)]
pub(crate) fn take_read_ticks() -> u64 {
    READ_TICKS.with(|total| total.replace(0))
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
///
/// Past the run's allowance, Excel's pairwise order returns `Equal` without reading ([`out_of_ticks`],
/// T-4278): its callers already take any order (the lookups' scans, `=`, SORT's merge,
/// `sort_indices_pairwise`), and the run's value is the work budget's error. The total order does
/// not, since `slice::sort_by` may panic on a comparator that is not one (GROUPBY's sorts).
pub(super) fn cmp_text_in(left: &Text, right: &Text, order: AccentOrder) -> Ordering {
    if let AccentOrder::Pairwise = order {
        if out_of_ticks() {
            return Ordering::Equal;
        }
    }
    let read = Cell::new(0);
    let ordering = cmp_text_counted(left, right, order, &read);
    read_ticks(read.get());
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
