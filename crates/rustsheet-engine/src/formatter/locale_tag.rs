//! The `[$-...]` locale tag of a number format: what Excel shows for date names and `AM/PM` under
//! it, with names from [`crate::locale::excel_date_names`]. Evidence: probes/text-locale-date-names,
//! probes/text-locale-name-tables, probes/text-locale-am-neutral (excel-graph, 2026-10-18);
//! docs/excel-semantics/number-formats.md "Locale tags localize date names".

use chrono::NaiveDate;

use super::dates::convert_to_serial_number;
use super::numerals::Numerals;
pub use crate::locale::excel_date_names::DateNames;
use crate::locale::excel_date_names::{
    calendar_names, lcid_names, tag_locale, CalendarKey, CalendarNames, TagLocale, EN_US,
};

/// The calendar a locale tag selects: what `yy`, `yyyy`, `b`, `e` and `g` print.
/// Evidence: probes/text-locale-calendars, text-locale-calendars-detail, text-locale-japanese-year
/// (excel-graph, 2026-10-18, T-1859); number-formats.md "Calendars of a locale tag".
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Calendar {
    /// Gregorian years; `g` prints nothing and `e` is the four-digit year.
    Gregorian,
    /// Japanese eras: `g`/`gg`/`ggg` the era name, `e`/`ee` the era year. `explicit` (calendar
    /// digit 3) makes `yy`/`yyyy` the era year too; a Japanese locale's own calendar does that only
    /// after a `g` in the section (`[$-411]yy|g|yy` is "23|R|05").
    Japanese { explicit: bool },
    /// Calendar digit 4 (Taiwan): Gregorian, but `yy` prints four digits.
    Taiwan,
    /// Calendar digit 5: `yy`/`yyyy` count from 2333 BC; `b` and `e` stay Gregorian.
    Korean,
    /// Calendar digit 7: `yy`/`yyyy` and `b` count from 543 BC; `e` stays Gregorian.
    Thai,
    /// Calendar digits E, F, 11, 12 and 13: the lunar year, month and day for every date token, `b`
    /// and `e` included (`[$-110409]yyyy|m|d|e|bbbb` of 2024-01-09 is "2023|12|28|2023|2023";
    /// probes/text-locale-calendars A113, text-locale-lunar-tokens A75).
    Lunar(LunarTable),
    /// Calendar digit 6, or a `B2` prefix before the tag: Excel's Hijri date
    /// (`date_names::hijri_ymd`) for every date token; `yy`, `yyyy`, `e`, `b` and `bbbb` print
    /// the Hijri year and `g` nothing (`[$-60409]yy|yyyy|e|b|bbbb` of 2023-03-15 is
    /// "44|1444|1444|44|1444"; probes/t2729-hijri-locale-tag A203).
    Hijri,
    /// Calendar digit 10: the Saka year, month and day for every date token, `b` and `e`
    /// included (`[$-100409]yyyy-mm-dd|yy|e|b` of 2023-03-15 is "1944-12-24|44|1944|44";
    /// `calendar_dates::saka_ymd`).
    Saka,
    /// Calendar digit 8: the Hebrew year, month and day for every date token
    /// (`[$-80409]yyyy-mm-dd|e|b` of 2023-03-15 is "5783-06-22|5783|83";
    /// `calendar_dates::hebrew_ymd`); its month names are [`Localized::leap_months`] and the
    /// name set's.
    Hebrew,
    /// Calendar digit 16: the Persian (solar Hijri) year, month and day for every date token
    /// (`[$-160409]yyyy-mm-dd|yy|e|b` of 2023-03-15 is "1401-12-24|01|1401|01";
    /// `table_calendars::persian_ymd`).
    Persian,
    /// Calendar digit 17: Um al-Qura, Excel's month table from 1318 AH (1900-04-30) to 1500 AH
    /// and the [`Calendar::Hijri`] date outside it (`table_calendars::um_al_qura_ymd`).
    UmAlQura,
}

/// Which month table a lunar calendar digit uses: F and 11 follow China; E, 12 and 13 one that
/// starts some months a day later (`[$-E0409]m` of 2023-05-19 is "4", `[$-110409]m` "5";
/// probes/text-locale-lunar-months-1900 columns A-F).
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LunarTable {
    /// Calendar digits F and 11.
    Chinese,
    /// Calendar digits E, 12 and 13.
    Eastern,
}

/// A resolved locale tag: the names of months and days, the day-half strings and the calendar.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Localized {
    /// Month and day names, which some calendar digits override (`[$-90409]mmmm` is "mars").
    pub names: &'static DateNames,
    /// `AM/PM` and `A/P` always follow the LCID, whatever the calendar (`[$-30409]h AM/PM` is
    /// "6 PM", `[$-30411]h AM/PM` "6 午後"). `None` under an LCID with no locale, whose calendar
    /// digit still applies: `A/P` stays as written (`[$-E08DE]h A/P` is "6 P";
    /// probes/t1924-locale-tag-extended-hex B13).
    pub day_half: Option<&'static DateNames>,
    pub calendar: Calendar,
    /// A Japanese LCID: after a `g`, `yy` and `yyyy` print the whole Gregorian year in its
    /// non-Japanese calendars (`[$-10411]g|yy` is "|2023", `[$-70411]g|yyyy` "|2023", but
    /// `[$-70409]g|yyyy` "|2566"; probes/text-locale-era-gregorian).
    pub japanese_lcid: bool,
    /// A `B2` before a calendar-3, -5 or -7 tag: every year token converts once more than it
    /// would without the `B2` ([`japanese_era`]; `calendar_dates::year_token_text`). Before a
    /// calendar-4 tag the years are a garbled Minguo year instead (`calendar_dates::taiwan_after_b2`).
    pub year_twice: bool,
    /// A `B2` before a calendar-1, -3 or -5 tag: the names every name token prints from the
    /// section's first `mmm` on, in place of [`Localized::names`] (`date_names::NameTokens`).
    pub names_after_mmm: Option<&'static DateNames>,
    /// A calendar read with the names of an LCID with no locale (`excel_date_names::CalendarNames`),
    /// whose months go by Gregorian number: a Hebrew or lunar month past the twelfth has none (`[$-80000]mmmm` and
    /// `[$-E0001]mmmm` of a month 13 print nothing; probes/t2779-hebrew-saka-more E1,
    /// t4061-calendar-names-full A496). With names of its own a lunar month 13 or 14 takes the
    /// twelfth name (`[$-110409]mmmm` of 2024-01-11 is "十二月"; text-locale-lunar-edges A26).
    pub months_by_number: bool,
    /// The 13 month names of a Hebrew leap year under a Hebrew calendar with names of its own
    /// (`excel_date_names::CalendarNames`); `None` otherwise.
    pub leap_months: Option<&'static [&'static str; 13]>,
}

impl Localized {
    /// The common case: no era-year doubling and no names after `mmm`.
    fn new(
        names: &'static DateNames,
        day_half: Option<&'static DateNames>,
        calendar: Calendar,
        japanese_lcid: bool,
    ) -> Self {
        Self {
            names,
            day_half,
            calendar,
            japanese_lcid,
            year_twice: false,
            names_after_mmm: None,
            months_by_number: false,
            leap_months: None,
        }
    }
}

/// What a `[$-...]` tag (with no currency symbol) does to its section.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LocaleTag {
    /// A locale with probed names: date names and `AM/PM` follow it, and `A/P` prints its `AM/PM`
    /// strings (`TEXT(0.75,"[$-409]h A/P")` is "6 PM").
    Names(Localized),
    /// `[$-F800]` or `[$-x-sysdate]`: the section is the system long date, whatever else it holds.
    SystemDate,
    /// `[$-F400]` or `[$-x-systime]`: the section is the system long time.
    SystemTime,
    /// A tag Excel cannot read (`[$-]`, a non-ASCII one), or one read as extended hex with no
    /// locale and no calendar digit (`[$-zzz]`): it prints nothing and changes nothing
    /// (`B2[$-zzz]` is a bare `B2`). A hex LCID with no locale is not silent: it is en-US and
    /// Gregorian with `A/P` as written ([`localize`]).
    Silent,
}

/// The system long date format of en-US Windows (`[$-F800]`).
pub const SYSTEM_LONG_DATE: &str = "dddd, mmmm d, yyyy";
/// The system long time format of en-US Windows (`[$-F400]`).
pub const SYSTEM_LONG_TIME: &str = "h:mm:ss AM/PM";
/// The tag value of the system long date (`[$-F800]`).
const SYSTEM_DATE_LCID: u32 = 0xF800;
/// The tag value of the system long time (`[$-F400]`).
const SYSTEM_TIME_LCID: u32 = 0xF400;

/// Resolves the text between `[$-` and `]`, in Excel's order: a hex LCID that has a locale
/// (`[$-40C]`, `[$-C]`; `excel_date_names::lcid_names`), then a BCP-47 tag Excel knows, matched
/// case-insensitively and then by its language (`[$-fr-FR]`, `[$-fr-XX]`, `[$-de]`, which is not
/// LCID 0xDE), then the hex value of an LCID with no locale (`[$-2412]`, `[$-80000]`), then any other
/// tag read as extended hex ([`extended_hex`]: `[$-fr_FR]` is the Saka calendar over an LCID with
/// no locale). The value's calendar digit then picks the calendar and its names ([`localize`]).
/// [`resolve_after_hijri_prefix`] resolves a tag that follows a `B2` prefix in its section.
pub fn resolve(tag: &str) -> LocaleTag {
    resolve_with(tag, false).0
}

/// [`resolve`] for a tag after a `B2` prefix in its section: a hex or BCP-47 tag in its locale's
/// own calendar (digit 0) takes the Hijri calendar, digit 6, with that LCID's Hijri names
/// (`B2[$-40C]mmmm` is "Chaaban", `B2[$-0]mmmm` "شعبان"). Any other calendar digit wins over the
/// prefix (`B2[$-10409]yyyy` is "2023", `B2[$-110409]yyyy|m` of 2024-01-09 "2023|12"). An
/// extended-hex tag, and so `[$-zzz]`, is not resolved this way: `B2[$-zzz]` is a bare `B2`
/// (probes/t2729-hijri-locale-tag A243-A264, t2729-hijri-locale-tag-names A239-A277;
/// text-locale-lunar-tokens A79). The digit's [`CalendarSpec`] says what else the `B2` does.
pub fn resolve_after_hijri_prefix(tag: &str) -> LocaleTag {
    match resolve_with(tag, true) {
        (LocaleTag::Names(localized), Some(spec)) => LocaleTag::Names(Localized {
            year_twice: spec.b2_twice,
            names_after_mmm: spec.b2_names_after_mmm.and_then(lcid_names),
            ..localized
        }),
        (other, _) => other,
    }
}

/// [`resolve`], with digit 0 read as 6 when `hijri_prefix`, and the spec of the calendar digit it
/// resolved to.
fn resolve_with(tag: &str, hijri_prefix: bool) -> (LocaleTag, Option<CalendarSpec>) {
    if tag.eq_ignore_ascii_case("x-sysdate") {
        return (LocaleTag::SystemDate, None);
    }
    if tag.eq_ignore_ascii_case("x-systime") {
        return (LocaleTag::SystemTime, None);
    }
    let hex = hex_value(tag);
    let lcid_of = |value: u32| (value & 0xFFFF) as u16;
    if let Some(value) = hex.filter(|value| lcid_names(lcid_of(*value)).is_some()) {
        return localize(Source::Lcid(lcid_of(value)), value, hijri_prefix);
    }
    if let Some(locale) = known_tag(tag) {
        let source = Source::Tag {
            names: locale.names,
            lcid: locale.lcid,
        };
        return localize(source, 0, hijri_prefix);
    }
    match hex {
        Some(value) => localize(Source::Lcid(lcid_of(value)), value, hijri_prefix),
        None => extended_hex(tag).map_or((LocaleTag::Silent, None), |value| {
            silent_if_inert(localize(Source::Lcid(lcid_of(value)), value, false))
        }),
    }
}

/// The locale of a BCP-47 tag Excel knows (`excel_date_names::tag_locale`). A table tag with no
/// LCID that Excel reads as extended hex is not one: its reading, in its own calendar, is an LCID
/// with the same names, and the tag does not spell that LCID's digits (`[$-bgc]` is 0xC0C, fr-CA,
/// so `[$-bgc-IN]` reads as extended hex too and prints English; `[$-ceb]` spells 0xCEB, which has
/// no locale, and is Cebuano; probes/t3666-locale-tag-names A89, t4061-calendars-without-locale).
fn known_tag(tag: &str) -> Option<TagLocale> {
    tag_locale(tag, |entry, locale| {
        let Some(value) = extended_hex(entry).filter(|value| value >> 16 == 0) else {
            return true;
        };
        let reads_as_lcid =
            lcid_names(value as u16).is_some_and(|names| std::ptr::eq(names, locale.names));
        locale.lcid.is_some() || !reads_as_lcid || format!("{value:x}") == entry
    })
}

/// An extended-hex reading with no locale and no calendar digit changes nothing: `B2[$-zzz]` is a
/// bare `B2` (probes/t2729-hijri-locale-tag-names A239:A263), where a hex LCID with no locale is
/// en-US and Gregorian (`[$-D001]b` is "23"; probes/t4061-calendars-without-locale A1).
fn silent_if_inert(
    resolved: (LocaleTag, Option<CalendarSpec>),
) -> (LocaleTag, Option<CalendarSpec>) {
    match resolved {
        (LocaleTag::Names(localized), None) if localized.day_half.is_none() => {
            (LocaleTag::Silent, None)
        }
        resolved => resolved,
    }
}

/// What a tag names: an LCID, which may have no locale, or a BCP-47 tag Excel knows, with its own
/// names and the LCID [MS-LCID] gives it, if any (which may have no locale: `ar` is LCID 1).
#[derive(Clone, Copy)]
enum Source {
    Lcid(u16),
    Tag {
        names: &'static DateNames,
        lcid: Option<u16>,
    },
}

impl Source {
    /// The locale's own names; `None` for an LCID with no locale.
    fn own_names(self) -> Option<&'static DateNames> {
        match self {
            Source::Lcid(lcid) => lcid_names(lcid),
            Source::Tag { names, .. } => Some(names),
        }
    }

    /// The LCID whose calendar names a calendar digit reads. A tag reads its LCID's, or when that
    /// LCID has no locale its language's primary LCID's, or with neither the digit's default names
    /// (`B2[$-ar]` the Arabic Hijri names, `B2[$-bg]`, `B2[$-qps-ploc]` and `B2[$-pt-AO]` the
    /// default English ones; probes/t3666-review-followups A1:A5).
    fn calendar_key(self) -> CalendarKey {
        match self {
            Source::Lcid(lcid) => CalendarKey::Lcid(lcid),
            Source::Tag { lcid, .. } => lcid
                .into_iter()
                .flat_map(|lcid| [lcid, primary_lcid(lcid)])
                .find(|lcid| lcid_names(*lcid).is_some())
                .map_or(CalendarKey::Default, CalendarKey::Lcid),
        }
    }

    /// Whether the locale is Japanese: its own calendar is the Japanese eras.
    fn japanese(self) -> bool {
        match self {
            Source::Lcid(lcid)
            | Source::Tag {
                lcid: Some(lcid), ..
            } => lcid & LANGUAGE_MASK == LANGUAGE_JAPANESE,
            Source::Tag { lcid: None, .. } => false,
        }
    }
}

/// What a calendar digit (Windows' calendar id) does: its date system and its effect after a `B2`
/// prefix. Its names are data (`excel_date_names::calendar_names`). Evidence:
/// probes/t1923-calendar-digits, text-locale-calendars, t2729-hijri-locale-tag, t2779-hebrew-saka-*,
/// t2932-b2-calendar-twice, t3101-b2-calendar-names, t3666-calendar-top-bit, t4061-*;
/// number-formats.md "Calendars of a locale tag".
#[derive(Clone, Copy)]
pub(super) struct CalendarSpec {
    calendar: Calendar,
    /// After a `B2`, every year token converts once more ([`Localized::year_twice`]).
    b2_twice: bool,
    /// After a `B2`, the LCID whose names every name token prints from the first `mmm` on
    /// ([`Localized::names_after_mmm`]).
    b2_names_after_mmm: Option<u16>,
}

const fn spec(calendar: Calendar) -> CalendarSpec {
    CalendarSpec {
        calendar,
        b2_twice: false,
        b2_names_after_mmm: None,
    }
}

/// The spec of calendar digit `digit` (Windows' CAL_* ids), `None` for the locale's own calendar
/// (digit 0, and 0x18-0x7F, which Excel ignores). The Gregorian digits differ only by their names
/// (`excel_date_names::calendar_names`): 1 Gregorian, 2 en-US, 9 Middle-East French, A-C Arabic
/// and transliterated, D, 14 (Korean month names) and 15.
const fn calendar_spec(digit: u32) -> Option<CalendarSpec> {
    Some(match digit {
        // `B2[$-10411]mmm|mmmm` prints the ar-SA names from the first `mmm` (t3101).
        1 => CalendarSpec {
            b2_names_after_mmm: Some(LCID_AR_SA),
            ..spec(Calendar::Gregorian)
        },
        2 | 9..=0xD | 0x14 | 0x15 => spec(Calendar::Gregorian),
        3 => CalendarSpec {
            b2_twice: true,
            b2_names_after_mmm: Some(LCID_EN_US),
            ..spec(Calendar::Japanese { explicit: true })
        },
        4 => CalendarSpec {
            b2_twice: true,
            ..spec(Calendar::Taiwan)
        },
        5 => CalendarSpec {
            b2_twice: true,
            b2_names_after_mmm: Some(LCID_EN_US),
            ..spec(Calendar::Korean)
        },
        6 => spec(Calendar::Hijri),
        7 => CalendarSpec {
            b2_twice: true,
            ..spec(Calendar::Thai)
        },
        8 => spec(Calendar::Hebrew),
        0xE | 0x12 | 0x13 => spec(Calendar::Lunar(LunarTable::Eastern)),
        0xF | 0x11 => spec(Calendar::Lunar(LunarTable::Chinese)),
        0x10 => spec(Calendar::Saka),
        0x16 => spec(Calendar::Persian),
        0x17 => spec(Calendar::UmAlQura),
        _ => return None,
    })
}

/// The LCID of ar-SA, whose names a `B2` before calendar 1 prints from the first `mmm`.
const LCID_AR_SA: u16 = 0x401;
/// The LCID of en-US, whose names a `B2` before calendars 3 and 5 prints from the first `mmm`.
const LCID_EN_US: u16 = 0x409;
/// The primary-language bits of an LCID.
const LANGUAGE_MASK: u16 = 0x3FF;
/// The primary language of Japanese.
const LANGUAGE_JAPANESE: u16 = 0x11;

/// The primary LCID of `lcid`'s language (sublanguage 1: `ar`, LCID 1, has 0x401).
const fn primary_lcid(lcid: u16) -> u16 {
    0x400 | (lcid & LANGUAGE_MASK)
}

/// `value` (an LCID with its calendar byte) under `source`: the system tags, or the locale's own
/// names and calendar under digit 0, or the calendar of the digit with the names it has for the
/// LCID ([`calendar_names`]). Day halves always follow the locale, and an LCID with no locale keeps
/// `A/P` as written (`[$-E08DE]h A/P` is "6 P"; probes/t1924-locale-tag-extended-hex B13). After a
/// `B2` (`hijri_prefix`) digit 0 is read as 6. The numeral byte (bits 24-31) is [`numerals`]'s
/// (`[$-1000411]e|mmmm` is "5|3月"; probes/t1925-numeral-digits B2).
fn localize(source: Source, value: u32, hijri_prefix: bool) -> (LocaleTag, Option<CalendarSpec>) {
    match value & 0xFF_FFFF {
        SYSTEM_DATE_LCID => return (LocaleTag::SystemDate, None),
        SYSTEM_TIME_LCID => return (LocaleTag::SystemTime, None),
        _ => {}
    }
    let own = source.own_names();
    let japanese = source.japanese();
    let digit = match calendar_digit(value) {
        0 if hijri_prefix => 6,
        digit => digit,
    };
    let Some(spec) = calendar_spec(digit) else {
        // The locale's own calendar: Japanese eras for a Japanese LCID (`[$-411]g` is "R").
        let calendar = if japanese {
            Calendar::Japanese { explicit: false }
        } else {
            Calendar::Gregorian
        };
        // With no locale the names are en-US, the calendar Gregorian (`[$-D001]b` is "23", where a
        // section with no tag prints the Buddhist year) and `A/P` stays as written
        // (probes/t4061-calendars-without-locale A1).
        let names = own.unwrap_or(EN_US);
        let tag = LocaleTag::Names(Localized::new(names, own, calendar, japanese));
        return (tag, None);
    };
    let Some(names) = calendar_names(digit, source.calendar_key()) else {
        return (LocaleTag::Silent, None);
    };
    let (names, leap_months, no_locale) = match names {
        CalendarNames::Own => (own.unwrap_or(EN_US), None, false),
        CalendarNames::Set {
            names,
            leap_months,
            no_locale,
        } => (names, leap_months, no_locale),
    };
    let localized = Localized {
        months_by_number: no_locale,
        leap_months: leap_months.filter(|_| !no_locale),
        ..Localized::new(names, own, spec.calendar, japanese)
    };
    (LocaleTag::Names(localized), Some(spec))
}

/// The numeral system of the text between `[$-` and `]`: byte 24-31 of the value of a hex tag, or
/// of an extended-hex one ([`extended_hex`]) that is no BCP-47 tag (`[$-fr_FR-x]` is 0x10E00AF1,
/// Myanmar digits; probes/t1924-locale-tag-extended-hex A26). It holds whatever the LCID resolves
/// to (`[$-fr_FR-x]` keeps the workbook's names and `A/P` as written).
pub fn numerals(tag: &str) -> Numerals {
    let value = hex_value(tag).or_else(|| match known_tag(tag) {
        None => extended_hex(tag),
        Some(_) => None,
    });
    value.map_or(Numerals::Ascii, |value| Numerals::from_byte(value >> 24))
}

/// `tag` read as a hex LCID with its calendar digits, if it is one.
fn hex_value(tag: &str) -> Option<u32> {
    u32::from_str_radix(tag, 16)
        .ok()
        .filter(|_| tag.len() <= 8 && tag.chars().all(|c| c.is_ascii_hexdigit()))
}

/// The value Excel reads from a tag that is neither hex nor BCP-47, as if it were hex with every
/// character a digit: a letter counts from 10 in either case (`g` 16, `r` 27, `z` 35), any other
/// character below `a` counts from `0` (`_` 47, `-` -3), and one above `z` from `a` (`{` 36, `~`
/// 39).
/// So `[$-fr_FR]` is 0x10E00B, the Saka calendar over an LCID with no locale, `[$-g]` is Italian
/// (0x10), `[$-3gC]` French (0x40C), `[$-0_]` Macedonian (0x2F) and `[$-~]` Lithuanian (0x27)
/// (probes/t1924-locale-tag-extended-hex, text-locale-calendars A135-A145). Of the other characters
/// below `a` only `_` and `-` (-3) are probed. The value wraps at 32 bits, so a longer tag reads
/// too: `[$-rhg-Rohg-BD]` is 0xEC91FDBD, the Chinese lunar calendar (0x91, 0x11 by [`calendar_digit`])
/// over an LCID with no locale, and `[$-tok-001]` 0x1E93D001 (probes/t3666-locale-tag-names A842,
/// A1007). Non-ASCII tags are unprobed and resolve to nothing.
fn extended_hex(tag: &str) -> Option<u32> {
    if tag.is_empty() || !tag.is_ascii() {
        return None;
    }
    Some(tag.bytes().fold(0u32, |value, c| {
        let digit = match c {
            b'a'.. => i32::from(c) - i32::from(b'a') + 10,
            b'A'..=b'Z' => i32::from(c) - i32::from(b'A') + 10,
            _ => i32::from(c) - i32::from(b'0'),
        };
        value.wrapping_mul(16).wrapping_add_signed(digit)
    }))
}

/// The calendar of a tag value: its third byte without the top bit, so 0x83 is 3 (Japanese eras;
/// `[$-myv-RU]`, 0x0183EECE, prints 2023-03-15 as "05-03-15") and 0x93 is 0x13 (lunar;
/// `[$-tok-001]`) and 0xFF is 0x7F, ignored (`[$-FF040C]` is French in its own calendar). Every tag of
/// probes/t3666-locale-tag-names with no locale fits this, over 70 distinct bytes, and so do the
/// LCIDs with a locale of probes/t3666-calendar-top-bit (`[$-83040C]` is Japanese eras,
/// `[$-91040C]` lunar, `[$-95040C]` en-US names like 0x15).
fn calendar_digit(value: u32) -> u32 {
    (value >> 16) & 0x7F
}

/// The Japanese eras Excel uses from 1900 on, latest first: the serial of the first day (1900 date
/// system), the first Gregorian year and what `g`, `gg` and `ggg` print. Excel: Taisho starts at
/// serial 4595 (1912-07-30), Showa at 9856, Heisei at 32516 and Reiwa at 43586 (2019-05-01); the
/// year before is the last of the previous era (`[$-411]g|e` of 43585 is "H|31";
/// probes/text-locale-calendars A1-A14, text-locale-calendars-detail A10, A41).
static ERAS: [(i64, i32, [&str; 3]); 5] = [
    (43586, 2019, ["R", "令", "令和"]),
    (32516, 1989, ["H", "平", "平成"]),
    (9856, 1926, ["S", "昭", "昭和"]),
    (4595, 1912, ["T", "大", "大正"]),
    (i64::MIN, 1868, ["M", "明", "明治"]),
];

/// The Japanese era of 1900-system `serial`: its first Gregorian year and its names.
fn era_of(serial: i64) -> (i32, &'static [&'static str; 3]) {
    let (_, first_year, names) = ERAS
        .iter()
        .find(|(first_serial, _, _)| serial >= *first_serial)
        .unwrap_or(&ERAS[ERAS.len() - 1]);
    (*first_year, names)
}

/// The Japanese era of 1900-system `serial`, whose Gregorian date is `(year, month, day)`: the
/// era year and what `g`, `gg` and `ggg` print. With `twice` (a `B2` before a calendar-3 tag)
/// Excel converts once more, reading the era year as the Gregorian year of a date with the same
/// month and day: Reiwa 5 (2023-03-15) is year 5, before Meiji, so Meiji 5 - 1868 + 1 = -1862,
/// which Excel prints as an unsigned 32-bit number, "4294965434|M"; 9999-12-31, Reiwa 7981, is
/// Reiwa 5963, and the era changes on its own day of the year read back: 3930-07-29 (Reiwa 1912)
/// is Meiji 45, 3930-07-30 Taisho 1 (probes/t2932-b2-calendar-twice A1-A16, A69-A80).
pub fn japanese_era(
    serial: i64,
    (year, month, day): (i32, u32, u32),
    twice: bool,
) -> (i32, &'static [&'static str; 3]) {
    let (first_year, names) = era_of(serial);
    let era_year = year - first_year + 1;
    if !twice {
        return (era_year, names);
    }
    // Day 0 (serial 0, "1900-01-00") and a 29 February of a common year have no date: the
    // month's 28th stands in for them, which no era boundary separates.
    let again = NaiveDate::from_ymd_opt(era_year, month, day)
        .or_else(|| NaiveDate::from_ymd_opt(era_year, month, 28))
        .map_or(i64::MIN, |date| i64::from(convert_to_serial_number(date)));
    let (first_year, names) = era_of(again);
    (era_year - first_year + 1, names)
}

#[cfg(test)]
mod tests {
    use super::extended_hex;

    /// Excel's extended-hex reading of tags (probes/t1924-locale-tag-extended-hex,
    /// t3666-locale-tag-names).
    const EXTENDED_HEX: [(&str, u32); 6] = [
        ("fr_FR", 0x0010_E00B),
        ("fr_FR-x", 0x10E0_0AF1),
        ("bgc", 0x0C0C),
        ("ceb", 0x0CEB),
        ("tok-001", 0x1E93_D001),
        ("rhg-Rohg-BD", 0xEC91_FDBD),
    ];

    #[test]
    fn extended_hex_golden() {
        for (tag, value) in EXTENDED_HEX {
            assert_eq!(extended_hex(tag), Some(value), "{tag}");
        }
    }

    /// A table tag Excel reads as extended hex is not a tag (`[$-bgc]` is fr-CA by 0xC0C, and
    /// `[$-bgc-IN]` falls back to no language), one that spells its LCID is (`[$-ceb]`).
    #[test]
    fn tags_read_as_hex() {
        assert!(super::known_tag("bgc").is_none());
        assert!(super::known_tag("bgc-IN").is_none());
        assert!(super::known_tag("ceb").is_some());
        assert!(super::known_tag("fr-XX").is_some());
    }
}
