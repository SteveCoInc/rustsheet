//! The typed model of a pivot's `<filters>` and a field's legacy `AutoShow` (T-4165): what the
//! importer reads from the definition parts it keeps verbatim, and `pivot::filter` applies.

use bitcode::{Decode, Encode};

/// A `<filter>` of a pivot's `<filters>` (ECMA-376 `CT_PivotFilter`), as the importer types it.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PivotFilter {
    /// `fld`: the cache field it filters.
    pub field: u32,
    /// What `type` (`ST_PivotFilterType`) makes it.
    pub kind: PivotFilterKind,
    /// `iMeasureFld`: the data field a value filter or Top-N compares (default 0).
    pub measure: u32,
    /// `evalOrder` (default 0). Not read by the engine: Excel applies the filters in `<filters>`
    /// order with every probe writing -1 or 0 (UNVERIFIED, T-4799, that it never orders by it).
    pub eval_order: i32,
    /// `id`. Not read by the engine, for the same reason as [`PivotFilter::eval_order`].
    pub id: u32,
    /// The `autoFilter`'s one `filterColumn` criterion: it decides what the filter keeps, not
    /// `stringValue1` (pivots.md "Pivot sorts and filters as Excel for the web renders them",
    /// probes/t4130-pivot-label-filters `CMix`).
    pub criteria: FilterCriteria,
    /// The x15 `pivotFilter` extension's `useWholeDay` (default off): a date filter compares the
    /// item's day, not its date-time (pivots.md "useWholeDay", probes/t4164-pivot-date-times).
    pub whole_day: bool,
}

/// The class of an `ST_PivotFilterType`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PivotFilterKind {
    /// `captionEqual` to `captionNotBetween`: the item's label.
    Caption,
    /// `valueEqual` to `valueNotBetween`: the item's value of data field `iMeasureFld`.
    Value,
    /// `count`: the Top-N by count.
    Count,
    /// `percent`: the Top-N by percent of the total.
    Percent,
    /// `sum`: the Top-N by sum.
    Sum,
    /// `dateEqual` to `dateNotBetween`, `M1`-`M12`, `Q1`-`Q4` and the relative dates: the item's
    /// date.
    Date,
    /// `unknown` and anything else: Excel for the web refuses such a file (pivots.md "Pivot flags
    /// Excel for the web refuses"); the engine applies nothing for it.
    Unknown,
}

impl PivotFilterKind {
    /// The class of the `ST_PivotFilterType` named `name`.
    #[must_use]
    pub fn from_type_name(name: &str) -> Self {
        let date_named = matches!(
            name,
            "today"
                | "yesterday"
                | "tomorrow"
                | "thisWeek"
                | "lastWeek"
                | "nextWeek"
                | "thisMonth"
                | "lastMonth"
                | "nextMonth"
                | "thisQuarter"
                | "lastQuarter"
                | "nextQuarter"
                | "thisYear"
                | "lastYear"
                | "nextYear"
                | "yearToDate"
        );
        match name {
            "count" => Self::Count,
            "percent" => Self::Percent,
            "sum" => Self::Sum,
            _ if name.starts_with("caption") => Self::Caption,
            _ if name.starts_with("value") => Self::Value,
            _ if name.starts_with("date") || date_named || DynamicDate::period(name).is_some() => {
                Self::Date
            }
            _ => Self::Unknown,
        }
    }

    /// Whether the filter keeps or drops an item by the item alone (caption and date filters),
    /// before any value filter ranks what is left.
    pub(super) fn on_items(self) -> bool {
        matches!(self, Self::Caption | Self::Date)
    }
}

/// The criterion of a filter's `autoFilter/filterColumn`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum FilterCriteria {
    /// `customFilters`: one or two conditions, all of them (`and="1"`) or any.
    Custom {
        /// `and` (default off).
        and: bool,
        /// The `customFilter`s, in file order.
        conditions: Vec<FilterCondition>,
    },
    /// `top10`: the Top-N's `top` (default on: the largest; off: the smallest) and `val`. The
    /// stored `filterVal` is not read: Excel recomputes it (probes/t4130-pivot-value-filters `TCv`).
    Top10 {
        /// `top`.
        top: bool,
        /// `val` as written: N, the percentage or the sum. One that is no finite number applies
        /// nothing (UNVERIFIED, T-4799).
        value: String,
    },
    /// `dynamicFilter`, by its `type`. Its stored `val`/`maxVal` are not read (`RtMv`).
    Dynamic(DynamicDate),
    /// `filters`: a values list. An item is kept when its label is one of `values`, ignoring
    /// case (probes/t4130-pivot-filter-attrs `FVals`), a blank item when `blank` is on, and a date
    /// item when a `dateGroupItem` covers it (UNVERIFIED, T-4799: only a dateGroupItem on a text
    /// field is probed, `FDgi`, and it keeps nothing).
    Values {
        /// The `filter@val`s.
        values: Vec<String>,
        /// `blank` (default off).
        blank: bool,
        /// The `dateGroupItem`s.
        dates: Vec<DateGroupItem>,
    },
    /// A criterion that keeps no item: an `iconFilter` (probes/t4130-pivot-filter-attrs `FIcon`).
    Nothing,
    /// No criterion the importer reads: the filter keeps every item (UNVERIFIED, T-4799).
    Everything,
}

/// One `customFilter`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct FilterCondition {
    /// `operator` (default `equal`).
    pub operator: FilterOperator,
    /// `val`, as written: a pattern with `*` and `?` for a caption filter, a number otherwise.
    pub value: String,
}

/// `ST_FilterOperator`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum FilterOperator {
    /// `equal`.
    Equal,
    /// `lessThan`.
    LessThan,
    /// `lessThanOrEqual`.
    LessThanOrEqual,
    /// `notEqual`.
    NotEqual,
    /// `greaterThanOrEqual`.
    GreaterThanOrEqual,
    /// `greaterThan`.
    GreaterThan,
}

impl FilterOperator {
    /// The operator an `ST_FilterOperator` name gives; `None` for an unknown one.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "equal" => Self::Equal,
            "lessThan" => Self::LessThan,
            "lessThanOrEqual" => Self::LessThanOrEqual,
            "notEqual" => Self::NotEqual,
            "greaterThanOrEqual" => Self::GreaterThanOrEqual,
            "greaterThan" => Self::GreaterThan,
            _ => return None,
        })
    }

    /// Whether `ordering` (the item against the criterion) satisfies the operator.
    pub(super) fn holds(self, ordering: std::cmp::Ordering) -> bool {
        use std::cmp::Ordering::{Equal, Greater, Less};
        match self {
            Self::Equal => ordering == Equal,
            Self::NotEqual => ordering != Equal,
            Self::LessThan => ordering == Less,
            Self::LessThanOrEqual => ordering != Greater,
            Self::GreaterThan => ordering == Greater,
            Self::GreaterThanOrEqual => ordering != Less,
        }
    }
}

/// A `dynamicFilter@type` of a date filter (`ST_DynamicFilterType`).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DynamicDate {
    /// `M1` to `M12`: the month of any year (probes/t4130-pivot-date-filters M1 keeps 1/15/2024
    /// and 1/10/2025).
    Month(u8),
    /// `Q1` to `Q4`: the quarter of any year.
    Quarter(u8),
    /// A period relative to today: its offset in [`RelativeUnit`]s (-1 last, 0 this, 1 next).
    Relative(RelativeUnit, i8),
    /// `yearToDate`: from January 1 of today's year to today.
    YearToDate,
    /// Any other type (`aboveAverage`, `belowAverage`, `null`): keeps every item (UNVERIFIED,
    /// T-4799).
    Other,
}

/// The period of a relative [`DynamicDate`].
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum RelativeUnit {
    /// A day (`today`, `yesterday`, `tomorrow`).
    Day,
    /// A week, Sunday to Saturday.
    Week,
    /// A calendar month.
    Month,
    /// A calendar quarter.
    Quarter,
    /// A calendar year.
    Year,
}

impl DynamicDate {
    /// The type a `dynamicFilter@type` names.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        use RelativeUnit::{Day, Month, Quarter, Week, Year};
        if let Some(period) = Self::period(name) {
            return period;
        }
        let (unit, offset) = match name {
            "today" => (Day, 0),
            "yesterday" => (Day, -1),
            "tomorrow" => (Day, 1),
            "thisWeek" => (Week, 0),
            "lastWeek" => (Week, -1),
            "nextWeek" => (Week, 1),
            "thisMonth" => (Month, 0),
            "lastMonth" => (Month, -1),
            "nextMonth" => (Month, 1),
            "thisQuarter" => (Quarter, 0),
            "lastQuarter" => (Quarter, -1),
            "nextQuarter" => (Quarter, 1),
            "thisYear" => (Year, 0),
            "lastYear" => (Year, -1),
            "nextYear" => (Year, 1),
            "yearToDate" => return Self::YearToDate,
            _ => return Self::Other,
        };
        Self::Relative(unit, offset)
    }

    /// `M1` to `M12` and `Q1` to `Q4`.
    fn period(name: &str) -> Option<Self> {
        let number =
            |rest: &str, last: u8| rest.parse::<u8>().ok().filter(|n| (1..=last).contains(n));
        if let Some(month) = name.strip_prefix('M').and_then(|rest| number(rest, 12)) {
            return Some(Self::Month(month));
        }
        name.strip_prefix('Q')
            .and_then(|rest| number(rest, 4))
            .map(Self::Quarter)
    }
}

/// A `dateGroupItem` of a values list: the date parts down to `grouping`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub struct DateGroupItem {
    /// `year`.
    pub year: i32,
    /// `month` (1 to 12).
    pub month: u32,
    /// `day`.
    pub day: u32,
    /// `hour`.
    pub hour: u32,
    /// `minute`.
    pub minute: u32,
    /// `second`.
    pub second: u32,
    /// `dateTimeGrouping`: how many of the parts it names, 1 (year) to 6 (second).
    pub parts: u8,
}

/// A pivot field's legacy `AutoShow` (`autoShow="1"`): a count Top-N on the field (pivots.md
/// "Pivot sorts and filters as Excel for the web renders them", probes/t4130-pivot-field-filters
/// AS, `ASb`).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub struct AutoShow {
    /// `itemPageCount` (default 10): N.
    pub count: u32,
    /// `topAutoShow` (default on): the largest, or the smallest.
    pub top: bool,
    /// `rankBy`: the data field it ranks by; absent or negative reads as the first (UNVERIFIED,
    /// T-4799).
    pub measure: u32,
}
