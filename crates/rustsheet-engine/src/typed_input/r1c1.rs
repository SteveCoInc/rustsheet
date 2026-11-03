//! Excel's short R1C1 forms, typed in R1C1 mode, rewritten into the engine's R1C1 storage grammar.
//!
//! Typed in R1C1 mode (`Range.formulasR1C1`), Excel reads a bare `R` or `C` as an offset of 0
//! (`RC[-1]`, `R[-1]C`, `RC`), a row alone as the whole row (`R2`, `R[1]`, `R`) and a column alone
//! as the whole column (`C3`, `C[-1]`, `C`), in any letter case, and wraps a relative offset
//! around the grid (`R[-100]C[-7]` in H45 is A1048521). The engine's storage grammar always
//! writes both numbers, and there a bare `R`, `C` or `RC` is a name (a LAMBDA parameter re-parses
//! from storage), so the typed text is rewritten here before it is parsed. Evidence:
//! probes/t3212-r1c1-bare-offsets (excel-graph, 2026-11-03, T-3212); docs/excel-semantics/syntax.md
//! "Short R1C1 forms typed in R1C1 mode".

use crate::constants::{LAST_COLUMN, LAST_ROW};

/// One axis of an R1C1 reference as typed: `R` alone, `R5` or `R[-2]`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    /// No number: an offset of 0.
    Bare,
    Absolute(i64),
    Relative(i64),
}

/// A token shaped like an R1C1 reference: a row part, a column part, or both.
#[derive(Clone, Copy)]
struct Shape {
    row: Option<Part>,
    column: Option<Part>,
    end: usize,
}

/// How a typed short form reads: the storage text to write, the text kept as written (a name
/// such as `R0C1`, which Excel never resolves), or a refusal.
enum Rewrite {
    Text(String),
    Keep,
    Refused,
}

/// `formula` (without its `=`) typed in R1C1 mode into the 1-based `row`, `column`, with each short
/// R1C1 form written out in the storage grammar: `R[n]C[n]`, `RnCn`, and a whole row or column as
/// the range of its first and last cells. `None` when Excel refuses the text: an offset of a whole
/// grid or more (`RC[16384]`), or an absolute column past the grid (`R2C16385`). Text inside
/// string literals, quoted sheet names and brackets (structured and external references) is
/// copied as it is.
pub(super) fn expand_short_forms(formula: &str, row: i32, column: i32) -> Option<String> {
    let chars: Vec<char> = formula.chars().collect();
    let mut out = String::with_capacity(formula.len() + 16);
    let mut index = 0;
    while index < chars.len() {
        let c = chars[index];
        match c {
            '"' | '\'' => {
                let end = quoted_end(&chars, index);
                out.extend(&chars[index..end]);
                index = end;
            }
            '[' => {
                let end = bracket_end(&chars, index);
                out.extend(&chars[index..end]);
                index = end;
            }
            _ if starts_token(&chars, index) => {
                let Some(first) = shape_at(&chars, index) else {
                    index = copy_word(&chars, index, &mut out);
                    continue;
                };
                // A row alone or a column alone joined by `:` to another of its kind is one
                // range of whole rows or columns (`R[-1]:R[1]`, `C2:C3`).
                let second = (chars.get(first.end) == Some(&':'))
                    .then(|| shape_at(&chars, first.end + 1))
                    .flatten()
                    .filter(|s| is_line(&first) && same_axis(&first, s));
                let (rewrite, end) = match second {
                    Some(second) => (rewrite_lines(&first, &second, row, column), second.end),
                    None => (rewrite_one(&first, row, column), first.end),
                };
                match rewrite {
                    Rewrite::Text(text) => out.push_str(&text),
                    Rewrite::Keep => out.extend(&chars[index..end]),
                    Rewrite::Refused => return None,
                }
                index = end;
            }
            _ if is_word_char(c) => index = copy_word(&chars, index, &mut out),
            _ => {
                out.push(c);
                index += 1;
            }
        }
    }
    Some(out)
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '.' | '\\')
}

/// Whether a token that may be an R1C1 reference starts at `index`: an `R` or `C` that does not
/// continue a word or a number.
fn starts_token(chars: &[char], index: usize) -> bool {
    matches!(chars[index], 'R' | 'r' | 'C' | 'c') && (index == 0 || !is_word_char(chars[index - 1]))
}

/// Copies the word starting at `index` (a name, a number, a function) and returns its end.
fn copy_word(chars: &[char], mut index: usize, out: &mut String) -> usize {
    while index < chars.len() && is_word_char(chars[index]) {
        out.push(chars[index]);
        index += 1;
    }
    index
}

/// The end of the quoted text starting at `index` (a doubled quote stays inside).
fn quoted_end(chars: &[char], index: usize) -> usize {
    let quote = chars[index];
    let mut i = index + 1;
    while i < chars.len() {
        if chars[i] == quote {
            if chars.get(i + 1) == Some(&quote) {
                i += 2;
                continue;
            }
            return i + 1;
        }
        i += 1;
    }
    chars.len()
}

/// The end of the bracketed text starting at `index`, nested brackets included.
fn bracket_end(chars: &[char], index: usize) -> usize {
    let mut depth = 0_usize;
    for (i, c) in chars.iter().enumerate().skip(index) {
        match c {
            '[' => depth += 1,
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
    }
    chars.len()
}

/// The number part after an `R` or `C` at `index`: `[n]`, digits, or nothing.
fn part_at(chars: &[char], index: usize) -> Option<(Part, usize)> {
    match chars.get(index) {
        Some('[') => {
            let mut i = index + 1;
            let negative = chars.get(i) == Some(&'-');
            if negative || chars.get(i) == Some(&'+') {
                i += 1;
            }
            let (value, end) = digits_at(chars, i)?;
            if chars.get(end) != Some(&']') {
                return None;
            }
            Some((
                Part::Relative(if negative { -value } else { value }),
                end + 1,
            ))
        }
        Some(c) if c.is_ascii_digit() => {
            let (value, end) = digits_at(chars, index)?;
            Some((Part::Absolute(value), end))
        }
        _ => Some((Part::Bare, index)),
    }
}

/// The decimal number at `index` and its end, `None` when there is none. A number too long for
/// `i64` saturates; it is past the grid either way.
fn digits_at(chars: &[char], index: usize) -> Option<(i64, usize)> {
    let mut value: i64 = 0;
    let mut end = index;
    while let Some(d) = chars.get(end).and_then(|c| c.to_digit(10)) {
        value = value.saturating_mul(10).saturating_add(i64::from(d));
        end += 1;
    }
    (end > index).then_some((value, end))
}

/// The R1C1-shaped token at `index`, when the whole word there is one (`RC2X`, `ROUND(` and
/// `C3!` are not).
fn shape_at(chars: &[char], index: usize) -> Option<Shape> {
    let mut i = index;
    let mut row = None;
    if matches!(chars.get(i), Some('R' | 'r')) {
        let (part, end) = part_at(chars, i + 1)?;
        row = Some(part);
        i = end;
    }
    let mut column = None;
    if matches!(chars.get(i), Some('C' | 'c')) {
        let (part, end) = part_at(chars, i + 1)?;
        column = Some(part);
        i = end;
    }
    if row.is_none() && column.is_none() {
        return None;
    }
    if chars
        .get(i)
        .is_some_and(|&c| is_word_char(c) || matches!(c, '(' | '!' | '['))
    {
        return None;
    }
    Some(Shape {
        row,
        column,
        end: i,
    })
}

/// Whether `shape` is a whole row or a whole column (one axis only).
fn is_line(shape: &Shape) -> bool {
    shape.row.is_none() != shape.column.is_none()
}

fn same_axis(a: &Shape, b: &Shape) -> bool {
    is_line(b) && a.row.is_some() == b.row.is_some()
}

/// The storage text of one axis, or why there is none. `size` is the grid's extent on that axis
/// and `at` the typed cell's index on it.
fn axis_text(part: Part, axis: char, at: i32, size: i32) -> Result<String, Rewrite> {
    let size = i64::from(size);
    match part {
        Part::Bare => Ok(format!("{axis}[0]")),
        // `R0C1` and `R1048577C1` stay names (`#NAME?`, probes/t3212 H38, H49), but an absolute
        // column past the grid is refused (H47, `R2C16385`).
        Part::Absolute(n) if n == 0 || (axis == 'R' && n > size) => Err(Rewrite::Keep),
        Part::Absolute(n) if n > size => Err(Rewrite::Refused),
        Part::Absolute(n) => Ok(format!("{axis}{n}")),
        // An offset of a whole grid or more is refused (`RC[16384]`, `R[1048576]C`: H46, H48);
        // a smaller one wraps around the grid (H16 `RC[-100]` in column H is XBP; H45).
        Part::Relative(n) if n.abs() >= size => Err(Rewrite::Refused),
        Part::Relative(n) => {
            let target = (i64::from(at) - 1 + n).rem_euclid(size) + 1;
            Ok(format!("{axis}[{}]", target - i64::from(at)))
        }
    }
}

fn rewrite_one(shape: &Shape, row: i32, column: i32) -> Rewrite {
    match (shape.row, shape.column) {
        (Some(r), Some(c)) => {
            match (
                axis_text(r, 'R', row, LAST_ROW),
                axis_text(c, 'C', column, LAST_COLUMN),
            ) {
                (Ok(r), Ok(c)) => Rewrite::Text(format!("{r}{c}")),
                (Err(Rewrite::Refused), _) | (_, Err(Rewrite::Refused)) => Rewrite::Refused,
                _ => Rewrite::Keep,
            }
        }
        _ => rewrite_lines(shape, shape, row, column),
    }
}

/// The range of the whole rows (or columns) from `first` to `last`, two shapes of one axis.
fn rewrite_lines(first: &Shape, last: &Shape, row: i32, column: i32) -> Rewrite {
    let line = |shape: &Shape| match (shape.row, shape.column) {
        (Some(r), None) => axis_text(r, 'R', row, LAST_ROW),
        (None, Some(c)) => axis_text(c, 'C', column, LAST_COLUMN),
        _ => Err(Rewrite::Keep),
    };
    match (line(first), line(last)) {
        (Ok(a), Ok(b)) if first.row.is_some() => Rewrite::Text(format!("{a}C1:{b}C{LAST_COLUMN}")),
        (Ok(a), Ok(b)) => Rewrite::Text(format!("R1{a}:R{LAST_ROW}{b}")),
        (Err(Rewrite::Refused), _) | (_, Err(Rewrite::Refused)) => Rewrite::Refused,
        _ => Rewrite::Keep,
    }
}
