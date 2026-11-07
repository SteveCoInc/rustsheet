#![allow(clippy::unwrap_used, reason = "tests")]
//! The pass's own rules on hand-made values; the probes' cells are compared in
//! `layout::render::tests` (`the_show_values_as_probes_cells_are_excels`).

use super::*;

#[test]
fn ranks_are_dense_and_leave_empty_cells_out() {
    let values = [
        Some(Ok(5.0)),
        None,
        Some(Ok(5.0)),
        Some(Ok(3.0)),
        Some(Ok(-0.0)),
    ];
    let ascending = ranks(&values, false);
    assert_eq!(
        ascending,
        [
            Some(Ok(3.0)),
            None,
            Some(Ok(3.0)),
            Some(Ok(2.0)),
            Some(Ok(1.0))
        ]
    );
    let descending = ranks(&values, true);
    assert_eq!(
        descending,
        [
            Some(Ok(1.0)),
            None,
            Some(Ok(1.0)),
            Some(Ok(2.0)),
            Some(Ok(3.0))
        ]
    );
}

#[test]
fn running_totals_add_empty_cells_as_zero_and_keep_an_error() {
    let values = [
        None,
        Some(Ok(2.0)),
        None,
        Some(Err(Error::VALUE)),
        Some(Ok(1.0)),
    ];
    assert_eq!(
        running(&values),
        [
            Ok(0.0),
            Ok(2.0),
            Ok(2.0),
            Err(Error::VALUE),
            Err(Error::VALUE)
        ]
    );
    // Over a total of 0, a cell whose own value is 0 or empty is empty (t4161 RZ PRY).
    let zero = [Some(Ok(-4.0)), Some(Ok(0.0)), None, Some(Ok(4.0))];
    assert_eq!(
        of_running_total(&zero),
        [Some(Err(Error::DIV)), None, None, Some(Err(Error::DIV))]
    );
}

/// The diagonal pivot of `n` items with its data field shown as `mode` against `base_field`.
fn diagonal_shown(
    n: u32,
    mode: ShowDataAs,
    base_field: i32,
) -> (super::super::PivotCache, PivotTable) {
    let (cache, mut table) = crate::pivot::probe_cases::diagonal(n, true);
    let data = table.data_fields.first_mut().unwrap();
    data.show_data_as = mode;
    data.base_field = base_field;
    (cache, table)
}

#[test]
fn a_base_field_on_no_axis_is_na_everywhere() {
    // Field 2 (V) is the data field's source, on no axis (UNVERIFIED, T-4750).
    let (cache, table) = diagonal_shown(3, ShowDataAs::Difference, 2);
    let mut work = super::super::MAX_PIVOT_WORK;
    let result = super::super::compute(&table, &cache, &mut work).unwrap();
    for row in 0..4 {
        for column in 0..4 {
            assert_eq!(result.value(row, column, 0), Some(Err(Error::NA)));
        }
    }
    // On the row axis, the root and the first item are their own base (previous).
    let (cache, mut table) = diagonal_shown(3, ShowDataAs::Difference, 0);
    table.data_fields[0].base_item = PREVIOUS;
    let mut work = super::super::MAX_PIVOT_WORK;
    let result = super::super::compute(&table, &cache, &mut work).unwrap();
    assert_eq!(result.value(1, 0, 0), None);
    // Row item 1 (value 1) against item 0 (value 0), in the grand total column.
    assert_eq!(result.value(2, 0, 0), Some(Ok(1.0)));
}

#[test]
fn the_pass_is_charged_per_cell() {
    // A diagonal pivot has n + 1 nodes per axis: the pass charges (n + 1)² units, so a pivot
    // whose plain values fit the budget fails when its values are shown as a calculation.
    let n = 1_000;
    let (cache, plain) = crate::pivot::probe_cases::diagonal(n, true);
    let mut work = super::super::MAX_PIVOT_WORK;
    super::super::compute(&plain, &cache, &mut work).unwrap();
    let spent = super::super::MAX_PIVOT_WORK - work;
    let (cache, shown) = diagonal_shown(n, ShowDataAs::PercentOfTotal, -1);
    let mut work = spent + u64::from(n) * u64::from(n);
    assert_eq!(
        super::super::compute(&shown, &cache, &mut work),
        Err(PivotComputeError::Limit)
    );
    assert_eq!(work, 0);
    let mut work = spent + u64::from(n + 1) * u64::from(n + 1);
    let result = super::super::compute(&shown, &cache, &mut work).unwrap();
    assert_eq!(result.value(0, 0, 0), Some(Ok(1.0)));
}
