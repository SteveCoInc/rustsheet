// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(elided_lifetimes_in_paths)]
// LINT-DEBT-END
use std::fmt;

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::language::Language;

use super::{lexer::LexerError, types::ParsedReference};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum OpCompare {
    LessThan,
    GreaterThan,
    Equal,
    LessOrEqualThan,
    GreaterOrEqualThan,
    NonEqual,
}

impl fmt::Display for OpCompare {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpCompare::LessThan => write!(fmt, "<"),
            OpCompare::GreaterThan => write!(fmt, ">"),
            OpCompare::Equal => write!(fmt, "="),
            OpCompare::LessOrEqualThan => write!(fmt, "<="),
            OpCompare::GreaterOrEqualThan => write!(fmt, ">="),
            OpCompare::NonEqual => write!(fmt, "<>"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OpUnary {
    Minus,
    Percentage,
}

impl fmt::Display for OpUnary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpUnary::Minus => write!(fmt, "-"),
            OpUnary::Percentage => write!(fmt, "%"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum OpSum {
    Add,
    Minus,
}

impl fmt::Display for OpSum {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpSum::Add => write!(fmt, "+"),
            OpSum::Minus => write!(fmt, "-"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum OpProduct {
    Times,
    Divide,
}

impl fmt::Display for OpProduct {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpProduct::Times => write!(fmt, "*"),
            OpProduct::Divide => write!(fmt, "/"),
        }
    }
}

/// List of `errors`
/// Note that "#ERROR!" and "#N/IMPL!" are not part of the xlsx standard
///  * "#ERROR!" means there was an error processing the formula (for instance "=A1+")
///  * "#N/IMPL!" means the formula or feature in Excel but has not been implemented in IronCalc
///    Note that they are serialized/deserialized by index
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Error {
    REF,
    NAME,
    VALUE,
    DIV,
    NA,
    NUM,
    ERROR,
    NIMPL,
    SPILL,
    CALC,
    CIRC,
    NULL,
    // Excel's newer error kinds (SPEC-0003). Appended after `NULL` so the existing variant
    // indices, and with them every serialized model, stay valid.
    /// `#GETTING_DATA` (ERROR.TYPE 8). Excel saves it to xlsx as `#N/A` (`stored_literal`).
    GETTINGDATA,
    /// `#CONNECT!` (ERROR.TYPE 10).
    CONNECT,
    /// `#BLOCKED!` (ERROR.TYPE 11).
    BLOCKED,
    /// `#FIELD!` (ERROR.TYPE 13).
    FIELD,
    /// `#BUSY!` (ERROR.TYPE 0). Only a stored constant: Excel loads `<c t="e"><v>#BUSY!</v></c>`
    /// but rejects `=#BUSY!`, and no formula is known to produce it (SPEC-0003, "Stored `#BUSY!`").
    BUSY,
}

/// The en-US formula literal for [`Error::GETTINGDATA`]. Of Excel's newer errors only this one
/// parses in a formula (probes/error-literals-new A1, A11; T-0175). It is a constant rather than a
/// `Language` field because `language.bin` is a bitcode blob (SPEC-0003).
const GETTING_DATA_LITERAL: &str = "#GETTING_DATA";

/// Whether `text` starts with the `#GETTING_DATA` literal, ignoring ASCII case
/// (`=#getting_data` parses in Excel; probes/error-literals-new A11).
fn starts_with_getting_data(text: &str) -> bool {
    text.get(..GETTING_DATA_LITERAL.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(GETTING_DATA_LITERAL))
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.english_literal())
    }
}
impl Error {
    /// Every kind, in declaration order; the first twelve are `error_index`'s order and the
    /// older kinds a `Language` names. The English name lookups below walk it, so
    /// [`Error::english_literal`] is the one en-US literal table.
    pub(crate) const ALL: [Error; 17] = [
        Error::REF,
        Error::NAME,
        Error::VALUE,
        Error::DIV,
        Error::NA,
        Error::NUM,
        Error::ERROR,
        Error::NIMPL,
        Error::SPILL,
        Error::CALC,
        Error::CIRC,
        Error::NULL,
        Error::GETTINGDATA,
        Error::CONNECT,
        Error::BLOCKED,
        Error::FIELD,
        Error::BUSY,
    ];

    /// The en-US literal of this error, as `Display` writes it and xlsx stores it: the one table
    /// that printing ([`fmt::Display`]), parsing ([`get_error_by_english_name`]) and criteria
    /// ([`english_criterion_error`]) share, and the `rustsheet` facade maps onto.
    pub fn english_literal(&self) -> &'static str {
        match self {
            Error::NULL => "#NULL!",
            Error::REF => "#REF!",
            Error::NAME => "#NAME?",
            Error::VALUE => "#VALUE!",
            Error::DIV => "#DIV/0!",
            Error::NA => "#N/A",
            Error::NUM => "#NUM!",
            Error::ERROR => "#ERROR!",
            Error::NIMPL => "#N/IMPL!",
            Error::SPILL => "#SPILL!",
            Error::CALC => "#CALC!",
            Error::CIRC => "#CIRC!",
            Error::GETTINGDATA => GETTING_DATA_LITERAL,
            Error::CONNECT => "#CONNECT!",
            Error::BLOCKED => "#BLOCKED!",
            Error::FIELD => "#FIELD!",
            Error::BUSY => "#BUSY!",
        }
    }

    /// The number `ERROR.TYPE` returns for this error. The single source for that table; the
    /// `rustsheet` facade's `ErrorKind::code` is built on it.
    pub fn error_type_code(&self) -> u8 {
        match self {
            Error::NULL => 1,
            Error::DIV => 2,
            Error::VALUE => 3,
            Error::REF => 4,
            Error::NAME => 5,
            Error::NUM => 6,
            Error::NA => 7,
            Error::SPILL => 9,
            Error::GETTINGDATA => 8,
            Error::CONNECT => 10,
            Error::BLOCKED => 11,
            Error::FIELD => 13,
            Error::CALC => 14,
            // A real code, not "no code": ERROR.TYPE of a stored #BUSY! is 0, and
            // ERROR.TYPE(A1)+100 is 100 (probes/error-stored-constants S!B1, H1).
            Error::BUSY => 0,
            // Excel's #UNKNOWN! (12) and #EXTERNAL! (19) have no kind here: there is no evidence
            // of a producer. The rest are RustSheet/IronCalc specific.
            Error::ERROR => 101,
            Error::NIMPL => 102,
            Error::CIRC => 104,
        }
    }

    pub fn to_localized_error_string(&self, language: &Language) -> String {
        match self {
            Error::NULL => language.errors.null.clone(),
            Error::REF => language.errors.r#ref.clone(),
            Error::NAME => language.errors.name.clone(),
            Error::VALUE => language.errors.value.clone(),
            Error::DIV => language.errors.div.clone(),
            Error::NA => language.errors.na.clone(),
            Error::NUM => language.errors.num.clone(),
            Error::ERROR => language.errors.error.clone(),
            Error::NIMPL => language.errors.nimpl.clone(),
            Error::SPILL => language.errors.spill.clone(),
            Error::CALC => language.errors.calc.clone(),
            Error::CIRC => language.errors.circ.clone(),
            // Not in `Language` (SPEC-0003): the en-US names in every language.
            Error::GETTINGDATA | Error::CONNECT | Error::BLOCKED | Error::FIELD | Error::BUSY => {
                self.to_string()
            }
        }
    }

    /// The literal Excel saves for this error in a workbook's cell value: `#N/A` for
    /// `#GETTING_DATA` (SPEC-0003, "Stored form"), the `Display` text otherwise. A storage rule
    /// only; in memory the value keeps its kind, so ERROR.TYPE stays 8 and ISNA FALSE.
    pub fn stored_literal(&self) -> &'static str {
        match self {
            Error::GETTINGDATA => Error::NA.english_literal(),
            Error::REF
            | Error::NAME
            | Error::VALUE
            | Error::DIV
            | Error::NA
            | Error::NUM
            | Error::ERROR
            | Error::NIMPL
            | Error::SPILL
            | Error::CALC
            | Error::CIRC
            | Error::NULL
            | Error::CONNECT
            | Error::BLOCKED
            | Error::FIELD
            | Error::BUSY => self.english_literal(),
        }
    }
}

/// The twelve errors a [`Language`] names, with their localized literals, in the order the lexer
/// tries them as prefixes.
fn localized_names(language: &Language) -> [(&str, Error); 12] {
    let errors = &language.errors;
    [
        (errors.r#ref.as_str(), Error::REF),
        (errors.name.as_str(), Error::NAME),
        (errors.value.as_str(), Error::VALUE),
        (errors.div.as_str(), Error::DIV),
        (errors.na.as_str(), Error::NA),
        (errors.num.as_str(), Error::NUM),
        (errors.error.as_str(), Error::ERROR),
        (errors.nimpl.as_str(), Error::NIMPL),
        (errors.spill.as_str(), Error::SPILL),
        (errors.calc.as_str(), Error::CALC),
        (errors.null.as_str(), Error::NULL),
        (errors.circ.as_str(), Error::CIRC),
    ]
}

/// Whether two error names are the same ignoring case. Excel reads an error name in any case,
/// typed as a constant, as a formula literal or as a COUNTIF criterion (`#n/a`, `#Div/0!`,
/// `#getting_data`; probes/t4852-error-name-case, docs/excel-semantics/syntax.md "Error names
/// are read in any case").
fn same_error_name(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b) || (!a.is_ascii() && a.to_uppercase() == b.to_uppercase())
}

/// The error a typed constant names in `language`, in any case (probes/t4852-error-name-case
/// A1:D14). Of Excel's newer errors only `#GETTING_DATA` is an input literal (SPEC-0003).
pub fn get_error_by_name(name: &str, language: &Language) -> Option<Error> {
    localized_names(language)
        .into_iter()
        .find(|(literal, _)| same_error_name(literal, name))
        .map(|(_, error)| error)
        .or_else(|| same_error_name(name, GETTING_DATA_LITERAL).then_some(Error::GETTINGDATA))
}

/// The error literal `text` starts with in a formula, read in any case (`=#n/a` is `#N/A`;
/// probes/t4852-error-name-case E1:G8), and its length in characters. `#BLOCKED!`,
/// `#CONNECT!` and `#FIELD!` stay unparseable, as Excel refuses them (T-0175).
pub(crate) fn formula_error_prefix(text: &str, language: &Language) -> Option<(Error, usize)> {
    localized_names(language)
        .into_iter()
        .find_map(|(literal, error)| {
            let len = literal.chars().count();
            let head: String = text.chars().take(len).collect();
            same_error_name(&head, literal).then_some((error, len))
        })
        .or_else(|| {
            starts_with_getting_data(text)
                .then_some((Error::GETTINGDATA, GETTING_DATA_LITERAL.len()))
        })
}

/// The error a stored en-US literal names, matched exactly: xlsx stores the upper-case
/// literal. Callers reading user text upper-case it first (the `rustsheet` facade's
/// `ErrorKind::from_literal`).
pub fn get_error_by_english_name(name: &str) -> Option<Error> {
    Error::ALL
        .into_iter()
        .find(|error| error.english_literal() == name)
}

/// The error a COUNTIF-family criterion names, in any case (`"#n/a"` counts `#N/A` cells;
/// probes/t4852-error-name-case I1:I6). Deliberately only the twelve older names:
/// COUNTIF with `"#BLOCKED!"`, `"#CONNECT!"` or `"#FIELD!"` counts 0 even over a cell holding
/// that error (T-1012, information.md), so they stay text criteria. `"#BUSY!"` is assumed to
/// behave the same (UNVERIFIED, T-1228).
pub fn english_criterion_error(name: &str) -> Option<Error> {
    Error::ALL[..12]
        .iter()
        .find(|error| same_error_name(error.english_literal(), name))
        .cloned()
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TableSpecifier {
    All,
    Data,
    Headers,
    ThisRow,
    Totals,
    /// `[[#Headers],[#Data]]`: everything above the totals row.
    HeadersData,
    /// `[[#Data],[#Totals]]`: everything below the header row.
    DataTotals,
}

/// The whitespace Excel keeps inside a structured reference's outer brackets, which the lexer
/// otherwise skips (syntax.md "Whitespace beside a missing argument is kept; a structured
/// reference keeps some and normalizes the rest", T-2801): the runs after the outer `[` and
/// before the outer `]`, kept as typed, and whether whitespace followed the comma before the
/// column, which prints as one space. Whitespace before that comma is dropped.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct TableSpacing {
    /// The run after the outer `[` (`T[ [Col]:[Other] ]`).
    pub open: String,
    /// The run before the outer `]`.
    pub close: String,
    /// Whether whitespace followed the comma (`T[[#Data], [Col]]`).
    pub comma_space: bool,
}

impl TableSpacing {
    /// Whether the outer brackets hold a run of their own.
    pub fn is_padded(&self) -> bool {
        !self.open.is_empty() || !self.close.is_empty()
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TableReference {
    ColumnReference(String),
    RangeReference((String, String)),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TokenType {
    Illegal(LexerError),
    EOF,
    Ident(String),         // abc123
    String(String),        // "A season"
    Number(f64),           // 123.4
    Boolean(bool),         // TRUE | FALSE
    Error(Error),          // #VALUE!
    SheetRefError(String), // Sheet1!#REF!: a deleted reference that keeps its sheet prefix
    Compare(OpCompare),    // <,>, ...
    Addition(OpSum),       // +,-
    Product(OpProduct),    // *,/
    Power,                 // ^
    LeftParenthesis,       // (
    RightParenthesis,      // )
    Colon,                 // :
    Semicolon,             // ;
    LeftBracket,           // [
    RightBracket,          // ]
    LeftBrace,             // {
    RightBrace,            // }
    Comma,                 // ,
    Bang,                  // !
    Percent,               // %
    And,                   // &
    At,                    // @
    Spill,                 // #
    Backslash,             // \
    Reference {
        sheet: Option<String>,
        row: i32,
        column: i32,
        absolute_column: bool,
        absolute_row: bool,
    },
    Range {
        sheet: Option<String>,
        left: ParsedReference,
        right: ParsedReference,
    },
    StructuredReference {
        table_name: String,
        specifier: Option<TableSpecifier>,
        table_reference: Option<TableReference>,
        spacing: TableSpacing,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::get_default_language;

    /// The five newer kinds (SPEC-0003) with their literal and ERROR.TYPE code.
    const NEWER: [(Error, &str, u8); 5] = [
        (Error::GETTINGDATA, "#GETTING_DATA", 8),
        (Error::CONNECT, "#CONNECT!", 10),
        (Error::BLOCKED, "#BLOCKED!", 11),
        (Error::FIELD, "#FIELD!", 13),
        (Error::BUSY, "#BUSY!", 0),
    ];

    #[test]
    fn newer_kinds_display_code_and_stored_name() {
        for (error, literal, code) in NEWER {
            assert_eq!(error.to_string(), literal);
            assert_eq!(error.error_type_code(), code);
            assert_eq!(get_error_by_english_name(literal), Some(error));
        }
    }

    #[test]
    fn stored_literal_saves_getting_data_as_na() {
        assert_eq!(Error::GETTINGDATA.stored_literal(), "#N/A");
        for error in [
            Error::CONNECT,
            Error::BLOCKED,
            Error::FIELD,
            Error::BUSY,
            Error::NA,
            Error::DIV,
        ] {
            assert_eq!(error.stored_literal(), error.to_string());
        }
    }

    #[test]
    fn only_getting_data_is_a_formula_name() {
        let language = get_default_language();
        for name in ["#GETTING_DATA", "#getting_data", "#Getting_Data"] {
            assert_eq!(get_error_by_name(name, language), Some(Error::GETTINGDATA));
        }
        for name in ["#CONNECT!", "#BLOCKED!", "#FIELD!", "#BUSY!", "#UNKNOWN!"] {
            assert_eq!(get_error_by_name(name, language), None, "{name}");
        }
    }

    #[test]
    fn criteria_error_names_are_unchanged() {
        // COUNTIF with "#BLOCKED!" counts 0 over a #BLOCKED! cell (T-1012, information.md).
        for (_, literal, _) in NEWER {
            assert_eq!(english_criterion_error(literal), None, "{literal}");
        }
        assert_eq!(english_criterion_error("#N/A"), Some(Error::NA));
        // Any case (probes/t4852-error-name-case I1, I3, I6).
        assert_eq!(english_criterion_error("#n/a"), Some(Error::NA));
        assert_eq!(english_criterion_error("#div/0!"), Some(Error::DIV));
    }

    #[test]
    fn one_english_literal_table() {
        // Every kind prints the literal its English lookup reads back, `#N/IMPL!` included.
        for error in Error::ALL {
            assert_eq!(get_error_by_english_name(&error.to_string()), Some(error));
        }
        assert_eq!(Error::NIMPL.to_string(), "#N/IMPL!");
        // Stored literals are upper case; lower case is the caller's to fold.
        assert_eq!(get_error_by_english_name("#n/a"), None);
    }

    #[test]
    fn typed_and_formula_error_names_read_in_any_case() {
        // probes/t4852-error-name-case A1:A14 (typed) and E1:E8 (formula literals).
        let language = get_default_language();
        for (name, error) in [
            ("#n/a", Error::NA),
            ("#N/a", Error::NA),
            ("#Ref!", Error::REF),
            ("#spill!", Error::SPILL),
            ("#calc!", Error::CALC),
        ] {
            assert_eq!(get_error_by_name(name, language), Some(error), "{name}");
        }
        assert_eq!(
            formula_error_prefix("#Div/0!+1", language),
            Some((Error::DIV, 7))
        );
        assert_eq!(
            formula_error_prefix("#null!", language),
            Some((Error::NULL, 6))
        );
        assert_eq!(
            formula_error_prefix("#getting_data", language),
            Some((Error::GETTINGDATA, 13))
        );
        assert_eq!(formula_error_prefix("#blocked!", language), None);
    }
}
