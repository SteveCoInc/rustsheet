//! The names of the calendars a `[$-...]` locale tag's calendar digit selects (Hijri, lunar,
//! Levant Arabic, Saka, Hebrew): a calendar's own month (and some weekday) names over the names of
//! the locale Excel takes the rest from (`crate::formatter::locale_tag`). Every string is Excel's,
//! from probes/t1923-calendar-digits, text-locale-lunar-edges, text-locale-lunar-tokens,
//! t1613-hijri-b2, t2729-hijri-locale-tag(-names), t2779-hebrew-saka-names and
//! t2779-hebrew-saka-more (excel-graph); number-formats.md "Calendars of a locale tag".

use std::sync::OnceLock;

use super::excel_date_names::{lcid_names, DateNames, EN_US};

/// The calendar name sets, each a calendar's names over a base locale's.
pub(crate) struct CalendarNames {
    /// Calendar digits B and C under any LCID: Gregorian with the ar-SA names (`[$-B0409]mmmm|dddd`
    /// of 2023-03-15 is "مارس|الأربعاء"; probes/t1923-calendar-digits A22:A63, T-1923).
    pub arabic: &'static DateNames,
    /// Calendar digit A under any LCID: Gregorian with the Levant Arabic month names, `mmm` too, and
    /// the ar-SA weekdays (`[$-A0409]mmmm` of 2023-01-15 is "كانون الثاني", of 2023-03-15 "آذار";
    /// probes/t1923-calendar-digits A1:A21). The two-word names join with a no-break space.
    pub arabic_levant: DateNames,
    /// Calendar digits F and 11 under any LCID: lunar month names and zh-CN weekdays
    /// (`[$-F0412]mmmm|ddd` is "二月|周三"; probes/text-locale-lunar-edges A33-A44).
    pub lunar_zh: DateNames,
    /// Calendar digit E under a non-Korean LCID: lunar month names and Japanese weekdays
    /// (`[$-E0804]mmmm|ddd` is "二月|水"; probes/text-locale-lunar-edges A30-A33).
    pub lunar_ja: DateNames,
    /// Calendar digit E under a Korean LCID: the Korean lunar month names, `mmm` too, and Korean
    /// weekdays (`[$-E0412]mmmm|dddd` of 2023-01-22 is "정월|일요일"; probes/text-locale-lunar-tokens
    /// A59-A74). Calendar digit 14 gives the same names for the Gregorian months under any LCID
    /// (`[$-140409]mmmm|ddd` of 2023-03-15 is "삼월|수"; probes/t1923-calendar-digits A64:A84).
    pub lunar_ko: DateNames,
    /// The Hijri month names of an Arabic LCID and of an LCID with no locale under calendar digit 6,
    /// and of every section that a `B2` prefix puts in Excel's Arabic path (`B2mmmm`,
    /// `B2[$-409]mmm`): Muharram first, `mmm` the same, the ar-SA weekdays. Two-word names join
    /// with a no-break space (probes/t1613-hijri-b2, t2729-hijri-locale-tag A25-A36).
    pub hijri_arabic: DateNames,
    /// The Hijri names of every probed LCID but the Arabic, French and Turkish ones under calendar
    /// digit 6 or a `B2` prefix before the tag: English transliterations, `mmm` = `mmmm` and `ddd` =
    /// `dddd`, Muharram and Sunday first (`[$-60409]mmmm|dddd` of 2023-03-15 is
    /// "Sha'ban|AlArbia'a"; probes/t2729-hijri-locale-tag A1-A12, A73-A79,
    /// t2729-hijri-locale-tag-names A39-A82).
    pub hijri_english: DateNames,
    /// The Hijri names of a French LCID: French transliterated months (`mmm` the same) and the
    /// ordinary French weekdays (`[$-6040C]mmmm|ddd` of 2023-03-15 is "Chaaban|mer.";
    /// probes/t2729-hijri-locale-tag A13-A24, A80-A86).
    pub hijri_french: DateNames,
    /// The Hijri names of a Turkish LCID: Turkish months (`mmm` the same) and the ordinary Turkish
    /// weekdays (`[$-6041F]mmmm|ddd` of 2023-03-15 is "Şaban|Çar";
    /// probes/t2729-hijri-locale-tag-names A1-A19).
    pub hijri_turkish: DateNames,
    /// The Saka names of every LCID with a locale but an Arabic or Hindi one: Chaitra first (`mmm`
    /// the same), short Sanskrit weekdays for `ddd` and long ones for `dddd`, Sunday first
    /// (`[$-100409]mmmm|ddd|dddd` of 2023-03-15 is "Phalguna|Budh|Budhavara").
    pub saka: DateNames,
    /// The Saka names of an Arabic or Hindi LCID (`[$-100401]`, `[$-100439]`): Devanagari months,
    /// and the hi-IN `dddd` for both `ddd` and `dddd` (`[$-100439]mmmm|ddd` of 2023-03-15 is
    /// "फल्गुण|बुधवार"; probes/t2779-hebrew-saka-more B1:B23, C32, C33).
    pub saka_hindi: DateNames,
    /// The Hebrew names of every LCID with a locale but a Hebrew or Arabic one: the months of a
    /// common year from Tishrei (a leap year's Adars are [`HEBREW_LATIN_ADARS`]), and `ddd` the same
    /// as `dddd` (`[$-80409]mmmm|ddd` of 2023-03-15 is "Adar|Yom Revi'i";
    /// probes/t2779-hebrew-saka-names B1:B21).
    pub hebrew_latin: DateNames,
    /// The Hebrew names of a Hebrew or Arabic LCID (`[$-8040D]`, `[$-80401]`): Hebrew-script months,
    /// `ddd` the weekday's letter and `dddd` the he-IL name (`[$-8040D]mmmm|ddd|dddd` of 2023-03-15
    /// is "אדר|ד|יום רביעי"; probes/t2779-hebrew-saka-names C1:D21, t1923-calendar-digits A211).
    pub hebrew_script: DateNames,
}

/// The calendar name sets, built once from the locale tables.
pub(crate) fn calendar_names() -> &'static CalendarNames {
    static NAMES: OnceLock<CalendarNames> = OnceLock::new();
    NAMES.get_or_init(|| {
        let base = |lcid| lcid_names(lcid).unwrap_or(EN_US);
        let (arabic, english, hindi) = (base(0x401), EN_US, base(0x439));
        CalendarNames {
            arabic,
            arabic_levant: arabic.with_months(LEVANT_MONTHS),
            lunar_zh: base(0x804).with_months(LUNAR_MONTHS),
            lunar_ja: base(0x411).with_months(LUNAR_MONTHS),
            lunar_ko: base(0x412).with_months(LUNAR_KO_MONTHS),
            hijri_arabic: arabic.with_months(HIJRI_ARABIC_MONTHS),
            hijri_english: DateNames {
                days_short: HIJRI_ENGLISH_DAYS,
                days: HIJRI_ENGLISH_DAYS,
                ..english.with_months(HIJRI_ENGLISH_MONTHS)
            },
            hijri_french: base(0x40C).with_months(HIJRI_FRENCH_MONTHS),
            hijri_turkish: base(0x41F).with_months(HIJRI_TURKISH_MONTHS),
            saka: DateNames {
                days_short: ["Ravi", "Som", "Mangal", "Budh", "Guru", "Sukra", "Sani"],
                days: SAKA_DAYS,
                ..english.with_months(SAKA_MONTHS)
            },
            saka_hindi: DateNames {
                days_short: hindi.days,
                ..hindi.with_months(SAKA_HINDI_MONTHS)
            },
            hebrew_latin: DateNames {
                days_short: HEBREW_LATIN_DAYS,
                days: HEBREW_LATIN_DAYS,
                ..english.with_months(HEBREW_LATIN_MONTHS)
            },
            hebrew_script: DateNames {
                days_short: ["א", "ב", "ג", "ד", "ה", "ו", "ש"],
                ..base(0x40D).with_months(HEBREW_SCRIPT_MONTHS)
            },
        }
    })
}

const SAKA_DAYS: [&str; 7] = [
    "Ravivara",
    "Somavara",
    "Mangalvara",
    "Budhavara",
    "Guruvara",
    "Sukravara",
    "Sanivara",
];

const LEVANT_MONTHS: [&str; 12] = [
    "كانون\u{a0}الثاني",
    "شباط",
    "آذار",
    "نيسان",
    "أيار",
    "حزيران",
    "تموز",
    "آب",
    "أيلول",
    "تشرين\u{a0}الأول",
    "تشرين\u{a0}الثاني",
    "كانون\u{a0}الأول",
];

/// The Chinese lunar month names of calendar digits E, F and 11, first month first; `mmm` prints
/// them too and months 13 and 14 print the twelfth (`[$-110409]mmm|mmmm` of 2023-03-15 is
/// "二月|二月", of 2024-01-11 "十二月|十二月"; probes/text-locale-lunar-edges, T-1922).
const LUNAR_MONTHS: [&str; 12] = [
    "正月",
    "二月",
    "三月",
    "四月",
    "五月",
    "六月",
    "七月",
    "八月",
    "九月",
    "十月",
    "十一月",
    "十二月",
];

const LUNAR_KO_MONTHS: [&str; 12] = [
    "정월",
    "이월",
    "삼월",
    "사월",
    "오월",
    "유월",
    "칠월",
    "팔월",
    "구월",
    "시월",
    "동짓달",
    "섣달",
];

const HIJRI_ARABIC_MONTHS: [&str; 12] = [
    "محرم",
    "صفر",
    "ربيع\u{a0}الأول",
    "ربيع\u{a0}الثاني",
    "جمادى\u{a0}الأولى",
    "جمادى\u{a0}الثانية",
    "رجب",
    "شعبان",
    "رمضان",
    "شوال",
    "ذو\u{a0}القعدة",
    "ذو\u{a0}الحجة",
];

const HIJRI_ENGLISH_MONTHS: [&str; 12] = [
    "Muharram",
    "Safar",
    "Rabi' Awwal",
    "Rabi' Thani",
    "Jamada El Oula",
    "Jamada El Thaniah",
    "Rajab",
    "Sha'ban",
    "Ramadan",
    "Shawwal",
    "Thoul Ki'dah",
    "Thoul Hijjah",
];

const HIJRI_ENGLISH_DAYS: [&str; 7] = [
    "AlAhad",
    "AlEthnien",
    "AthThulatha",
    "AlArbia'a",
    "AlKhamis",
    "AlJumaa",
    "AsSabt",
];

const HIJRI_FRENCH_MONTHS: [&str; 12] = [
    "Mouharam",
    "Safar",
    "Rabi' Aoual",
    "Rabi' Thani",
    "Jamada El Oula",
    "Jamada El Thaniah",
    "Rajab",
    "Chaaban",
    "Ramadan",
    "Chaoual",
    "Zoul Ki'dah",
    "Zoul Hijjah",
];

const HIJRI_TURKISH_MONTHS: [&str; 12] = [
    "Muharrem",
    "Safer",
    "Rebiyülevvel",
    "Rebiyülahir",
    "Cemaziyelevvel",
    "Cemaziyelahir",
    "Recep",
    "Şaban",
    "Ramazan",
    "Şevval",
    "Zilkade",
    "Zilhicce",
];

const SAKA_MONTHS: [&str; 12] = [
    "Chaitra",
    "Vaisakha",
    "Jyaistha",
    "Asadha",
    "Sravana",
    "Bhadra",
    "Asvina",
    "Karthika",
    "Agrahayana",
    "Pausa",
    "Magha",
    "Phalguna",
];

/// Excel's months 4 and 8 hold the precomposed U+095D and U+0958, which are not NFC, so they are
/// escaped.
const SAKA_HINDI_MONTHS: [&str; 12] = [
    "चैत्र",
    "वैशाख",
    "ज्येष्ट",
    "आषा\u{95d}",
    "श्रावण",
    "भाद्र",
    "आश्वीन",
    "\u{958}ार्तीक",
    "अग्रहायन",
    "पौसा",
    "माघ",
    "फल्गुण",
];

const HEBREW_LATIN_MONTHS: [&str; 12] = [
    "Tishrei", "Cheshvan", "Kislev", "Tevet", "Shvat", "Adar", "Nisan", "Iyar", "Sivan", "Tamuz",
    "Av", "Elul",
];

const HEBREW_LATIN_DAYS: [&str; 7] = [
    "Yom Rishon",
    "Yom Sheni",
    "Yom Shlishi",
    "Yom Revi'i",
    "Yom Chamishi",
    "Yom Shishi",
    "Shabat",
];

/// Months 6 and 7 of a Hebrew leap year under [`CalendarNames::hebrew_latin`].
pub(crate) static HEBREW_LATIN_ADARS: [&str; 2] = ["AdarI", "AdarII"];

const HEBREW_SCRIPT_MONTHS: [&str; 12] = [
    "תשרי", "חשון", "כסלו", "טבת", "שבט", "אדר", "ניסן", "אייר", "סיון", "תמוז", "אב", "אלול",
];

/// Months 6 and 7 of a Hebrew leap year under [`CalendarNames::hebrew_script`], with a plain space
/// (probes/t1923-calendar-digits A212).
pub(crate) static HEBREW_SCRIPT_ADARS: [&str; 2] = ["אדר א", "אדר ב"];
