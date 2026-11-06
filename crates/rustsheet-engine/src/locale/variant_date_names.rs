//! Excel's date names for the Russian and French variants probed under T-4064 and T-4065
//! (probes/t4064-ru-md-names, t4065-french-regional-halves, t4065-french-regional-names;
//! excel-graph, 2026-11-06); the table list lives in [`super::excel_date_names`].

use super::excel_date_names::{DateNames, FR, RU};

/// Lowercase nominative months, the `mmmm` of ru-MD and of `[$-F419]`.
const RU_MONTHS_LOWER: [&str; 12] = [
    "январь",
    "февраль",
    "март",
    "апрель",
    "май",
    "июнь",
    "июль",
    "август",
    "сентябрь",
    "октябрь",
    "ноябрь",
    "декабрь",
];

/// Lowercase genitive months, the `mmmm` of `[$-FC19]`.
const RU_MONTHS_GENITIVE_LOWER: [&str; 12] = [
    "января",
    "февраля",
    "марта",
    "апреля",
    "мая",
    "июня",
    "июля",
    "августа",
    "сентября",
    "октября",
    "ноября",
    "декабря",
];

/// ru-MD (`[$-819]`, `[$-ru-MD]`): dotted short months, lowercase months and short weekdays, the
/// ru-RU long weekdays and day halves. Excel: probes/t4064-ru-md-names B1:B12, G1:G12.
pub(super) static RU_MD: DateNames = DateNames {
    months_short: [
        "янв.",
        "февр.",
        "март",
        "апр.",
        "май",
        "июнь",
        "июль",
        "авг.",
        "сент.",
        "окт.",
        "нояб.",
        "дек.",
    ],
    months: RU_MONTHS_LOWER,
    days_short: ["вс", "пн", "вт", "ср", "чт", "пт", "сб"],
    ..RU
};

/// `[$-F419]` (Russian, sublanguage 0x3D): ru-RU with lowercase nominative months.
/// Excel: probes/t4064-ru-md-names C1:C12.
pub(super) static RU_F419: DateNames = DateNames {
    months: RU_MONTHS_LOWER,
    ..RU
};

/// `[$-F819]` (Russian, sublanguage 0x3E): ru-RU with capitalised genitive months ("Января").
/// Excel: probes/t4064-ru-md-names E1:E12.
pub(super) static RU_F819: DateNames = DateNames {
    months: [
        "Января",
        "Февраля",
        "Марта",
        "Апреля",
        "Мая",
        "Июня",
        "Июля",
        "Августа",
        "Сентября",
        "Октября",
        "Ноября",
        "Декабря",
    ],
    ..RU
};

/// `[$-FC19]` (Russian, sublanguage 0x3F): ru-RU with lowercase genitive months ("января").
/// Excel: probes/t4064-ru-md-names D1:D12.
pub(super) static RU_FC19: DateNames = DateNames {
    months: RU_MONTHS_GENITIVE_LOWER,
    ..RU
};

/// fr-CM (`[$-2C0C]`): the fr-FR names with the day halves "mat." and "soir".
/// Excel: probes/t4065-french-regional-halves B1, C1, B17, C17; t4065-french-regional-names D1:D12.
pub(super) static FR_CM: DateNames = DateNames {
    am: "mat.",
    pm: "soir",
    ..FR
};

/// fr-MA (`[$-380C]`): fr-FR with its own short months ("jan.", "fév.", "mar.", "jui.").
/// Excel: probes/t4065-french-regional-names B1:B12.
pub(super) static FR_MA: DateNames = DateNames {
    months_short: [
        "jan.", "fév.", "mar.", "avr.", "mai", "jui.", "juil.", "août", "sept.", "oct.", "nov.",
        "déc.",
    ],
    ..FR
};

/// fr-029 (`[$-1C0C]`, French Caribbean): fr-FR with capitalised months, short and long.
/// Excel: probes/t4065-french-regional-names C1:C12.
pub(super) static FR_029: DateNames = DateNames {
    months_short: [
        "Janv.", "Févr.", "Mars", "Avr.", "Mai", "Juin", "Juil.", "Août", "Sept.", "Oct.", "Nov.",
        "Déc.",
    ],
    months: [
        "Janvier",
        "Février",
        "Mars",
        "Avril",
        "Mai",
        "Juin",
        "Juillet",
        "Août",
        "Septembre",
        "Octobre",
        "Novembre",
        "Décembre",
    ],
    ..FR
};
