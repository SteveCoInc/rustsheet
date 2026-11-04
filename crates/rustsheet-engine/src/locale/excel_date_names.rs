//! Excel's own month, day and day-half names per locale, which a `[$-...]` number-format tag
//! selects (`crate::formatter::locale_tag`). They are Excel's locale data (Windows'), not CLDR
//! (`[$-407]mmm` of March is "Mrz" where CLDR and `locales.bin` have "Mär"), and they are data, not
//! code: the three tables beside this file hold Excel's answers for every LCID that has a locale and
//! every BCP-47 tag Excel knows, generated from probes/t3666-locale-names and
//! probes/t3666-locale-tag-names (excel-graph, 2026-11-04) by `rustsheet-excel-locales` (tools/):
//!
//! - `excel_names.tsv`: the distinct name sets, one per line (12 `mmm`, 12 `mmmm`, 7 `ddd`,
//!   7 `dddd`, then the day halves), numbered from 0, with `\\`, `\t` and `\n` escaped (Excel's
//!   `mmmm` under 0xF803 holds tabs);
//! - `excel_lcids.tsv`: each LCID with a locale (hex) and its name set;
//! - `excel_tags.tsv`: each tag, the LCID [MS-LCID] gives it (`-` for none) and its name set.
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

impl DateNames {
    /// These names with other month names, `mmm` and `mmmm` alike (a calendar's own months).
    pub(crate) const fn with_months(&self, months: [&'static str; 12]) -> DateNames {
        DateNames {
            months_short: months,
            months,
            ..*self
        }
    }

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

struct Tables {
    names: Vec<DateNames>,
    /// Sorted by LCID, as the file is.
    lcids: Vec<(u16, usize)>,
    tags: Vec<(&'static str, Option<u16>, usize)>,
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
        let tags = rows(include_str!("excel_tags.tsv"))
            .filter_map(|fields| match fields.as_slice() {
                [tag, lcid, names] => Some((
                    *tag,
                    u16::from_str_radix(lcid, 16).ok(),
                    names.parse().ok()?,
                )),
                _ => None,
            })
            .collect();
        Tables { names, lcids, tags }
    })
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
/// French, `[$-zh-XX]` Traditional Chinese like `[$-zh]`); `None` when Excel knows neither.
pub(crate) fn tag_locale(tag: &str) -> Option<TagLocale> {
    let tables = tables();
    let find = |wanted: &str| {
        tables
            .tags
            .iter()
            .find(|(t, _, _)| t.eq_ignore_ascii_case(wanted))
            .map(|(_, lcid, names)| TagLocale {
                lcid: *lcid,
                names: tables.names(*names),
            })
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
    use super::{lcid_names, rows, tag_locale, unescape, DateNames, EN};

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
    }

    #[test]
    fn lookups() {
        assert_eq!(lcid_names(0x409), Some(&EN));
        assert_eq!(lcid_names(0x40C).map(|n| n.months[2]), Some("mars"));
        assert_eq!(lcid_names(0x1409).map(|n| n.pm), Some("pm"));
        assert_eq!(lcid_names(0x2412), None);
        assert_eq!(lcid_names(0x1), None);
        assert_eq!(tag_locale("FR-fr").map(|t| t.lcid), Some(Some(0x40C)));
        assert_eq!(tag_locale("fr-XX").map(|t| t.names.months[2]), Some("mars"));
        assert_eq!(tag_locale("pt-AO").map(|t| t.lcid), Some(None));
        assert!(tag_locale("xx").is_none());
        // 0xF803 is Catalan with a `mmmm` holding tabs (probes/t3666-locale-names A584).
        assert!(lcid_names(0xF803).is_some_and(|n| n.months[0].contains('\t')));
        assert_eq!(unescape("a\\tb\\\\c"), "a\tb\\c");
    }
}
