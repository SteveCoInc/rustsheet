//! Slicers and timelines as pivot filters (T-4188): what the importer reads from a workbook's
//! slicer and timeline caches, held on the [`PivotField`](super::PivotField) they filter.
//!
//! Excel for the web applies both on open, and both win over the pivot's own filter of the
//! field (docs/excel-semantics/pivots.md "Slicers and timelines filter the pivot they drive",
//! probes/t4767-pivot-slicers and probes/t4767-pivot-timelines):
//! - a slicer's selection is the field's item visibility, in place of its hidden items
//!   (`item@h`), so an item the slicer selects shows even when the pivot hides it (SX);
//! - a timeline's selection is a span of days the field's records must fall in, in place of the
//!   pivot's own `<filters>` on the field (TX).
//!
//! `result::filter` applies both before anything is accumulated, as it applies hidden items.

use bitcode::{Decode, Encode};

use super::CacheItem;

/// A timeline's selection: the days it keeps, first and last included, as whole day serials.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub struct DateSpan {
    /// The first day kept.
    pub first: i64,
    /// The last day kept.
    pub last: i64,
}

impl DateSpan {
    /// The span from the day of serial `start` to the day of serial `end`; `None` when either is
    /// no finite serial a workbook holds, or `end` is before `start` (the importer then applies
    /// no selection).
    #[must_use]
    pub fn from_serials(start: f64, end: f64) -> Option<Self> {
        let day = |serial: f64| {
            // Excel's last day, 9999-12-31, is serial 2958465; nothing past it is a date.
            // Bounded, so the cast is exact.
            (serial.is_finite() && (0.0..2_958_466.0).contains(&serial))
                .then(|| serial.floor() as i64)
        };
        let (first, last) = (day(start)?, day(end)?);
        (first <= last).then_some(DateSpan { first, last })
    }

    /// Whether the span keeps a record holding `item`: a date or number whose day is in it.
    /// Any other item (a text, a blank) is outside every selection (UNVERIFIED, T-4962: the
    /// probes' timeline field holds dates only).
    #[must_use]
    pub fn keeps(&self, item: &CacheItem) -> bool {
        match item {
            CacheItem::Date(serial) | CacheItem::Number(serial) => {
                DateSpan::from_serials(*serial, *serial)
                    .is_some_and(|day| (self.first..=self.last).contains(&day.first))
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;
    use crate::pivot::filter::{FilterCondition, FilterCriteria, FilterOperator, PivotFilter};
    use crate::pivot::probe_cases::{item_of, probe_pivot, read, sheet, sidecar_differences};
    use crate::pivot::PivotFilterKind;

    /// Sheet `name` of probes/t4767-pivot-slicers with its slicer applied as the importer
    /// applies it: the field's items the slicer leaves unselected (`selected`, or every item).
    fn slicer_differences(name: &str) -> Vec<String> {
        let stem = "t4767-pivot-slicers";
        let case = read(stem, "case");
        let (cache, mut table) = probe_pivot(&case, name);
        let slicer = &sheet(&case, name)["slicers"][0];
        let field = (cache.fields.iter())
            .position(|f| slicer["field"] == f.name.as_str())
            .unwrap();
        let count = cache.fields[field].items.len();
        let mut bits = vec![0_u64; count.div_ceil(64)];
        if let Some(selected) = slicer["selected"].as_array() {
            let field = u32::try_from(field).unwrap();
            let chosen: Vec<u32> = selected.iter().map(|v| item_of(&cache, field, v)).collect();
            for item in (0..u32::try_from(count).unwrap()).filter(|i| !chosen.contains(i)) {
                bits[item as usize / 64] |= 1 << (item % 64);
            }
        }
        table.fields[field].slicer = Some(bits);
        sidecar_differences(stem, name, &cache, &table, None, &[])
    }

    /// probes/t4767-pivot-slicers: the selection filters the pivot (SL, SA, SM, SH, SD) and wins
    /// over the pivot's own hidden items (SX).
    #[test]
    fn a_slicers_selection_filters_its_pivot_over_the_hidden_items() {
        let wrong: Vec<String> = ["SL", "SA", "SM", "SH", "SD", "SX"]
            .into_iter()
            .flat_map(slicer_differences)
            .collect();
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    /// Sheet `name` of probes/t4767-pivot-timelines with its timeline's selection, if any, and
    /// `filters`.
    fn timeline_differences(name: &str, filters: Vec<PivotFilter>) -> Vec<String> {
        let stem = "t4767-pivot-timelines";
        let case = read(stem, "case");
        let (cache, mut table) = probe_pivot(&case, name);
        table.filters = filters;
        let timeline = &sheet(&case, name)["timelines"][0];
        let field = (cache.fields.iter())
            .position(|f| timeline["field"] == f.name.as_str())
            .unwrap();
        if let Some([start, end]) = timeline["selection"].as_array().map(Vec::as_slice) {
            let span = DateSpan::from_serials(start.as_f64().unwrap(), end.as_f64().unwrap());
            table.fields[field].timeline = span;
        }
        sidecar_differences(stem, name, &cache, &table, None, &[])
    }

    /// probes/t4767-pivot-timelines: the selection filters the pivot by a field on no axis (TL,
    /// TA, TD0, TD1; TN has none) and replaces the pivot's own date filter on the field (TX's
    /// Q2 2019 `dateBetween`, typed as the importer types it).
    #[test]
    fn a_timelines_selection_filters_its_pivot_over_its_date_filter() {
        let q2 = PivotFilter {
            field: 2,
            kind: PivotFilterKind::Date,
            measure: 0,
            eval_order: -1,
            id: 1,
            criteria: FilterCriteria::Custom {
                and: true,
                conditions: vec![
                    FilterCondition {
                        operator: FilterOperator::GreaterThanOrEqual,
                        value: "43556".into(),
                    },
                    FilterCondition {
                        operator: FilterOperator::LessThanOrEqual,
                        value: "43646".into(),
                    },
                ],
            },
            whole_day: false,
        };
        let mut wrong: Vec<String> = ["TL", "TN", "TA", "TD0", "TD1"]
            .into_iter()
            .flat_map(|name| timeline_differences(name, Vec::new()))
            .collect();
        wrong.extend(timeline_differences("TX", vec![q2]));
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    #[test]
    fn a_span_keeps_whole_days_first_and_last_included() {
        let span = DateSpan::from_serials(43466.0, 43555.0);
        assert_eq!(
            span,
            Some(DateSpan {
                first: 43466,
                last: 43555
            })
        );
        let span = span.unwrap_or(DateSpan { first: 0, last: 0 });
        assert!(span.keeps(&CacheItem::Date(43466.0)));
        assert!(span.keeps(&CacheItem::Date(43555.75)));
        assert!(!span.keeps(&CacheItem::Date(43556.0)));
        assert!(!span.keeps(&CacheItem::Date(43465.99)));
        assert!(span.keeps(&CacheItem::Number(43500.0)));
        assert!(!span.keeps(&CacheItem::Text("43500".into())));
        assert!(!span.keeps(&CacheItem::Missing));
    }

    #[test]
    fn a_reversed_or_unbounded_span_is_none() {
        assert_eq!(DateSpan::from_serials(43555.0, 43466.0), None);
        assert_eq!(DateSpan::from_serials(f64::NAN, 43466.0), None);
        assert_eq!(DateSpan::from_serials(0.0, 1e300), None);
        assert_eq!(DateSpan::from_serials(-1.0, 3.0), None);
    }
}
