//! Attributes of one XML start tag, read and rewritten in place for a patched part: every other
//! byte of the tag is kept.

/// Where the value of attribute `name` lies in the start tag `tag` (quotes excluded). Either quote
/// is read, any whitespace may separate the name from `=` and the value, and a quoted value of
/// another attribute is skipped, so text such as `x='a count="9"'` never matches `count`.
pub(super) fn attr_span(tag: &str, name: &str) -> Option<(usize, usize)> {
    let bytes = tag.as_bytes();
    let mut i = 0;
    let mut after_space = false;
    while i < bytes.len() {
        match bytes[i] {
            quote @ (b'"' | b'\'') => {
                // `i` is an ASCII quote, so `i + 1` is a char boundary.
                i += 1 + tag[i + 1..].find(char::from(quote))? + 1;
                after_space = false;
                continue;
            }
            b if b.is_ascii_whitespace() => after_space = true,
            _ => {
                // After ASCII whitespace `i` is a char boundary.
                if after_space && bytes[i..].starts_with(name.as_bytes()) {
                    let rest = tag[i + name.len()..].trim_start();
                    if let Some(value) = rest.strip_prefix('=').map(str::trim_start) {
                        let quote = value.chars().next().filter(|q| matches!(q, '"' | '\''))?;
                        let start = tag.len() - value.len() + 1;
                        let len = tag[start..].find(quote)?;
                        return Some((start, start + len));
                    }
                }
                after_space = false;
            }
        }
        i += 1;
    }
    None
}

/// `tag` with the number in attribute `name` set to `value` of it, or `tag` as it was when the
/// attribute is absent or not a number.
pub(super) fn set_attr(tag: &str, name: &str, value: impl Fn(usize) -> usize) -> String {
    let Some((start, end)) = attr_span(tag, name) else {
        return tag.to_string();
    };
    let Ok(old) = tag[start..end].parse::<usize>() else {
        return tag.to_string();
    };
    format!("{}{}{}", &tag[..start], value(old), &tag[end..])
}
