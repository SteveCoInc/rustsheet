//! Sorting and grouping under a comparison that need not be transitive: Excel's text comparison
//! (docs/excel-semantics/coercion.md "Not a total order").

use std::cmp::Ordering;

/// Sorts `indices` stably by `cmp`, which need not be a total order: Excel's text comparison is
/// not transitive (docs/excel-semantics/coercion.md "Not a total order"), and `slice::sort_by`
/// may panic on such a comparator. A natural merge sort: the input is cut into maximal
/// non-descending runs (a strictly descending stretch is runs of one element, never reversed), and
/// neighbouring runs are merged pairwise, bottom-up, a tie taking the left run's element, until
/// one is left. On any comparator it ends with a permutation of `indices`, in O(n log n)
/// comparisons and one buffer of n indices. On texts holding comparison cycles this is Excel's
/// SORT and SORTBY order, ascending and descending: every row of
/// probes/t3211-collation-sort-cycles and of probes/t3418-sort-cycle-permutations (every order
/// of three 4- and 5-text sets). Reversing strictly descending runs, the T-3211 model, misses 28
/// of those 168 rows, since on a cycle a strictly descending chain is not sorted (T-3418).
pub(crate) fn sort_indices_pairwise(
    indices: &mut [usize],
    mut cmp: impl FnMut(usize, usize) -> Ordering,
) {
    let n = indices.len();
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut start = 0;
    while start < n {
        let mut end = start + 1;
        while end < n && cmp(indices[end], indices[end - 1]) != Ordering::Less {
            end += 1;
        }
        runs.push((start, end));
        start = end;
    }
    let mut merged: Vec<usize> = Vec::with_capacity(n);
    while runs.len() > 1 {
        let mut next = Vec::with_capacity(runs.len().div_ceil(2));
        for pair in runs.chunks(2) {
            let &[(lo, mid), (_, hi)] = pair else {
                next.extend_from_slice(pair);
                continue;
            };
            merged.clear();
            let (mut i, mut j) = (lo, mid);
            while i < mid && j < hi {
                if cmp(indices[j], indices[i]) == Ordering::Less {
                    merged.push(indices[j]);
                    j += 1;
                } else {
                    merged.push(indices[i]);
                    i += 1;
                }
            }
            merged.extend_from_slice(&indices[i..mid]);
            merged.extend_from_slice(&indices[j..hi]);
            indices[lo..hi].copy_from_slice(&merged);
            next.push((lo, hi));
        }
        runs = next;
    }
}

/// For each of `count` lines, the smallest line of its group, where lines group as Excel's
/// UNIQUE groups them under a comparison `cmp` that need not be transitive (T-4200, T-4393): the
/// natural merge of [`sort_indices_pairwise`] over groups. A line `cmp` finds equal to the last
/// group of its run joins that group. When the heads of two runs being merged are equal, the
/// right head's group joins the left head's, which stays the head and meets the next right head
/// too; a group taken out of a merge is never compared with the one taken out before it. A group
/// is compared through the line that started it. On a total order this is grouping by equality.
/// On the comparison cycles of probes/t4200-unique-cycle-permutations it is Excel's grouping in
/// every row, where grouping the neighbours of SORT's order is not (`{a͂;a̧;a͂͂}` sorts to a̧, a͂,
/// a͂͂, yet Excel groups a̧ with a͂͂: rows 146 and 150), and on the two-field intransitive runs of
/// probes/t4379, t4380 and the t4393 hold-out (docs/excel-semantics/coercion.md, "UNIQUE's
/// merge: equal heads join, the output never coalesces"). `cmp` errors end the walk (a work
/// budget's). It makes O(n log n) comparisons and holds O(n) indices.
pub(crate) fn coalesce_equal_lines<E>(
    count: usize,
    mut cmp: impl FnMut(usize, usize) -> Result<Ordering, E>,
) -> Result<Vec<usize>, E> {
    /// Appends `group` to the run `out`, or makes it part of `out`'s last group when the two are
    /// equal (run building only: merges never coalesce, T-4393).
    fn push<E>(
        parent: &mut [usize],
        out: &mut Vec<usize>,
        group: usize,
        cmp: &mut impl FnMut(usize, usize) -> Result<Ordering, E>,
    ) -> Result<(), E> {
        if let Some(&last) = out.last() {
            if cmp(last, group)? == Ordering::Equal {
                parent[group] = last;
                return Ok(());
            }
        }
        out.push(group);
        Ok(())
    }
    /// The root of `line`'s group, halving the path to it.
    fn root(parent: &mut [usize], mut line: usize) -> usize {
        while parent[line] != line {
            parent[line] = parent[parent[line]];
            line = parent[line];
        }
        line
    }
    // A union-find over the lines; each group's root is the line that started it.
    let mut parent: Vec<usize> = (0..count).collect();
    let mut runs: Vec<Vec<usize>> = Vec::new();
    let mut run: Vec<usize> = Vec::new();
    for line in 0..count {
        if let Some(&last) = run.last() {
            if cmp(line, last)? == Ordering::Less {
                runs.push(std::mem::take(&mut run));
            }
        }
        push(&mut parent, &mut run, line, &mut cmp)?;
    }
    runs.push(run);
    while runs.len() > 1 {
        let mut next = Vec::with_capacity(runs.len().div_ceil(2));
        let mut pending = runs.into_iter();
        while let Some(left) = pending.next() {
            let Some(right) = pending.next() else {
                next.push(left);
                break;
            };
            let mut out = Vec::with_capacity(left.len() + right.len());
            let (mut i, mut j) = (0, 0);
            while let (Some(&a), Some(&b)) = (left.get(i), right.get(j)) {
                match cmp(b, a)? {
                    Ordering::Less => {
                        out.push(b);
                        j += 1;
                    }
                    Ordering::Greater => {
                        out.push(a);
                        i += 1;
                    }
                    // The right head joins the left head, which stays and meets the next right
                    // head too (T-4393: over Cy Cx Ay Bx Cy, the last Cy joins Cy).
                    Ordering::Equal => {
                        parent[b] = a;
                        j += 1;
                    }
                }
            }
            // Nothing out of a merge is compared with the group out before it (T-4393: over
            // Cy Cx Ay Bx Ay, Bx follows Cx though Bx = Cx, and the second Ay, < Bx, never meets
            // Cy and is kept).
            out.extend_from_slice(left.get(i..).unwrap_or_default());
            out.extend_from_slice(right.get(j..).unwrap_or_default());
            next.push(out);
        }
        runs = next;
    }
    let mut smallest = vec![usize::MAX; count];
    for line in 0..count {
        let r = root(&mut parent, line);
        smallest[r] = smallest[r].min(line);
    }
    Ok((0..count)
        .map(|line| smallest[root(&mut parent, line)])
        .collect())
}
