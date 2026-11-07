use std::sync::Arc;

use crate::arithmetic::{extended_product, extended_sum, flush_subnormal};
use crate::functions::statistical::ifs_cache::BlockNumbers;
use crate::limits::check_array_size;
use crate::nested_array::NestedKind;
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{left_chain, value_operands, ArrayNode, Node},
        token::{Error, OpProduct},
        types::CellReferenceIndex,
    },
    model::Model,
    reference_operators::RefArg,
};

/// One SUMPRODUCT argument's elements as numbers.
enum Factors {
    /// Row by row.
    Grid(Vec<Vec<f64>>),
    /// One column: the head's rows, then `tail_len` rows of `tail`.
    Tailed {
        head: Vec<f64>,
        tail: f64,
        tail_len: usize,
    },
    /// A `block*mask*...` argument, unbuilt.
    Masked(MaskedBlock),
}

/// A SUMPRODUCT argument `block*mask*...` read without building its arrays (T-3230): a block of
/// cells times masks, each a comparison's 1-row array as wide as the block or 1-column array as
/// tall, whose elements are booleans or errors. A mask element is 1 or 0, so each element of the
/// product is its cell's number, or a zero where a mask is 0, exactly as `*` gives it (the zero's
/// sign never shows: adding it leaves SUMPRODUCT's total as it is, and the total is never -0).
/// Its first error element, row by row, is its cell's error, or else the first in operator order
/// of the masks' errors at its row and column, as `*` picks the left operand's error first.
///
/// An operand may also be a weight ([`Weight`], T-3725): a cached block as tall as the block or
/// one row, as wide or one column, but not one cell (calipers `pv26_002`'s
/// `block*header_mask*amount_column`). Where the masks are TRUE the element is the cell's number
/// times each weight's at its row and column, rounded and flushed as `*` rounds and flushes each
/// product (`times`: `extended_product`, then a subnormal is 0); a mask that is FALSE makes a
/// zero, before or after a weight, since no product overflows (the bound, below). A weight's
/// errors stand at its positions like a mask's.
struct MaskedBlock {
    numbers: Arc<BlockNumbers>,
    rows: usize,
    columns: usize,
    /// Whether every row mask is TRUE at each row, every column mask at each column.
    row_on: Vec<bool>,
    column_on: Vec<bool>,
    /// The weights, in operator order.
    weights: Vec<Weight>,
    /// The product of the block's and each weight's largest magnitude: below
    /// [`MaskedBlock::BOUND`], no element's product overflows into `*`'s `#NUM!`.
    bound: f64,
    /// The first error element, row by row: its position and error.
    error: Option<(usize, Error)>,
}

/// A weight of a [`MaskedBlock`]: a cached block of `rows` x `columns`, each 1 or the block's.
struct Weight {
    numbers: Arc<BlockNumbers>,
    rows: usize,
    columns: usize,
}

impl Weight {
    /// Its number at the block's row `i`, column `j`: a one-row or one-column weight repeats.
    fn get(&self, i: usize, j: usize) -> f64 {
        let i = if self.rows == 1 { 0 } else { i };
        let j = if self.columns == 1 { 0 } else { j };
        let index = i.saturating_mul(self.columns).saturating_add(j);
        self.numbers.values.get(index).copied().unwrap_or(0.0)
    }
}

impl MaskedBlock {
    fn new(numbers: Arc<BlockNumbers>, rows: usize, columns: usize) -> MaskedBlock {
        MaskedBlock {
            error: numbers.first_error.clone(),
            bound: numbers.largest,
            numbers,
            rows,
            columns,
            row_on: vec![true; rows],
            column_on: vec![true; columns],
            weights: Vec::new(),
        }
    }

    /// Products of magnitudes below this are finite, with room for `*`'s rounding.
    const BOUND: f64 = f64::MAX / 2.0;

    /// Multiplies the block by a cached block `rows` x `columns` when it is a weight (type docs);
    /// false, leaving the block as it was, when it is not one or its products could overflow.
    fn weigh(&mut self, numbers: Arc<BlockNumbers>, rows: usize, columns: usize) -> bool {
        let shaped = (rows == self.rows || rows == 1) && (columns == self.columns || columns == 1);
        let bound = self.bound * numbers.largest;
        // `bound` is never NaN: finite magnitudes, stored only below the bound.
        if !shaped || rows * columns == 1 || bound >= Self::BOUND {
            return false;
        }
        // Its error at index k (row by row) is first met at its row and column of the block; the
        // map keeps the order, so its first error is first there too.
        if let Some((index, error)) = &numbers.first_error {
            let (i, j) = (index / columns, index % columns);
            let position = i.saturating_mul(self.columns).saturating_add(j);
            if self
                .error
                .as_ref()
                .is_none_or(|(first, _)| position < *first)
            {
                self.error = Some((position, error.clone()));
            }
        }
        self.bound = bound;
        self.weights.push(Weight {
            numbers,
            rows,
            columns,
        });
        true
    }

    /// Multiplies the block by `value` when it is a mask (type docs); false, leaving the block as
    /// it was, when it is anything else.
    fn multiply(&mut self, value: &CalcResult) -> bool {
        let CalcResult::Array(array) = value else {
            return false;
        };
        let column_mask = array.len() == 1 && array.iter().all(|row| row.len() == self.columns);
        let row_mask = array.len() == self.rows && array.iter().all(|row| row.len() == 1);
        let elements = || array.iter().flatten();
        let masks =
            elements().all(|node| matches!(node, ArrayNode::Boolean(_) | ArrayNode::Error(_)));
        if !masks || !(column_mask || row_mask) {
            return false;
        }
        // A mask's error at index k is first met at position k of its row (column mask) or at
        // the start of row k (row mask); where an earlier operand has an error at the same
        // position, that one stays.
        let stride = if column_mask { 1 } else { self.columns };
        let first_error = elements().enumerate().find_map(|(index, node)| match node {
            ArrayNode::Error(error) => Some((index.saturating_mul(stride), error.clone())),
            _ => None,
        });
        if let Some((position, error)) = first_error {
            if self
                .error
                .as_ref()
                .is_none_or(|(first, _)| position < *first)
            {
                self.error = Some((position, error));
            }
        }
        let on = if column_mask {
            &mut self.column_on
        } else {
            &mut self.row_on
        };
        for (on, node) in on.iter_mut().zip(elements()) {
            *on = *on && matches!(node, ArrayNode::Boolean(true));
        }
        true
    }

    /// The element at row `i`, column `j`, within the shape.
    fn get(&self, i: usize, j: usize) -> f64 {
        let on = self.row_on.get(i).copied().unwrap_or(false)
            && self.column_on.get(j).copied().unwrap_or(false);
        if on {
            let index = i.saturating_mul(self.columns).saturating_add(j);
            // At least one `*` multiplies every cell, and `*` flushes each product below the
            // smallest normal double to 0 (`times`; numbers.md "Subnormal results become 0"): a
            // kept subnormal times TRUE is 0, and so is a product that underflows before a
            // weight would grow it back.
            let number = self.numbers.values.get(index).copied().unwrap_or(0.0);
            self.weights
                .iter()
                .fold(flush_subnormal(number), |product, weight| {
                    flush_subnormal(extended_product(product, weight.get(i, j)))
                })
        } else {
            0.0
        }
    }
}

/// A SUMPRODUCT argument read by [`Model::read_masked_block`].
enum MaskedRead {
    Block(MaskedBlock),
    /// Its value, as the operators gave it.
    Value(CalcResult),
}

impl Factors {
    /// A bound on the magnitude of its elements: the largest one, or a masked block's `bound`.
    fn largest(&self) -> f64 {
        let largest = |values: &mut dyn Iterator<Item = f64>| {
            values.fold(0.0f64, |largest, value| largest.max(value.abs()))
        };
        match self {
            Factors::Grid(grid) => largest(&mut grid.iter().flatten().copied()),
            Factors::Tailed { head, tail, .. } => {
                largest(&mut head.iter().copied()).max(tail.abs())
            }
            Factors::Masked(block) => block.bound,
        }
    }

    /// (rows, columns).
    fn shape(&self) -> (usize, usize) {
        match self {
            Factors::Grid(grid) => (grid.len(), grid.first().map_or(0, Vec::len)),
            Factors::Tailed { head, tail_len, .. } => (head.len() + tail_len, 1),
            Factors::Masked(block) => (block.rows, block.columns),
        }
    }

    /// The element at row `i`, column `j`, within the shape.
    fn get(&self, i: usize, j: usize) -> f64 {
        match self {
            Factors::Grid(grid) => grid
                .get(i)
                .and_then(|row| row.get(j))
                .copied()
                .unwrap_or(0.0),
            Factors::Tailed { head, tail, .. } => head.get(i).copied().unwrap_or(*tail),
            Factors::Masked(block) => block.get(i, j),
        }
    }
}

/// One element as a SUMPRODUCT factor: non-numbers are 0, an error is 0 and goes to
/// `element_error` unless one is there already; a lambda or a nested reference is `#VALUE!`.
fn sumproduct_factor(
    node: &ArrayNode,
    cell: CellReferenceIndex,
    element_error: &mut Option<Error>,
) -> Result<f64, CalcResult> {
    Ok(match node {
        ArrayNode::Number(v) => *v,
        ArrayNode::Boolean(_) | ArrayNode::String(_) | ArrayNode::Empty => 0.0,
        // A nested GROUPBY or PIVOTBY cell is no number: `SUMPRODUCT(GROUPBY(..))`
        // is 0 (probes/t2594-nested-aggregates A22, T-2625). A reference cell is
        // unprobed and stays `#VALUE!`.
        ArrayNode::Nested(nested) if nested.kind() == NestedKind::Array => 0.0,
        ArrayNode::Lambda(_) | ArrayNode::Nested(_) => {
            return Err(Model::lambda_as_number_error(cell))
        }
        ArrayNode::Error(e) => {
            element_error.get_or_insert_with(|| e.clone());
            0.0
        }
    })
}

impl Model {
    // ── SUMPRODUCT ────────────────────────────────────────────────────────────

    /// `=SUMPRODUCT(array1, [array2], ...)`
    ///
    /// Multiplies corresponding elements across all arrays and returns the sum.
    /// Non-numeric values (text, booleans, empty) are treated as 0, but a blank one-cell
    /// reference is `#VALUE!`. All arrays must have the same dimensions; a mismatch yields
    /// `#VALUE!` unless an argument holds an error element or a later one is itself an error or blank.
    pub(crate) fn fn_sumproduct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        // Read every argument in order: a whole-argument error or a blank one-cell argument
        // wins even over a shape mismatch between earlier arguments
        // (`SUMPRODUCT(NA(),D1:E2,blank)` is `#N/A`, `SUMPRODUCT(blank,D1:E2,NA())` `#VALUE!`;
        // probes/t2200-blank-and-padded-m5-args A22, A23; coercion.md, T-2150). An error element
        // of any argument beats a shape mismatch in either order, but not a whole-argument
        // error of a later argument (`SUMPRODUCT(R,X,NA())` is `#N/A`); the first element error
        // in argument then element order is kept (probes/t2296-sumproduct-error-element, T-2450). A blank
        // one-cell argument after an element error also wins, inferred and unprobed.
        let mut factors: Vec<Factors> = Vec::with_capacity(args.len());
        let mut shape: Option<(usize, usize)> = None;
        let mut mismatch = false;
        let mut element_error: Option<Error> = None;
        for arg in args {
            let arg_factors = match self.sumproduct_factors(arg, cell, &mut element_error) {
                Ok(factors) => factors,
                Err(e) => return e,
            };
            let arg_shape = arg_factors.shape();
            mismatch = mismatch || *shape.get_or_insert(arg_shape) != arg_shape;
            factors.push(arg_factors);
        }

        if let Some(error) = element_error {
            return CalcResult::new_error(error, cell, String::new());
        }
        if mismatch {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "SUMPRODUCT: arrays must have the same dimensions".to_string(),
            );
        }
        let (rows, cols) = shape.unwrap_or((0, 0));
        // Each multiplication rounds twice, as `*` does (probes-numeric/
        // double-rounding-sumproduct-multi A1-A14, T-1697).
        let product = |i: usize, j: usize| {
            factors.iter().fold(1.0, |acc, factor| {
                flush_subnormal(extended_product(acc, factor.get(i, j)))
            })
        };
        // Past every tailed argument's head, when every argument is tailed, each row has the
        // same product (SPEC-0020, T-0471).
        let heads = factors
            .iter()
            .map(|factor| match factor {
                Factors::Tailed { head, .. } => Some(head.len()),
                Factors::Grid(_) | Factors::Masked(_) => None,
            })
            .try_fold(0, |longest, head| head.map(|head| longest.max(head)));
        let walked = heads.unwrap_or(rows).min(rows);
        // A masked argument's element is a zero where one of its masks is FALSE, and so is the
        // product there, which leaves the total as it is (below), unless the factors before it
        // overflow: inf times the zero is NaN, `#NUM!` (numbers.md "Overflow in `+ - * /` is
        // #NUM! where it happens"). So when the product of every factor's largest magnitude is
        // below `MaskedBlock::BOUND`, no product overflows, and only the rows and columns that
        // every masked argument has on are walked, in the same order (T-3732). Calipers
        // `pv26_002`'s 1,575 masked calls each keep one column of 41 and a few rows of 1,440.
        let bound = factors
            .iter()
            .fold(1.0f64, |bound, factor| bound * factor.largest());
        let masked: Vec<&MaskedBlock> = if bound < MaskedBlock::BOUND {
            factors
                .iter()
                .filter_map(|factor| match factor {
                    Factors::Masked(block) => Some(block),
                    Factors::Grid(_) | Factors::Tailed { .. } => None,
                })
                .collect()
        } else {
            Vec::new()
        };
        let on = |flags: fn(&MaskedBlock) -> &[bool], index: usize| {
            masked
                .iter()
                .all(|block| flags(block).get(index).copied().unwrap_or(false))
        };
        let columns: Vec<usize> = (0..cols)
            .filter(|&j| on(|block| &block.column_on, j))
            .collect();
        let mut total = 0.0f64;
        for i in (0..walked).filter(|&i| on(|block| &block.row_on, i)) {
            for &j in &columns {
                // Excel rounds each addition of the running total twice, as SUM does (numbers.md
                // "double rounding", probes-numeric/double-rounding-avedev-deviations, T-1665).
                total = extended_sum(total, product(i, j));
            }
        }
        if walked < rows {
            // A zero product leaves the total as it is (`extended_sum` adds a zero plainly, and
            // the total is never -0); any other one is added once per row, as the walk does.
            let tail = product(walked, 0);
            if tail != 0.0 {
                for _ in walked..rows {
                    total = extended_sum(total, tail);
                }
            }
        }

        CalcResult::finite_number_or_num(flush_subnormal(total), cell)
    }

    /// One SUMPRODUCT argument's factors, its elements read as numbers: an element-wise
    /// expression of whole columns as a head and a tail (SPEC-0020, T-0471), anything else as a
    /// grid. The first element error in element order goes to `element_error`.
    fn sumproduct_factors(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
        element_error: &mut Option<Error>,
    ) -> Result<Factors, CalcResult> {
        let tailed = self
            .read_tailed(arg, cell, |model| model.evaluate_value_or_grid(arg, cell))
            .and_then(crate::tailed_array::Tailed::elements);
        if let Some((head, tail)) = tailed {
            let head = head
                .map(|node| sumproduct_factor(&node, cell, element_error))
                .collect::<Result<Vec<f64>, CalcResult>>()?;
            let (tail, tail_len) = match tail {
                Some((node, rows)) => (sumproduct_factor(&node, cell, element_error)?, rows),
                // The pass is being abandoned: the value is discarded.
                None => (0.0, 0),
            };
            return Ok(Factors::Tailed {
                head,
                tail,
                tail_len,
            });
        }
        let array = match self.read_masked_block(arg, cell) {
            Some(MaskedRead::Block(block)) => {
                if let Some((_, error)) = &block.error {
                    element_error.get_or_insert_with(|| error.clone());
                }
                return Ok(Factors::Masked(block));
            }
            Some(MaskedRead::Value(value)) => self.sumproduct_read_array(RefArg::from(value), cell),
            None => self.sumproduct_array(arg, cell),
        }?;
        let grid = array
            .iter()
            .map(|row| {
                row.iter()
                    .map(|node| sumproduct_factor(node, cell, element_error))
                    .collect()
            })
            .collect::<Result<Vec<Vec<f64>>, CalcResult>>()?;
        Ok(Factors::Grid(grid))
    }

    /// One SUMPRODUCT argument as an array; a blank one-cell reference is `#VALUE!`. An array of
    /// references reads each reference element as a non-number (0) and an error element as its
    /// error (SPEC-0014 rule 5:
    /// `SUMPRODUCT(OFFSET(A1,{0;1},0))` is 0, `SUMPRODUCT(OFFSET(A1,{0;-1},0))` `#REF!`;
    /// probes/reference-lift-consumers E29, -b2 E189).
    fn sumproduct_array(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        let read = self.evaluate_value_or_grid(arg, cell);
        self.sumproduct_read_array(read, cell)
    }

    /// [`Model::sumproduct_array`] of an argument already read.
    fn sumproduct_read_array(
        &mut self,
        read: RefArg,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        match read {
            RefArg::Grid(grid) => Ok(grid.to_array(ArrayNode::Empty)),
            read => {
                // A blank cell given as a cell or one-cell range reference is `#VALUE!`, alone or
                // beside other arguments, while a one-cell text or boolean is a 0 factor
                // (probes/t2200-blank-and-padded-m5-args A1-A19; coercion.md, T-2150).
                let result = self.collapse_single_cell(read.into_single_area(cell));
                if matches!(result, CalcResult::EmptyCell) {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "SUMPRODUCT: a one-cell argument is blank".to_string(),
                    ));
                }
                self.result_to_array(result, cell)
            }
        }
    }

    /// A SUMPRODUCT argument `block*mask*...` (a range times operands, [`MaskedBlock`]) read
    /// without building its arrays when the block's cells are cached (`ifs_cache.rs`) and every
    /// operand after it is a mask (T-3230): calipers `pv26_002` multiplies 1,440 x 41 cells by a
    /// header-row and a key-column comparison in each of 1,500 calls. The argument is evaluated
    /// as `evaluate_node_in_context` evaluates it, operand by operand in the same order; only the
    /// block's reads move, and reading a cached block evaluates nothing. At the first operand that
    /// is no mask, the operators run over the operands read so far, and the rest of the chain as
    /// usual: [`MaskedRead::Value`]. `None`, having read nothing, when `arg` is no such chain.
    fn read_masked_block(&mut self, arg: &Node, cell: CellReferenceIndex) -> Option<MaskedRead> {
        let node = arg.ungrouped();
        let times = |link: &Node| {
            matches!(
                link,
                Node::OpProductKind {
                    kind: OpProduct::Times,
                    ..
                }
            )
        };
        if !times(node) || !self.tail_views.is_empty() || !self.pass_cache_safe() {
            return None;
        }
        let (links, first) = left_chain(node);
        let links: Vec<&Node> = links.collect();
        if !links.iter().all(|link| times(link))
            || !matches!(first.ungrouped(), Node::RangeKind { .. })
        {
            return None;
        }
        if let Err(error) = self.spend_node(cell) {
            return Some(MaskedRead::Value(error));
        }
        let mut value = self.evaluate_node_in_context(first, cell);
        let mut block = match value {
            CalcResult::Range { left, right } => {
                let rows = usize::try_from(right.row - left.row + 1).unwrap_or(0);
                let columns = usize::try_from(right.column - left.column + 1).unwrap_or(0);
                // A block the operators could not lift is theirs to refuse (`#ERROR!`, arrays.md
                // "Array results above 1,048,576 cells").
                if check_array_size(rows, columns).is_some() {
                    None
                } else {
                    self.block_numbers(left, right)
                        .map(|numbers| MaskedBlock::new(numbers, rows, columns))
                }
            }
            _ => None,
        };
        let mut operands: Vec<CalcResult> = Vec::new();
        for (index, &link) in links.iter().enumerate() {
            let Some(masked) = block.as_mut() else {
                value = self.apply_operator_link(value, link, cell);
                if let Err(error) = self.charge_chain_link(node, link, &value, cell) {
                    return Some(MaskedRead::Value(error));
                }
                continue;
            };
            // Every link is a `*` (checked above), so it has a right operand.
            let (_, right) = value_operands(link)?;
            let right = self.evaluate_node_in_context(right, cell);
            let is_mask = masked.multiply(&right) || self.weigh_masked(masked, &right);
            operands.push(right);
            if is_mask {
                continue;
            }
            block = None;
            for (&link, right) in links.iter().take(index + 1).zip(operands.drain(..)) {
                value = self.collapse_single_cell(value);
                value = self.apply_operator(value, link, right, cell);
                if let Err(error) = self.charge_chain_link(node, link, &value, cell) {
                    return Some(MaskedRead::Value(error));
                }
            }
        }
        Some(match block {
            Some(block) => {
                #[cfg(test)]
                {
                    self.evaluation.ifs_cache.masked += 1;
                }
                MaskedRead::Block(block)
            }
            None => MaskedRead::Value(self.charged(value, cell)),
        })
    }

    /// Multiplies `block` by `operand` when it is a range whose cells are cached and that is a
    /// weight of it ([`MaskedBlock::weigh`]); reading a cached range evaluates nothing.
    fn weigh_masked(&mut self, block: &mut MaskedBlock, operand: &CalcResult) -> bool {
        let CalcResult::Range { left, right } = operand else {
            return false;
        };
        let rows = usize::try_from(right.row - left.row + 1).unwrap_or(0);
        let columns = usize::try_from(right.column - left.column + 1).unwrap_or(0);
        if rows > block.rows || columns > block.columns {
            return false;
        }
        self.block_numbers(*left, *right)
            .is_some_and(|numbers| block.weigh(numbers, rows, columns))
    }
}
