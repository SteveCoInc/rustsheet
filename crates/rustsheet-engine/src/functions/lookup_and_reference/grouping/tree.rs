//! The grouping tree of [`Grouping::new`](super::Grouping::new): classes built field by field
//! over UNIQUE's sorted keys, and each row's descent to its leaf (T-4241, T-4342, T-4344).

use std::cmp::Ordering;

use super::Keys;
use crate::calc_result::CalcResult;

/// A class of the grouping tree ([`Grouping::new`](super::Grouping::new)): the keys under one parent whose field `=`
/// the field of the first of them.
pub(super) struct Node {
    /// The line of the class's first key, which it shows and every comparison reads.
    pub(super) key: usize,
    /// Its index in its level.
    pub(super) id: usize,
    /// Its position among its parent's classes.
    pub(super) rank: usize,
    /// Its parent's index in the level above.
    parent: usize,
    /// Its classes in the level below, as a range of that level.
    children: (usize, usize),
}

/// The grouping tree over UNIQUE's sorted keys, one level per key field ([`Grouping::new`](super::Grouping::new)).
pub(super) struct Tree {
    levels: Vec<Vec<Node>>,
    /// The classes of the first field, as a range of the first level.
    roots: (usize, usize),
}

impl Tree {
    /// Builds the classes field by field over `sorted`: under one parent, a key joins the last
    /// class when its field `=` that class's first key's, else starts a class. So a class can
    /// repeat a shown key: Bx, Ay, Cy, Cx gives (C,x), (C,y), (C,x) (t4241 Calc!B25). Fails with
    /// `limit`'s error at the first leaf that does not fit.
    pub(super) fn build(
        lines: &Keys<'_>,
        sorted: &[usize],
        fields: usize,
        limit: impl Fn(usize) -> Option<CalcResult>,
        mut spend: impl FnMut(u64) -> Result<(), CalcResult>,
    ) -> Result<Self, CalcResult> {
        let mut tree = Tree {
            levels: Vec::with_capacity(fields),
            roots: (0, 0),
        };
        // The span of `sorted` each class of the level above holds (the root holds it all).
        let mut spans = vec![(0, sorted.len())];
        for field in 0..fields {
            let mut level: Vec<Node> = Vec::new();
            let mut next_spans: Vec<(usize, usize)> = Vec::new();
            for (parent, &(start, end)) in spans.iter().enumerate() {
                let begin = level.len();
                for (position, &line) in sorted.iter().enumerate().take(end).skip(start) {
                    let joins = match level.last() {
                        Some(class) if level.len() > begin => {
                            spend(lines.units(class.key, line, field))?;
                            lines.cmp(class.key, line, field) == Ordering::Equal
                        }
                        _ => false,
                    };
                    if joins {
                        if let Some(span) = next_spans.last_mut() {
                            span.1 = position + 1;
                        }
                        continue;
                    }
                    if field + 1 == fields {
                        if let Some(error) = limit(level.len() + 1) {
                            return Err(error);
                        }
                    }
                    level.push(Node {
                        key: line,
                        id: level.len(),
                        rank: level.len() - begin,
                        parent,
                        children: (0, 0),
                    });
                    next_spans.push((position, position + 1));
                }
                let range = (begin, level.len());
                match tree.levels.last_mut() {
                    Some(above) => {
                        if let Some(class) = above.get_mut(parent) {
                            class.children = range;
                        }
                    }
                    None => tree.roots = range,
                }
            }
            tree.levels.push(level);
            spans = next_spans;
        }
        Ok(tree)
    }

    /// The number of leaves (classes of the last field).
    pub(super) fn leaves(&self) -> usize {
        self.levels.last().map_or(0, Vec::len)
    }

    /// The classes from the first field down to leaf `leaf`.
    pub(super) fn path(&self, leaf: usize) -> Vec<&Node> {
        let mut path = Vec::with_capacity(self.levels.len());
        let mut index = leaf;
        for level in self.levels.iter().rev() {
            let Some(node) = level.get(index) else {
                break;
            };
            path.push(node);
            index = node.parent;
        }
        path.reverse();
        path
    }

    /// The leaf line `line` reaches: at each field a lower-bound search over its parent's classes
    /// in the direction the first two of them run in (the field's, unless the run is
    /// intransitive: T-4344), over [0, n-1] so it never runs off the end, then `=` (t4342
    /// Calc!B14, B48: the half-open search over [0, n) puts their rows in the other repeated
    /// class). It stops where the class found is not `=` to the line's key: Cy, Ax, By leaves row 3
    /// (B = C) in no group (t4241 Calc!B14).
    ///
    /// Returns the ids of the classes the line reached, one per field from the first: all
    /// `fields` of them when it reached a leaf (the last is the leaf), fewer when a search found a
    /// class not `=` to it. A PIVOTBY subtotal column counts a line that reached its class but no
    /// leaf under it (probes/t4358-pivotby-unreached-column-leaf column D; T-4378).
    pub(super) fn descend(
        &self,
        lines: &Keys<'_>,
        line: usize,
        fields: usize,
        mut spend: impl FnMut(u64) -> Result<(), CalcResult>,
    ) -> Result<Vec<usize>, CalcResult> {
        let (mut start, mut end) = self.roots;
        let mut reached = Vec::with_capacity(fields);
        for field in 0..fields {
            let Some(level) = self.levels.get(field) else {
                return Ok(reached);
            };
            if start >= end {
                return Ok(reached);
            }
            // The search runs in the direction of the first two classes, not the field's: on an
            // intransitive run the merge can leave them the other way round (t4342 Calc!C40;
            // probes/t4344-groupby-descending-search B1:B30, docs/excel-semantics/coercion.md).
            let before = match (level.get(start), level.get(start + 1)) {
                (Some(first), Some(second)) if start + 1 < end => {
                    spend(lines.units(first.key, second.key, field))?;
                    if lines.cmp(first.key, second.key, field) == Ordering::Greater {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                }
                _ => Ordering::Less,
            };
            let (mut low, mut high) = (start, end - 1);
            while low < high {
                let middle = low + (high - low) / 2;
                let Some(class) = level.get(middle) else {
                    break;
                };
                spend(lines.units(class.key, line, field))?;
                if lines.cmp(class.key, line, field) == before {
                    low = middle + 1;
                } else {
                    high = middle;
                }
            }
            let Some(class) = level.get(low) else {
                return Ok(reached);
            };
            spend(lines.units(class.key, line, field))?;
            if lines.cmp(class.key, line, field) != Ordering::Equal {
                return Ok(reached);
            }
            reached.push(low);
            (start, end) = class.children;
        }
        Ok(reached)
    }
}
