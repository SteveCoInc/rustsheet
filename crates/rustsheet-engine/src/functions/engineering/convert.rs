use std::collections::HashMap;

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

/// Excel's decimal prefixes ("da" and its alias "e" are deka).
const DECIMAL_PREFIXES: [(&str, f64); 21] = [
    ("Y", 1E+24),
    ("Z", 1E+21),
    ("E", 1E+18),
    ("P", 1E+15),
    ("T", 1E+12),
    ("G", 1E+9),
    ("M", 1E+6),
    ("k", 1E+3),
    ("h", 1E+2),
    ("da", 1E+1),
    ("e", 1E+1),
    ("d", 1E-1),
    ("c", 1E-2),
    ("m", 1E-3),
    ("u", 1E-6),
    ("n", 1E-9),
    ("p", 1E-12),
    ("f", 1E-15),
    ("a", 1E-18),
    ("z", 1E-21),
    ("y", 1E-24),
];

/// Excel's binary prefixes, `2^10` to `2^80`.
const BINARY_PREFIXES: [(&str, f64); 8] = [
    ("ki", 1024.0),
    ("Mi", 1_048_576.0),
    ("Gi", 1_073_741_824.0),
    ("Ti", 1_099_511_627_776.0),
    ("Pi", 1_125_899_906_842_624.0),
    ("Ei", 1_152_921_504_606_846_976.0),
    ("Zi", 1_180_591_620_717_411_303_424.0),
    ("Yi", 1_208_925_819_614_629_174_706_176.0),
];

// Excel: these units, and only these, take a decimal prefix; every other unit is `#N/A` with any
// prefix (probes/convert-unit-names, every unit with k, m, da, e and M, and the 21 decimal
// prefixes on fifteen of them; docs/excel-semantics/engineering.md "CONVERT unit names").
const DECIMAL_PREFIXED: [&str; 52] = [
    "Pa", "p", "atm", "at", "mmHg", "g", "u", "m", "ang", "ly", "parsec", "pc", "ang2", "ang^2",
    "ar", "m2", "m^2", "N", "dyn", "dy", "pond", "J", "e", "c", "cal", "eV", "ev", "Wh", "wh", "W",
    "w", "T", "ga", "uk_pt", "l", "L", "lt", "ang3", "ang^3", "m3", "m^3", "bit", "byte", "m/h",
    "m/hr", "m/s", "m/sec", "mph", "K", "kel", "sec", "s",
];

// Excel: only bit and byte take a binary prefix (`kibyte`, `Gibit`); `kim`, `Mig`, `kiJ`, `kiK`
// are `#N/A` (probes/convert-unit-names).
const BINARY_PREFIXED: [&str; 2] = ["bit", "byte"];

/// A unit string read within one class: the unit's name, its factor in the class's internal
/// base and the prefix scale (already raised to the power for square and cubic units).
struct ParsedUnit {
    name: &'static str,
    factor: f64,
    scale: f64,
}

/// Reads `text` as a unit of the class `kind`: the unit itself, or one prefix the unit takes
/// followed by the unit (case-sensitive, nothing around it). `None` if neither reads.
fn parse_unit(kind: &str, class: &HashMap<&'static str, f64>, text: &str) -> Option<ParsedUnit> {
    if let Some((&name, &factor)) = class.get_key_value(text) {
        return Some(ParsedUnit {
            name,
            factor,
            scale: 1.0,
        });
    }
    // Two unit names of a class could both end `text`: try the longest name first, so the reading
    // does not depend on the map's iteration order.
    let mut names: Vec<_> = class.iter().collect();
    names.sort_unstable_by_key(|(name, _)| (std::cmp::Reverse(name.len()), **name));
    names.into_iter().find_map(|(&name, &factor)| {
        let prefix = text.strip_suffix(name)?;
        let lookup =
            |table: &[(&str, f64)]| table.iter().find_map(|&(p, m)| (p == prefix).then_some(m));
        let modifier = if DECIMAL_PREFIXED.contains(&name) {
            lookup(&DECIMAL_PREFIXES)
        } else {
            None
        }
        .or_else(|| {
            if BINARY_PREFIXED.contains(&name) {
                lookup(&BINARY_PREFIXES)
            } else {
                None
            }
        })?;
        // A prefix on a square unit scales its length, so it is squared ("km2" is 1E6 m2; "ar" is
        // not a square of a length and takes it once); likewise cubed for "ang3" and "m3".
        let scale = if kind == "area" && name != "ar" {
            modifier * modifier
        } else if kind == "volume" && ["ang3", "ang^3", "m3", "m^3"].contains(&name) {
            modifier * modifier * modifier
        } else {
            modifier
        };
        Some(ParsedUnit {
            name,
            factor,
            scale,
        })
    })
}

enum Temperature {
    Kelvin,
    Celsius,
    Rankine,
    Reaumur,
    Fahrenheit,
}

// Excel: every pair has its own last-digit form; no single pivot reproduces them (probes/convert-
// targets and probes/convert-temperature-sweep, excel-graph; docs/excel-semantics/engineering.md
// "CONVERT temperature"). Rankine and Fahrenheit are linked by `459.67`, the Rankine/Kelvin ratio
// is `1.8` (a multiply or divide, never `9/5`), and F to Reaumur is `(v - 32) * 0.8 / 1.8`.
fn convert_temperature(
    value: f64,
    from_temperature: Temperature,
    to_temperature: Temperature,
) -> f64 {
    use Temperature::{Celsius, Fahrenheit, Kelvin, Rankine, Reaumur};
    match (from_temperature, to_temperature) {
        (Celsius, Celsius)
        | (Kelvin, Kelvin)
        | (Rankine, Rankine)
        | (Reaumur, Reaumur)
        | (Fahrenheit, Fahrenheit) => value,
        (Celsius, Fahrenheit) => value * 1.8 + 32.0,
        (Celsius, Kelvin) => value + 273.15,
        (Celsius, Rankine) => (value + 273.15) * 1.8,
        (Celsius, Reaumur) => value * 0.8,
        (Fahrenheit, Celsius) => (value - 32.0) / 1.8,
        (Fahrenheit, Kelvin) => (value - 32.0) / 1.8 + 273.15,
        (Fahrenheit, Rankine) => value + 459.67,
        (Fahrenheit, Reaumur) => (value - 32.0) * 0.8 / 1.8,
        (Kelvin, Celsius) => value - 273.15,
        (Kelvin, Fahrenheit) => (value - 273.15) * 1.8 + 32.0,
        (Kelvin, Rankine) => value * 1.8,
        (Kelvin, Reaumur) => (value - 273.15) * 0.8,
        (Rankine, Celsius) => value / 1.8 - 273.15,
        (Rankine, Fahrenheit) => value - 459.67,
        (Rankine, Kelvin) => value / 1.8,
        (Rankine, Reaumur) => (value / 1.8 - 273.15) * 0.8,
        (Reaumur, Celsius) => value * 1.25,
        (Reaumur, Fahrenheit) => value * 2.25 + 32.0,
        (Reaumur, Kelvin) => value * 1.25 + 273.15,
        (Reaumur, Rankine) => (value * 1.25 + 273.15) * 1.8,
    }
}

impl Model {
    /// `CONVERT(number, from_unit, to_unit)`: every argument lifts over an array and refuses a
    /// multi-cell range (`ArgKind::LiftArrays`, SPEC-0011; probes/eng-argument-modes B14:E14,
    /// probes/scalar-lift-open-a Y8-Y10).
    pub(crate) fn fn_convert(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Convert, args, cell, &mut |model, values| {
            model.convert_of(values, cell)
        })
    }

    /// CONVERT of one element: the number, then the two units, read in order.
    fn convert_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [value, from_unit, to_unit] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        // A boolean, typed or in a cell, is `#VALUE!` before the units are read
        // (probes/eng-gen-sample-edges A8, A11:A14).
        let value = match self.cast_number_no_bools_ref(value, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // A boolean unit read from a cell is `#VALUE!`, not an unknown unit
        // (gen-m9 290180acf017709c Calc!A148, A158, f81bd3b6dc373e5c Calc!A291).
        // Each unit is checked and read in turn, so from_unit's error comes first.
        let unit_text = |unit: &CalcResult| {
            if matches!(unit, CalcResult::Boolean(_)) {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Expecting text".to_string(),
                ));
            }
            Self::cast_string_ref(unit, cell)
        };
        let from_unit = match unit_text(from_unit) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let to_unit = match unit_text(to_unit) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let mut units = HashMap::new();

        let weight = HashMap::from([
            ("g", 1.0),
            ("sg", 14593.902937206363),
            ("lbm", 453.59237),
            ("u", 1.660538782E-24),
            ("ozm", 28.349523125),
            ("grain", 0.06479891),
            ("cwt", 45359.237),
            ("shweight", 45359.237),
            ("uk_cwt", 50802.34544),
            ("lcwt", 50802.34544),
            ("stone", 6350.29318),
            ("ton", 907184.74),
            ("brton", 1016046.9088), // g-sheets has a different value for this
            ("LTON", 1016046.9088),
            ("uk_ton", 1016046.9088),
        ]);

        units.insert("weight", weight);

        let distance = HashMap::from([
            ("m", 1.0),
            ("mi", 1609.344),
            ("Nmi", 1852.0),
            ("in", 0.0254),
            ("ft", 0.3048),
            ("yd", 0.9144),
            ("ang", 0.0000000001),
            ("ell", 1.143),
            ("ly", 9460730472580800.0),
            ("parsec", 3.0856775812815532e+16),
            ("pc", 3.0856775812815532e+16),
            ("Picapt", 0.0003527777777777778),
            ("Pica", 0.0003527777777777778),
            ("pica", 0.004233333333333334),
            ("survey_mi", 1609.3472186944373),
        ]);
        units.insert("distance", distance);
        let time = HashMap::from([
            ("yr", 31557600.0),
            ("day", 86400.0),
            ("d", 86400.0),
            ("hr", 3600.0),
            ("mn", 60.0),
            ("min", 60.0),
            ("sec", 1.0),
            ("s", 1.0),
        ]);

        units.insert("time", time);

        let pressure = HashMap::from([
            ("Pa", 1.0),
            ("p", 1.0),
            ("atm", 101325.0),
            ("at", 101325.0),
            ("mmHg", 133.322),
            ("psi", 6894.757293168362),
            ("Torr", 133.32236842105263),
        ]);
        units.insert("pressure", pressure);
        // Force factors are in dynes (N = 1e5): Excel's internal base, the only one that reproduces
        // every cached cell (docs/excel-semantics/engineering.md "CONVERT constants and evaluation
        // order"). Every factor here and below is the double fitted to CONVERT.xlsx (T-0922).
        let force = HashMap::from([
            ("N", 100000.0),
            ("dyn", 1.0),
            ("dy", 1.0),
            ("lbf", 444822.16152605),
            ("pond", 980.665),
        ]);
        units.insert("force", force);

        // Energy factors are in ergs (J = 1e7), Excel's internal base (see `force`).
        let energy = HashMap::from([
            ("J", 10000000.0),
            ("e", 1.0),
            ("c", 41840000.0),
            ("cal", 41868000.0),
            ("eV", 1.602176487e-12),
            ("ev", 1.602176487e-12),
            ("HPh", 26845195376961.73),
            ("hh", 26845195376961.73),
            ("Wh", 36000000000.0),
            ("wh", 36000000000.0),
            ("flb", 13558179.483314004),
            ("BTU", 10550558526.2),
            ("btu", 10550558526.2),
        ]);
        units.insert("energy", energy);

        let power = HashMap::from([
            ("HP", 745.6998715822702),
            ("h", 745.6998715822702),
            ("PS", 735.49875),
            ("W", 1.0),
            ("w", 1.0),
        ]);
        units.insert("power", power);

        let magnetism = HashMap::from([("T", 1.0), ("ga", 0.0001)]);
        units.insert("magnetism", magnetism);

        // Volume factors are in litres (m3 = 1000), Excel's internal base (see `force`).
        let volume = HashMap::from([
            ("tsp", 0.00492892159375),
            ("tspm", 0.005),
            ("tbs", 0.01478676478125),
            ("oz", 0.0295735295625),
            ("cup", 0.2365882365),
            ("pt", 0.473176473),
            ("us_pt", 0.473176473),
            ("uk_pt", 0.56826125),
            ("qt", 0.946352946),
            ("uk_qt", 1.1365225),
            ("gal", 3.785411784),
            ("uk_gal", 4.54609),
            ("l", 1.0),
            ("L", 1.0),
            ("lt", 1.0),
            ("ang3", 1e-27),
            ("ang^3", 1e-27),
            ("barrel", 158.987294928),
            ("bushel", 35.23907016688),
            ("ft3", 28.316846592),
            ("ft^3", 28.316846592),
            ("in3", 0.016387064),
            ("in^3", 0.016387064),
            ("ly3", 8.467866646237152e+50),
            ("ly^3", 8.467866646237152e+50),
            ("m3", 1000.0),
            ("m^3", 1000.0),
            ("mi3", 4168181825440.5796),
            ("mi^3", 4168181825440.5796),
            ("yd3", 764.554857984),
            ("yd^3", 764.554857984),
            ("Nmi3", 6352182208000.0),
            ("Nmi^3", 6352182208000.0),
            ("Picapt3", 4.39039566186557e-08),
            ("Picapt^3", 4.39039566186557e-08),
            ("Pica3", 4.39039566186557e-08),
            ("Pica^3", 4.39039566186557e-08),
            ("GRT", 2831.6846592),
            ("regton", 2831.6846592),
            ("MTON", 1132.67386368),
        ]);
        units.insert("volume", volume);

        // Area factors are in square angstroms (m2 = 1e20), Excel's internal base (see `force`).
        let area = HashMap::from([
            ("uk_acre", 4.0468564224e+23),
            ("us_acre", 4.046872609874252e+23),
            ("ang2", 1.0),
            ("ang^2", 1.0),
            ("ar", 1e+22),
            ("ft2", 9.290304e+18),
            ("ft^2", 9.290304e+18),
            ("ha", 1e+24),
            ("in2", 6.4516e+16),
            ("in^2", 6.4516e+16),
            ("ly2", 8.950542107481889e+51),
            ("ly^2", 8.950542107481889e+51),
            ("m2", 1e+20),
            ("m^2", 1e+20),
            ("Morgen", 2.5e+23),
            ("mi2", 2.589988110336e+26),
            ("mi^2", 2.589988110336e+26),
            ("Nmi2", 3.429904e+26),
            ("Nmi^2", 3.429904e+26),
            ("Picapt2", 12445216049382.715),
            ("Pica2", 12445216049382.715),
            ("Pica^2", 12445216049382.715),
            ("Picapt^2", 12445216049382.715),
            ("yd2", 8.3612736e+19),
            ("yd^2", 8.3612736e+19),
        ]);
        units.insert("area", area);

        let information = HashMap::from([("bit", 1.0), ("byte", 8.0)]);
        units.insert("information", information);

        let speed = HashMap::from([
            ("admkn", 0.5147733333333333),
            ("kn", 0.5144444444444445),
            ("m/h", 0.0002777777777777778),
            ("m/hr", 0.0002777777777777778),
            ("m/s", 1.0),
            ("m/sec", 1.0),
            ("mph", 0.44704),
        ]);
        units.insert("speed", speed);

        let temperature = HashMap::from([
            ("C", 1.0),
            ("cel", 1.0),
            ("F", 1.0),
            ("fah", 1.0),
            ("K", 1.0),
            ("kel", 1.0),
            ("Rank", 1.0),
            ("Reau", 1.0),
        ]);
        units.insert("temperature", temperature);

        // Excel reads each unit string within one class at a time and takes the class in which both
        // strings parse: "pc" is the parsec against "m" but pico-calories against "J" or "eV"
        // (probes/convert-unit-names; docs/excel-semantics/engineering.md "CONVERT unit names").
        // Within a class an exact unit name wins over a prefixed reading. No two classes give both
        // strings a reading with different values (only "pc" parses in two classes, and
        // "pc" to "pc" is the identity in either), so the class order only makes the scan
        // deterministic.
        let mut classes: Vec<_> = units.iter().collect();
        classes.sort_unstable_by_key(|(name, _)| **name);
        let Some((kind, from, to)) = classes.into_iter().find_map(|(&kind, class)| {
            Some((
                kind,
                parse_unit(kind, class, &from_unit)?,
                parse_unit(kind, class, &to_unit)?,
            ))
        }) else {
            return CalcResult::new_error(Error::NA, cell, String::new());
        };
        let ParsedUnit {
            name: from_unit_name,
            factor: from_unit_factor,
            scale: from_unit_scale,
        } = from;
        let ParsedUnit {
            name: to_unit_name,
            factor: to_unit_factor,
            scale: to_unit_scale,
        } = to;

        if kind == "temperature" {
            // Temperature requires formula conversion
            // Kelvin (K, k), Celsius (C,cel), Rankine (Rank), Réaumur (Reau)
            let to_temperature = match to_unit_name {
                "K" | "kel" => Temperature::Kelvin,
                "C" | "cel" => Temperature::Celsius,
                "Rank" => Temperature::Rankine,
                "Reau" => Temperature::Reaumur,
                "F" | "fah" => Temperature::Fahrenheit,
                _ => {
                    return CalcResult::new_error(Error::ERROR, cell, String::new());
                }
            };
            let from_temperature = match from_unit_name {
                "K" | "kel" => Temperature::Kelvin,
                "C" | "cel" => Temperature::Celsius,
                "Rank" => Temperature::Rankine,
                "Reau" => Temperature::Reaumur,
                "F" | "fah" => Temperature::Fahrenheit,
                _ => {
                    return CalcResult::new_error(Error::ERROR, cell, String::new());
                }
            };
            let t = convert_temperature(
                value * (from_unit_scale * from_unit_factor),
                from_temperature,
                to_temperature,
            ) / (to_unit_scale * to_unit_factor);
            return CalcResult::Number(t);
        }
        // Excel: the value is multiplied by the from-factor, divided by the to-factor, and only then
        // scaled by the prefixes, in that order (docs/excel-semantics/engineering.md "CONVERT
        // constants and evaluation order"; ironcalc-calc ENGINEERING/CONVERT.xlsx, e.g. Volume!J17
        // `CONVERT(0.0002,"Mang3","m3")` = 2.0000000000000002E-16).
        // Excel: `1E290 m2 -> m2`, `1E300 ha -> m2` and `1E302 J -> J` do not overflow although
        // `value * F_from` does (probes/convert-targets): when the product is infinite the
        // division goes first. The target prefix is applied as a reciprocal multiply
        // (`CONVERT(0.0002,"m3","Mm3")` is 2.0000000000000003E-22, not 2E-22).
        let product = value * from_unit_factor;
        let scaled = if product.is_infinite() {
            value / to_unit_factor * from_unit_factor
        } else {
            product / to_unit_factor
        };
        // Excel: `CONVERT(1E290,"Ym","Ym")` is 1E+290 and `CONVERT(1E300,"Ym","Zm")` 1E+303 although
        // `value * 1E24` overflows (probes/convert-temperature-ov00, -ov01, -ov20): the prefixes are
        // combined into one ratio when applying the from-prefix first would overflow. A result that
        // overflows even so stays `#NUM!` (ov02, ov21).
        let from_scaled = scaled * from_unit_scale;
        let result = if from_scaled.is_infinite() {
            scaled * (from_unit_scale * (1.0 / to_unit_scale))
        } else {
            from_scaled * (1.0 / to_unit_scale)
        };
        CalcResult::Number(result)
    }
}
