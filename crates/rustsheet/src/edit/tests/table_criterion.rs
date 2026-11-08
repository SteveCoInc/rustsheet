use super::*;

/// T-4973: the criterion setters name the same table errors and take one or two custom conditions.
#[test]
fn table_criterion_setters_refuse_a_missing_table_column_or_condition_count() {
    let mut wb = keyed_book();
    let range = wb.range_ref("A1:B4").unwrap();
    wb.edit(|tx| tx.add_table(range, "Data", true)).unwrap();
    let cond = |v: &str| (None, v.to_string());
    for conditions in [vec![], vec![cond("a"), cond("b"), cond("c")]] {
        let err = wb
            .edit(|tx| tx.filter_table_column_custom("Data", 0, false, &conditions))
            .unwrap_err();
        assert!(
            matches!(&err, EditError::Table(m) if m.contains("one or two")),
            "{err:?}"
        );
    }
    let err = wb
        .edit(|tx| tx.filter_table_column_values("Nope", 0, &["a".to_string()]))
        .unwrap_err();
    assert!(
        matches!(&err, EditError::Table(m) if m == "No table 'Nope'"),
        "{err:?}"
    );
    let err = wb
        .edit(|tx| tx.filter_table_column_custom("Data", 5, false, &[cond("a")]))
        .unwrap_err();
    assert!(
        matches!(&err, EditError::Table(m) if m.contains("outside")),
        "{err:?}"
    );
    wb.edit(|tx| tx.filter_table_column_custom("Data", 0, true, &[cond("a"), cond("b")]))
        .unwrap();
}
