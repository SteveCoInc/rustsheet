//! `Clone`, `PartialEq` and `Debug` for [`Node`] that walk a run of wrappers in a loop (T-3253,
//! T-4002).
//!
//! A formula may write [`super::MAX_SIGNS`] signs in a row, each a [`Node::UnaryKind`] or a
//! [`Node::UnaryPlus`], each with a [`Node::Spaced`] when whitespace follows it, under the
//! deepest nesting the depth bound admits. Signs cost no depth (T-3193), so a derived `Clone` or
//! `PartialEq`, which recurses once per node, took about 500 KB of a 1 MiB optimized stack for
//! such a run (920 KB when spaced; measured in T-3253). These impls peel the wrappers of a run
//! (signs, unary pluses, whitespace and parentheses) in a loop and recurse only into the other
//! nodes, whose nesting the depth bound limits. `PartialEq` loops down operator chains too, as
//! [`super::ChainLeft`]'s `Clone` does.
//!
//! `Debug` prints a run the same way, keeping the derived format byte for byte: the heads of the
//! run's wrappers in a loop, then the node under them, then their closing braces (T-4002).
//!
//! One list of every variant's fields generates the three impls' per-node arms, so adding a
//! variant or a field fails to compile here until the list has it.

use std::fmt::{self, Write as _};

use super::chain::value_operands;
use super::spacing::Spacing;
use super::Node;
use crate::expressions::token::OpUnary;

/// One wrapper of a run, without its child.
#[derive(PartialEq)]
enum Shell {
    Unary(OpUnary),
    Plus,
    Spaced(Spacing),
    Group(u16),
}

impl Shell {
    /// `child` inside this wrapper.
    fn wrap(self, child: Node) -> Node {
        let child = Box::new(child);
        match self {
            Shell::Unary(kind) => Node::UnaryKind { kind, right: child },
            Shell::Plus => Node::UnaryPlus { child },
            Shell::Spaced(space) => Node::Spaced { space, child },
            Shell::Group(levels) => Node::Group { levels, child },
        }
    }
}

impl Node {
    /// The wrapper `self` is and the node it wraps, or `None` when `self` is not a wrapper.
    fn shell(&self) -> Option<(Shell, &Node)> {
        match self {
            Node::UnaryKind { kind, right } => Some((Shell::Unary(kind.clone()), right)),
            Node::UnaryPlus { child } => Some((Shell::Plus, child)),
            Node::Spaced { space, child } => Some((Shell::Spaced(*space), child)),
            Node::Group { levels, child } => Some((Shell::Group(*levels), child)),
            _ => None,
        }
    }

    /// The number of wrappers in the run starting at `self`, and the node under them.
    fn run(&self) -> (usize, &Node) {
        let (mut count, mut node) = (0, self);
        while let Some((_, child)) = node.shell() {
            count += 1;
            node = child;
        }
        (count, node)
    }
}

impl Shell {
    /// What the derived `Debug` prints of this wrapper ahead of its child: the variant's name, its
    /// other field (name and value) if it has one, and the child field's name.
    fn head(
        &self,
    ) -> (
        &'static str,
        Option<(&'static str, &dyn fmt::Debug)>,
        &'static str,
    ) {
        match self {
            Shell::Unary(kind) => ("UnaryKind", Some(("kind", kind)), "right"),
            Shell::Plus => ("UnaryPlus", None, "child"),
            Shell::Spaced(space) => ("Spaced", Some(("space", space)), "child"),
            Shell::Group(levels) => ("Group", Some(("levels", levels)), "child"),
        }
    }
}

/// The shortest run `{:#?}` prints in a loop. A shorter one recurses per wrapper through
/// [`Node::fmt_one`], exactly as the derived `Debug` did under every flag; the loop prints through
/// a fresh formatter, which keeps `#` but not a `{:#x?}` hex flag (std has no stable way to read
/// it). A formula's shells under any nesting are fewer than this unless it writes a long run.
const PRETTY_LOOP_RUN: usize = 32;

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (count, inner) = self.run();
        if count == 0 {
            return self.fmt_one(f);
        }
        if !f.alternate() {
            // `Name { field: value, child: ` per wrapper, the inner node, then ` }` per wrapper:
            // what `debug_struct` writes, through `f` itself, so every flag reaches the values.
            let mut node = self;
            while let Some((shell, child)) = node.shell() {
                let (name, field, child_name) = shell.head();
                f.write_str(name)?;
                f.write_str(" { ")?;
                if let Some((field_name, value)) = field {
                    f.write_str(field_name)?;
                    f.write_str(": ")?;
                    value.fmt(f)?;
                    f.write_str(", ")?;
                }
                f.write_str(child_name)?;
                f.write_str(": ")?;
                node = child;
            }
            inner.fmt_one(f)?;
            for _ in 0..count {
                f.write_str(" }")?;
            }
            return Ok(());
        }
        let only_alternate = f.width().is_none()
            && f.precision().is_none()
            && !f.sign_plus()
            && !f.sign_minus()
            && !f.sign_aware_zero_pad()
            && f.fill() == ' '
            && f.align().is_none();
        if count < PRETTY_LOOP_RUN || !only_alternate {
            return self.fmt_one(f);
        }
        // `debug_struct`'s pretty form, with one [`Indent`] standing for the nested pad adapters
        // std stacks up per field: a line starts with four spaces per enclosing field.
        let mut out = Indent {
            out: f,
            depth: 0,
            on_newline: false,
        };
        let mut node = self;
        while let Some((shell, child)) = node.shell() {
            let (name, field, child_name) = shell.head();
            out.write_str(name)?;
            out.write_str(" {\n")?;
            out.depth += 1;
            if let Some((field_name, value)) = field {
                write!(out, "{field_name}: {value:#?},\n")?;
            }
            out.write_str(child_name)?;
            out.write_str(": ")?;
            node = child;
        }
        write!(out, "{inner:#?}")?;
        for _ in 0..count {
            out.write_str(",\n")?;
            out.depth -= 1;
            out.write_str("}")?;
        }
        Ok(())
    }
}

/// A writer that starts each line with four spaces per level of `depth`, as std's pad adapter
/// does per level of nesting in `{:#?}`; `depth` changes as a run's wrappers open and close.
struct Indent<'a, 'b> {
    out: &'a mut fmt::Formatter<'b>,
    depth: usize,
    on_newline: bool,
}

impl fmt::Write for Indent<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for line in text.split_inclusive('\n') {
            if self.on_newline {
                // Four spaces per level, written up to 64 at a time.
                const SPACES: &str =
                    "                                                                ";
                let mut spaces = self.depth.saturating_mul(4);
                while spaces > 0 {
                    let chunk = spaces.min(SPACES.len());
                    self.out.write_str(&SPACES[..chunk])?;
                    spaces -= chunk;
                }
            }
            self.on_newline = line.ends_with('\n');
            self.out.write_str(line)?;
        }
        Ok(())
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        let Some((shell, mut node)) = self.shell() else {
            return self.clone_one();
        };
        let mut shells = vec![shell];
        while let Some((shell, child)) = node.shell() {
            shells.push(shell);
            node = child;
        }
        let mut copy = node.clone_one();
        while let Some(shell) = shells.pop() {
            copy = shell.wrap(copy);
        }
        copy
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        let (mut left, mut right) = (self, other);
        loop {
            match (left.shell(), right.shell()) {
                (Some((a, left_child)), Some((b, right_child))) => {
                    if a != b {
                        return false;
                    }
                    left = left_child;
                    right = right_child;
                    continue;
                }
                (None, None) => {}
                _ => return false,
            }
            // Down an operator chain's left operands in the same loop, as `ChainLeft`'s `Clone`
            // does: a derived comparison took a frame per link of a 4,000-link chain.
            match (value_operands(left), value_operands(right)) {
                (Some((left_first, left_last)), Some((right_first, right_last))) => {
                    if !same_operator(left, right) || left_last != right_last {
                        return false;
                    }
                    left = left_first;
                    right = right_first;
                }
                (None, None) => return left.eq_one(right),
                _ => return false,
            }
        }
    }
}

/// Whether two value operators are the same operator (their operands aside).
fn same_operator(left: &Node, right: &Node) -> bool {
    match (left, right) {
        (Node::OpSumKind { kind: a, .. }, Node::OpSumKind { kind: b, .. }) => a == b,
        (Node::OpProductKind { kind: a, .. }, Node::OpProductKind { kind: b, .. }) => a == b,
        (Node::CompareKind { kind: a, .. }, Node::CompareKind { kind: b, .. }) => a == b,
        (Node::OpPowerKind { .. }, Node::OpPowerKind { .. })
        | (Node::OpConcatenateKind { .. }, Node::OpConcatenateKind { .. }) => true,
        _ => false,
    }
}

/// `clone_one`, `eq_one` and `fmt_one` for every variant, from one list of each variant's fields: struct
/// variants by their field names, then the one-value tuple variants, then the unit variant.
macro_rules! one_node_walks {
    (
        $($variant:ident { $($field:ident),+ })+
        ; $($tuple:ident)+
        ; $unit:ident
    ) => {
        impl Node {
            /// A copy of `self` that clones its children with [`Node::clone`]. Reached for a
            /// wrapper only through [`Node::clone`], which peels wrappers first; cloning one here
            /// is still right.
            fn clone_one(&self) -> Node {
                match self {
                    $(Node::$variant { $($field),+ } => Node::$variant {
                        $($field: Clone::clone($field)),+
                    },)+
                    $(Node::$tuple(value) => Node::$tuple(Clone::clone(value)),)+
                    Node::$unit => Node::$unit,
                }
            }

            /// Whether `self` and `other` are equal, comparing their children with [`Node::eq`].
            /// Reached for wrappers and operator links only through [`Node::eq`], which peels
            /// them first; comparing them here is still right.
            fn eq_one(&self, other: &Node) -> bool {
                match self {
                    $(Node::$variant { $($field),+ } => {
                        let mine = ($($field,)+);
                        // The same names, bound now to `other`'s fields.
                        let Node::$variant { $($field),+ } = other else {
                            return false;
                        };
                        mine == ($($field,)+)
                    })+
                    $(Node::$tuple(value) => {
                        matches!(other, Node::$tuple(theirs) if value == theirs)
                    })+
                    Node::$unit => matches!(other, Node::$unit),
                }
            }

            /// What the derived `Debug` prints for `self`, its children printed with
            /// [`Node::fmt`]. Reached for a wrapper of a long run only through [`Node::fmt`],
            /// which prints the run in a loop.
            fn fmt_one(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Node::$variant { $($field),+ } => f
                        .debug_struct(stringify!($variant))
                        $(.field(stringify!($field), $field))+
                        .finish(),)+
                    $(Node::$tuple(value) => {
                        f.debug_tuple(stringify!($tuple)).field(value).finish()
                    })+
                    Node::$unit => f.write_str(stringify!($unit)),
                }
            }
        }
    };
}

// Every variant of `Node`, its fields in declaration order (the derived `Debug` prints them in
// that order): a variant or a field missing here fails to compile.
one_node_walks! {
    ReferenceKind { sheet_name, sheet_index, absolute_row, absolute_column, row, column }
    RangeKind { sheet_name, sheet_index, absolute_row1, absolute_column1, row1, column1, absolute_row2, absolute_column2, row2, column2 }
    SheetSpanKind { first_sheet, last_sheet, first_index, last_index, reference }
    WrongReferenceKind { sheet_name, absolute_row, absolute_column, row, column }
    WrongRangeKind { sheet_name, absolute_row1, absolute_column1, row1, column1, absolute_row2, absolute_column2, row2, column2 }
    OpRangeKind { left, right }
    OpIntersectKind { left, right }
    OpUnionKind { left, right, bare }
    OpConcatenateKind { left, right }
    OpSumKind { kind, left, right }
    OpProductKind { kind, left, right }
    OpPowerKind { left, right }
    FunctionKind { kind, args }
    LambdaDefKind { parameters, body }
    LambdaCallKind { lambda, args }
    NamedFunctionKind { id, name, args }
    NamedVariableKind { name, id }
    ImplicitIntersection { automatic, child }
    SpillRangeOperator { child }
    Group { levels, child }
    StructuredReferenceKind { reference, lowered }
    ThisRowKind { sheet_name, sheet_index, column1, column2, data_start, data_end, cell }
    CompareKind { kind, left, right }
    UnaryKind { kind, right }
    UnaryPlus { child }
    Spaced { space, child }
    SheetRefErrorKind { sheet_name, missing }
    ParseErrorKind { formula, message, position, expecting }
    ; BooleanKind NumberKind StringKind ArrayKind DefinedNameKind ErrorKind
    ; EmptyArgKind
}
