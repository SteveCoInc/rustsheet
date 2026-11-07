use super::chisq::RankData;
use super::sorted_data::{DataOrder, SortedNumbers};
use crate::calc_result::Range;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{value_error, LiftCost};
use crate::functions::Function;
use crate::reference_operators::{RefElement, RefGrid};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// How tied numbers rank: RANK and RANK.EQ give them the top rank, RANK.AVG their average.
#[derive(Clone, Copy)]
enum Ties {
    Top,
    Average,
}

impl Model {
    // RANK(number, ref, [order]) and RANK.EQ
    pub(crate) fn fn_rank_eq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.rank(args, Ties::Top, cell)
    }

    // RANK.AVG(number, ref, [order])
    pub(crate) fn fn_rank_avg(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.rank(args, Ties::Average, cell)
    }

    /// The RANK family: `number` and `order` lift over an array and a multi-cell range, and a
    /// reference `ref` is read once (SPEC-0011, T-1999). An element reads `number` (numeric text
    /// and booleans coerce, also from a cell), then `order`, then takes the data's error:
    /// `RANK("x",D1:D3)` and `RANK(4,D1:D3,"x")` with D2 = `#N/A` are `#VALUE!`,
    /// `RANK({1;3},D1:D3)` is `#N/A` per element (probes/stat-test-lift P20-P27, P32, P33, P85;
    /// probes/rank-direct-boolean). A `ref` that is not a reference is an array of `#VALUE!` (or
    /// the element's own error) that broadcasts with the lifted arguments, after `number`'s error
    /// and before `order`'s (statistics.md "RANK of a non-reference array"; P28-P31,
    /// probes/rank-nonref-errors). The numbers are sorted once for a lifted call, so it is charged
    /// their count plus its elements (`LiftCost::Prepared`).
    fn rank(&mut self, args: &[Node], ties: Ties, cell: CellReferenceIndex) -> CalcResult {
        // RANK and RANK.AVG share RANK.EQ's table entry (`ONE_THEN_REFERENCE_THEN_1`).
        let kind = Function::RankEq;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        let Some(reference) = args.get(1) else {
            return CalcResult::new_args_number_error(cell);
        };
        let mut numbers = match self.numbers_from_rank_ref_arg(reference, cell) {
            // Excel: a reference with no numeric value (or an empty one) gives #N/A like any
            // absent number, never #NUM! (statistics.md, "RANK/RANK.EQ/RANK.AVG: a number absent").
            Ok(RankData::Numbers(values)) => Ok(sorted_numbers(values)),
            Ok(RankData::PerElement(grid)) => {
                return self.rank_per_element(args, &grid, ties, cell);
            }
            Err(CalcResult::Array(grid)) => {
                // Every element of `grid` is an error, so no element reads the numbers.
                let mut numbers = Err(value_error(cell));
                return self.lift_call_beside(
                    kind,
                    args,
                    (1, grid),
                    LiftCost::Unit,
                    cell,
                    &mut |model, values| {
                        rank_element(model, &mut numbers, values, ties, true, cell)
                    },
                );
            }
            Err(error) => Err(error.without_message()),
        };
        let cells = numbers.as_ref().map_or(0, |numbers| {
            u64::try_from(numbers.len()).unwrap_or(u64::MAX)
        });
        // The first element walks the numbers, so a scalar call (`=RANK(A2,$A$2:$A$9999)` filled
        // down) never sorts; later elements sort them once and search (`RANK(A2:A9999,A2:A9999)`).
        let mut first = true;
        self.lift_call_costed(
            kind,
            args,
            // At least one unit an element, so a data error copied per element is charged too.
            LiftCost::Prepared(cells.max(1)),
            cell,
            &mut |model, values| {
                let walk = std::mem::replace(&mut first, false);
                rank_element(model, &mut numbers, values, ties, walk, cell)
            },
        )
    }
}

impl Model {
    /// The RANK family over an array of references lifts over its elements, broadcast with a
    /// lifted `number` or `order`: a reference element ranks against its own range, a value
    /// element is `#VALUE!` (a value is no reference) and an error element is itself, each after
    /// `number`'s error. `RANK(60,X)` over `X = XLOOKUP({2;9},A1:A3,B1:B3,Data!G1:G2)` (hits 20)
    /// is `{#VALUE!;1}`, `RANK(20,X)` `{#VALUE!;#N/A}` (probes/t4704-mixed-xlookup-grid-readers
    /// A25:C38, T-4740). The elements reach the driver as their indices, as OFFSET's do
    /// (`offset_over_grid`); each element reads its range, charged as the largest one.
    fn rank_per_element(
        &mut self,
        args: &[Node],
        grid: &RefGrid,
        ties: Ties,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let cells = grid
            .elements()
            .iter()
            .map(|element| match element {
                RefElement::Reference(range) => range_cells(range),
                RefElement::Error(_) | RefElement::Value(_) => 1,
            })
            .max()
            .unwrap_or(1);
        self.lift_call_beside(
            Function::RankEq,
            args,
            (1, grid.index_array()),
            LiftCost::Cells(cells),
            cell,
            &mut |model, values| {
                let element = values.get(1).and_then(|index| grid.element_at(index));
                let (mut numbers, slot) = match element {
                    Some(RefElement::Reference(range)) => (
                        model
                            .values_from_range(range.left, range.right)
                            .map(sorted_numbers)
                            .map_err(CalcResult::without_message),
                        CalcResult::EmptyArg,
                    ),
                    Some(RefElement::Error(error)) => (
                        Err(value_error(cell)),
                        CalcResult::new_error(error.clone(), cell, String::new()),
                    ),
                    Some(RefElement::Value(_)) | None => {
                        (Err(value_error(cell)), value_error(cell))
                    }
                };
                let mut values = values.to_vec();
                if let Some(ref_slot) = values.get_mut(1) {
                    *ref_slot = slot;
                }
                rank_element(model, &mut numbers, &values, ties, true, cell)
            },
        )
    }
}

/// The numbers RANK ranks against: the numeric values of its data, NaN dropped.
fn sorted_numbers(values: Vec<Option<f64>>) -> SortedNumbers {
    SortedNumbers::new(
        values
            .into_iter()
            .flatten()
            .filter(|v| !v.is_nan())
            .collect(),
        DataOrder::Ascending,
    )
}

/// The cells of `range`, saturating.
fn range_cells(range: &Range) -> u64 {
    let rows = u64::try_from(range.right.row - range.left.row).unwrap_or(0) + 1;
    let columns = u64::try_from(range.right.column - range.left.column).unwrap_or(0) + 1;
    rows.saturating_mul(columns)
}

/// One RANK element over the numbers read once. `values[1]` is the slot of `ref`: `EmptyArg`
/// for a reference, or the error of a non-reference array's element. `walk` counts the numbers
/// unsorted; otherwise they are sorted (once) and searched.
fn rank_element(
    model: &Model,
    numbers: &mut Result<SortedNumbers, CalcResult>,
    values: &[CalcResult],
    ties: Ties,
    walk: bool,
    cell: CellReferenceIndex,
) -> CalcResult {
    let number = match values
        .first()
        .map(|number| model.kernel_number(number, cell))
    {
        Some(Ok(number)) => number,
        Some(Err(error)) => return error,
        None => return CalcResult::new_args_number_error(cell),
    };
    if let Some(error @ CalcResult::Error { .. }) = values.get(1) {
        return error.clone();
    }
    // order: default 0 (descending). Text is `#VALUE!`, numeric text too, where `number` reads
    // it; a boolean is 0 or 1 (probes/rank-direct-boolean H4, H10, H15; stat-test-lift P33).
    let order = match values.get(2) {
        Some(CalcResult::String(_)) => return value_error(cell),
        order => match model.kernel_number_or(order, 0.0, cell) {
            Ok(order) => order,
            Err(error) => return error,
        },
    };
    let numbers = match numbers {
        Ok(numbers) => numbers,
        Err(error) => return error.clone(),
    };
    let (smaller, equal, larger) = if walk {
        let (mut smaller, mut equal, mut larger) = (0, 0, 0);
        for &v in numbers.as_read() {
            if v < number {
                smaller += 1;
            } else if v == number {
                equal += 1;
            } else {
                larger += 1;
            }
        }
        (smaller, equal, larger)
    } else {
        // Ascending, and `<`/`<=` treat -0 and 0 as equal, as `==` does.
        let sorted = numbers.sorted();
        let smaller = sorted.partition_point(|&v| v < number);
        let not_larger = sorted.partition_point(|&v| v <= number);
        (smaller, not_larger - smaller, sorted.len() - not_larger)
    };
    if equal == 0 {
        return CalcResult::new_error(Error::NA, cell, String::new());
    }
    // "better" = greater (descending, order 0) or smaller (ascending)
    let better = if order == 0.0 { larger } else { smaller } as f64;
    let rank = match ties {
        Ties::Top => better + 1.0,
        // The tied numbers hold ranks better+1 ..= better+equal; their average.
        Ties::Average => better + (equal as f64 + 1.0) / 2.0,
    };
    CalcResult::Number(rank)
}
