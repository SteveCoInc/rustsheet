use crate::arithmetic::{flush_subnormal, TINY_SUBNORMAL_BELOW};
use crate::cast::NumberOrArray;
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::{Error, OpUnary},
        types::CellReferenceIndex,
    },
    model::Model,
};

impl Model {
    /// Evaluates a unary `-` or `%` node. A chain such as `B:B%%%` or `---B:B` is applied
    /// in one element-wise pass, so a whole column is lifted once rather than once per
    /// operator (the `formula_eval` fuzz timeout, T-1078). Only the minus parity and the `%` count
    /// are kept, and the walk allocates nothing. Negation is exact and the subnormal flush is
    /// sign-symmetric, except that `-x` of a tiny positive x is +x; the chain applies that once
    /// per written group under an odd minus parity (T-3338), unprobed for `--x`, `-(-x)`, a tiny
    /// negative operand and references (numbers.md "Unary minus of a tiny subnormal"; TODO(T-2942)).
    pub(crate) fn apply_unary_chain(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut negate = false;
        // Groups under an odd count of minus signs: each a `-x` of its own, which keeps a tiny
        // x positive once it has made it so (T-3338).
        let mut negated_groups = 0_usize;
        let mut percents = 0_usize;
        let mut operand = node;
        loop {
            match operand {
                Node::UnaryKind { kind, right } => {
                    match kind {
                        OpUnary::Minus => negate = !negate,
                        OpUnary::Percentage => percents += 1,
                    }
                    operand = right;
                }
                // The identity: `-+-B:B` is one chain too, and so is `-(-(...))` (T-3338).
                Node::UnaryPlus { child } | Node::Spaced { child, .. } => operand = child,
                Node::Group { child, .. } => {
                    negated_groups += usize::from(std::mem::take(&mut negate));
                    operand = child;
                }
                _ => break,
            }
        }
        negated_groups += usize::from(negate);
        self.apply_number_unary(operand, cell, |mut f| {
            // A subnormal quotient is 0 like for `/` (numbers.md "Subnormal results
            // become 0", probes/subnormal-function-results A1, A21); 0 stays 0, so blank
            // cells stop early.
            for _ in 0..percents {
                if f == 0.0 {
                    break;
                }
                f = flush_subnormal(f / 100.0);
            }
            // Excel's `-x` of a positive x below 2^-1026 is +x, while `0-x` keeps the sign
            // (numbers.md "Unary minus of a tiny subnormal", probes/t2771-roundup-subnormal-sign-controls).
            if negated_groups > 0 && f != 0.0 && f.abs() < TINY_SUBNORMAL_BELOW {
                return Ok(f.abs());
            }
            Ok(if negated_groups % 2 == 1 { -f } else { f })
        })
    }

    /// Applies a scalar numeric transform element-wise over a node that may evaluate
    /// to a number, a range, or an array: the unary `-`/`%` chain ([`Model::apply_unary_chain`]).
    /// Functions lift through the SPEC-0011 driver instead (`functions/lift/`, T-1785).
    ///
    /// Element coercion matches the rest of the array machinery: booleans → 0/1,
    /// empty → 0, strings are parsed with `cast_number` (`#VALUE!` if they don't
    /// parse), and errors propagate.
    pub(crate) fn apply_number_unary(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        op: impl Fn(f64) -> Result<f64, Error>,
    ) -> CalcResult {
        match self.get_number_or_array(node, cell) {
            Ok(NumberOrArray::Number(f)) => match op(f) {
                Ok(x) => CalcResult::Number(x),
                Err(e) => CalcResult::new_error(e, cell, String::new()),
            },
            // Rewritten in place: a whole column is a million one-element rows, and
            // reallocating each of them per operator dominated the cost (T-1078).
            Ok(NumberOrArray::Array(mut array)) => {
                for value in array.iter_mut().flatten() {
                    let n = match value {
                        ArrayNode::Number(n) => Some(*n),
                        ArrayNode::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
                        ArrayNode::Empty => Some(0.0),
                        ArrayNode::String(s) => self.cast_number(s),
                        ArrayNode::Lambda(_) | ArrayNode::Nested(_) => None,
                        ArrayNode::Error(_) => continue,
                    };
                    *value = match n {
                        Some(f) => match op(f) {
                            Ok(x) => ArrayNode::Number(x),
                            Err(e) => ArrayNode::Error(e),
                        },
                        None => ArrayNode::Error(Error::VALUE),
                    };
                }
                CalcResult::Array(array)
            }
            Err(e) => e,
        }
    }
}
