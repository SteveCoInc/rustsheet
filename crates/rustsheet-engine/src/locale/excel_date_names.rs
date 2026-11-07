//! Excel's own month, day and day-half names per locale, which a `[$-...]` number-format tag
//! selects (`crate::formatter::locale_tag`). They are Excel's locale data (Windows'), not CLDR
//! (`[$-407]mmm` of March is "Mrz" where CLDR and `locales.bin` have "Mär"), and they are data, not
//! code: the four tables beside this file hold Excel's answers for every LCID that has a locale,
//! every BCP-47 tag Excel knows and every calendar digit under each, generated from
//! probes/t3666-* and t4061-* (excel-graph, 2026-11-04) by `rustsheet-excel-locales` (tools/). The
//! generator leaves out what Excel does not resolve as a locale (a tag whose `A/P` stays as
//! written or that prints another calendar, and an LCID whose hex spells a tag Excel knows);
//! which table tags Excel reads as hex instead is the resolver's rule (`formatter::locale_tag`).
//!
//! - `excel_names.tsv`: the distinct name sets, one per line (12 `mmm`, 12 `mmmm`, 7 `ddd`,
//!   7 `dddd`, then the day halves), numbered from 0, with `\\`, `\t` and `\n` escaped (Excel's
//!   `mmmm` under 0xF803 holds tabs);
//! - `excel_lcids.tsv`: each LCID with a locale (hex) and its name set;
//! - `excel_tags.tsv`: each tag, the LCID [MS-LCID] gives it (`-` for none) and its name set;
//! - `excel_calendars.tsv`: the names of each calendar digit of a tag ([`calendar_names`]) under
//!   every LCID, from probes/t4061-calendar-names-digest and -full: a default per digit, the
//!   names of an LCID with no locale (`-`), and the LCIDs whose names differ (`=` for their own).
//!
//! An LCID or tag that is in no table has no locale (number-formats.md "Which LCIDs have a locale").

use std::sync::OnceLock;

/// The date names and day-half strings of one locale.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct DateNames {
    /// `mmm`, January first.
    pub months_short: [&'static str; 12],
    /// `mmmm`, January first; `mmmmm` is its first character.
    pub months: [&'static str; 12],
    /// `ddd`, Sunday first.
    pub days_short: [&'static str; 7],
    /// `dddd`, Sunday first.
    pub days: [&'static str; 7],
    /// What `AM/PM` and `A/P` print before noon.
    pub am: &'static str,
    /// What `AM/PM` and `A/P` print from noon.
    pub pm: &'static str,
}

/// The en-US names: what a tag with no locale prints, and the fallback of every lookup here.
pub(crate) static EN_US: &DateNames = &EN;

static EN: DateNames = DateNames {
    months_short: [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ],
    months: [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ],
    days_short: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
    days: [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ],
    am: "AM",
    pm: "PM",
};

/// Excel's built-in custom lists (en-US), in the engine's chosen order (UNVERIFIED, T-4799): the
/// abbreviated day names, the day names, the abbreviated month names, the month names, each from
/// Sunday or January. A pivot's label sort under `customListSort` orders a field's names by them
/// (`pivot::sort::labels`; docs/excel-semantics/pivots.md "Pivot sorts and filters as Excel for the
/// web renders them": the abbreviated day names, probes/t4130-pivot-sort CL1). The other three
/// lists, and the order between lists, are UNVERIFIED (unprobed, T-4799).
pub(crate) static BUILT_IN_CUSTOM_LISTS: [&[&str]; 4] =
    [&EN.days_short, &EN.days, &EN.months_short, &EN.months];

impl DateNames {
    /// One name set from a table line's fields, `None` if it is malformed.
    fn from_fields(fields: &[&'static str]) -> Option<DateNames> {
        Some(DateNames {
            months_short: fields.get(0..12)?.try_into().ok()?,
            months: fields.get(12..24)?.try_into().ok()?,
            days_short: fields.get(24..31)?.try_into().ok()?,
            days: fields.get(31..38)?.try_into().ok()?,
            am: fields.get(38)?,
            pm: fields.get(39)?,
        })
    }
}

/// A BCP-47 tag Excel knows: the LCID [MS-LCID] gives it, if any, and its names.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TagLocale {
    pub lcid: Option<u16>,
    pub names: &'static DateNames,
}

/// The names a calendar digit prints under one LCID (`excel_calendars.tsv`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CalendarNames {
    /// The LCID's own names (`[$-1040C]` is French, `[$-30411]` Japanese).
    Own,
    /// One of the calendar's name sets (`[$-6040C]mmmm` is "Chaaban", `[$-60409]` "Sha'ban"),
    /// with the 13 month names of a Hebrew leap year (the set's 12 are a common year's), and
    /// whether they are those of an LCID with no locale (`[$-80000]`, also `[$-80CEB]`), whose
    /// months go by Gregorian number.
    Set {
        names: &'static DateNames,
        leap_months: Option<&'static [&'static str; 13]>,
        no_locale: bool,
    },
}

/// The names field of a row of `excel_calendars.tsv`.
#[derive(Clone, Copy)]
enum RowNames {
    /// `=`: the LCID's own names.
    Own,
    /// `-`: the names of an LCID with no locale.
    NoLocale,
    /// A name set, and the index of its leap months.
    Set(usize, Option<usize>),
}

/// Whose names [`calendar_names`] reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CalendarKey {
    /// An LCID: its own row, else the digit's default when it has a locale and the names of an
    /// LCID with none when it has none.
    Lcid(u16),
    /// The digit's default names (a BCP-47 tag with no LCID of its own).
    Default,
}

/// A row key of `excel_calendars.tsv`, ordered: an LCID, then `*`, then `-`.
const DEFAULT_ROW: u32 = 0x1_0000;
const NO_LOCALE_ROW: u32 = 0x1_0001;

struct Tables {
    names: Vec<DateNames>,
    /// Sorted by LCID, as the file is.
    lcids: Vec<(u16, usize)>,
    /// Sorted by the lower-cased tag.
    tags: Vec<(String, Option<u16>, usize)>,
    /// Sorted by (digit, row key).
    calendars: Vec<((u32, u32), RowNames)>,
    /// The leap-year months of the Hebrew rows.
    leap_months: Vec<[&'static str; 13]>,
}

impl Tables {
    fn names(&self, index: usize) -> &DateNames {
        self.names.get(index).unwrap_or(&EN)
    }
}

/// The non-comment lines of a table, split into fields.
fn rows(text: &'static str) -> impl Iterator<Item = Vec<&'static str>> {
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect())
}

/// A table field with its escapes undone. The few names that hold one are copied once, for the
/// life of the process, as the tables are.
fn unescape(field: &'static str) -> &'static str {
    if !field.contains('\\') {
        return field;
    }
    let mut text = String::with_capacity(field.len());
    let mut chars = field.chars();
    while let Some(c) = chars.next() {
        text.push(match (c, c == '\\') {
            (_, true) => match chars.next() {
                Some('t') => '\t',
                Some('n') => '\n',
                Some(other) => other,
                None => '\\',
            },
            (c, false) => c,
        });
    }
    text.leak()
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        // A malformed line keeps its place (the numbering is by line) with the en-US names; the
        // tables are generated and a test checks every line parses.
        let names = rows(include_str!("excel_names.tsv"))
            .map(|fields| {
                let fields: Vec<&'static str> = fields.into_iter().map(unescape).collect();
                DateNames::from_fields(&fields).unwrap_or(EN)
            })
            .collect();
        let lcids = rows(include_str!("excel_lcids.tsv"))
            .filter_map(|fields| match fields.as_slice() {
                [lcid, names] => Some((u16::from_str_radix(lcid, 16).ok()?, names.parse().ok()?)),
                _ => None,
            })
            .collect();
        let mut tags: Vec<_> = rows(include_str!("excel_tags.tsv"))
            .filter_map(|fields| match fields.as_slice() {
                [tag, lcid, names] => Some((
                    tag.to_ascii_lowercase(),
                    u16::from_str_radix(lcid, 16).ok(),
                    names.parse().ok()?,
                )),
                _ => None,
            })
            .collect();
        tags.sort_by(|a, b| a.0.cmp(&b.0));
        let mut leap_months = Vec::new();
        let mut calendars: Vec<_> = rows(include_str!("excel_calendars.tsv"))
            .filter_map(|fields| {
                let [digit, key, names, leap @ ..] = fields.as_slice() else {
                    return None;
                };
                let key = match *key {
                    "*" => DEFAULT_ROW,
                    "-" => NO_LOCALE_ROW,
                    lcid => u32::from_str_radix(lcid, 16).ok()?,
                };
                let names = match *names {
                    "=" => RowNames::Own,
                    "-" => RowNames::NoLocale,
                    set => {
                        let leap = <[&'static str; 13]>::try_from(leap).ok().map(|months| {
                            leap_months.push(months.map(unescape));
                            leap_months.len() - 1
                        });
                        RowNames::Set(set.parse().ok()?, leap)
                    }
                };
                Some(((u32::from_str_radix(digit, 16).ok()?, key), names))
            })
            .collect();
        calendars.sort_by_key(|(key, _)| *key);
        Tables {
            names,
            lcids,
            tags,
            calendars,
            leap_months,
        }
    })
}

/// The names calendar digit `digit` prints for `key`, `None` for a digit with no names of its own
/// (0, and 0x18 and above, the locale's own calendar). Evidence: probes/t4061-calendar-names-digest
/// (every LCID under digits 1-0x17), t4061-calendar-names-full; number-formats.md "Calendars of a
/// locale tag".
pub(crate) fn calendar_names(digit: u32, key: CalendarKey) -> Option<CalendarNames> {
    let tables = tables();
    let row = |row_key: u32| {
        let find = |key: u32| {
            let at = tables
                .calendars
                .binary_search_by_key(&(digit, key), |(key, _)| *key)
                .ok()?;
            tables.calendars.get(at).map(|(_, names)| *names)
        };
        let (names, no_locale) = match find(row_key)? {
            RowNames::NoLocale => (find(NO_LOCALE_ROW)?, true),
            names => (names, row_key == NO_LOCALE_ROW),
        };
        Some(match names {
            RowNames::Set(set, leap) => CalendarNames::Set {
                names: tables.names(set),
                leap_months: leap.and_then(|i| tables.leap_months.get(i)),
                no_locale,
            },
            RowNames::Own | RowNames::NoLocale => CalendarNames::Own,
        })
    };
    match key {
        CalendarKey::Lcid(lcid) => row(u32::from(lcid)).or_else(|| {
            row(if lcid_names(lcid).is_some() {
                DEFAULT_ROW
            } else {
                NO_LOCALE_ROW
            })
        }),
        CalendarKey::Default => row(DEFAULT_ROW),
    }
}

/// The names of `lcid`, or `None` when Excel gives it no locale (`[$-2412]`, `[$-1]`).
pub(crate) fn lcid_names(lcid: u16) -> Option<&'static DateNames> {
    let tables = tables();
    let at = tables
        .lcids
        .binary_search_by_key(&lcid, |(id, _)| *id)
        .ok()?;
    tables.lcids.get(at).map(|(_, names)| tables.names(*names))
}

/// The locale of a BCP-47 tag, matched case-insensitively, else of its language (`[$-fr-XX]` is
/// French, `[$-zh-XX]` Traditional Chinese like `[$-zh]`), taking only table entries `is_tag`
/// accepts (the resolver's rule for a tag Excel reads as hex instead); `None` when Excel knows
/// neither.
pub(crate) fn tag_locale(
    tag: &str,
    is_tag: impl Fn(&str, &TagLocale) -> bool,
) -> Option<TagLocale> {
    let tables = tables();
    let find = |wanted: &str| {
        let wanted = wanted.to_ascii_lowercase();
        let at = tables
            .tags
            .binary_search_by(|(t, _, _)| t.as_str().cmp(&wanted))
            .ok()?;
        let (_, lcid, names) = tables.tags.get(at)?;
        let locale = TagLocale {
            lcid: *lcid,
            names: tables.names(*names),
        };
        is_tag(&wanted, &locale).then_some(locale)
    };
    find(tag).or_else(|| {
        find(
            tag.split('-')
                .next()
                .filter(|l| !l.is_empty() && *l != tag)?,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{
        calendar_names, lcid_names, rows, tag_locale, unescape, CalendarKey, CalendarNames,
        DateNames, EN,
    };

    #[test]
    fn every_table_line_parses() {
        for fields in rows(include_str!("excel_names.tsv")) {
            assert!(DateNames::from_fields(&fields).is_some(), "{fields:?}");
        }
        for fields in rows(include_str!("excel_lcids.tsv")) {
            assert_eq!(fields.len(), 2, "{fields:?}");
        }
        for fields in rows(include_str!("excel_tags.tsv")) {
            assert_eq!(fields.len(), 3, "{fields:?}");
        }
        for fields in rows(include_str!("excel_calendars.tsv")) {
            assert!(matches!(fields.len(), 3 | 16), "{fields:?}");
        }
        assert_eq!(
            super::tables().calendars.len(),
            rows(include_str!("excel_calendars.tsv")).count()
        );
    }

    #[test]
    fn name_set_indexes_are_in_range() {
        let tables = super::tables();
        let count = tables.names.len();
        assert!(tables.lcids.iter().all(|(_, i)| *i < count));
        assert!(tables.tags.iter().all(|(_, _, i)| *i < count));
        assert!(tables.calendars.iter().all(|(_, names)| match names {
            super::RowNames::Set(i, leap) => {
                *i < count && leap.is_none_or(|l| l < tables.leap_months.len())
            }
            super::RowNames::Own | super::RowNames::NoLocale => true,
        }));
    }

    #[test]
    fn lookups() {
        assert_eq!(lcid_names(0x409), Some(&EN));
        assert_eq!(lcid_names(0x40C).map(|n| n.months[2]), Some("mars"));
        assert_eq!(lcid_names(0x1409).map(|n| n.pm), Some("pm"));
        assert_eq!(lcid_names(0x2412), None);
        assert_eq!(lcid_names(0x1), None);
        let any = |_: &str, _: &super::TagLocale| true;
        assert_eq!(tag_locale("FR-fr", any).map(|t| t.lcid), Some(Some(0x40C)));
        assert_eq!(
            tag_locale("fr-XX", any).map(|t| t.names.months[2]),
            Some("mars")
        );
        assert_eq!(tag_locale("pt-AO", any).map(|t| t.lcid), Some(None));
        assert!(tag_locale("xx", any).is_none());
        // 0xF803 is Catalan with a `mmmm` holding tabs (probes/t3666-locale-names A584).
        assert!(lcid_names(0xF803).is_some_and(|n| n.months[0].contains('\t')));
        assert_eq!(unescape("a\\tb\\\\c"), "a\tb\\c");
    }

    #[test]
    fn calendar_lookups() {
        let month = |digit, key| match calendar_names(digit, key) {
            Some(CalendarNames::Set { names, .. }) => Some(names.months[2]),
            Some(CalendarNames::Own) => Some("own"),
            None => None,
        };
        // probes/t4061-calendar-names-digest: Hijri month 3 by language, the no-locale default.
        assert_eq!(month(6, CalendarKey::Lcid(0x40C)), Some("Rabi' Aoual"));
        assert_eq!(month(6, CalendarKey::Lcid(0x409)), Some("Rabi' Awwal"));
        assert_eq!(month(6, CalendarKey::Lcid(0x2412)), Some("ربيع\u{a0}الأول"));
        assert_eq!(month(6, CalendarKey::Default), Some("Rabi' Awwal"));
        assert_eq!(month(1, CalendarKey::Lcid(0x40C)), Some("own"));
        assert_eq!(month(0, CalendarKey::Lcid(0x40C)), None);
    }
}
