//! The shared-string part of a patched save (SPEC-0034 D3.3): new strings appended after the
//! source's, with `uniqueCount` set and `count` moved by the change in references.

use quick_xml::events::Event;
use quick_xml::Reader;
use rustsheet_engine::types::{Cell, Workbook};

use super::super::escape::escape_xml;
use super::tag::set_attr;
use super::MAX_DEPTH;

/// The `<si>` elements directly under the shared-string part's root.
pub(super) fn count_si(xml: &str) -> Result<usize, String> {
    let mut reader = Reader::from_str(xml);
    let (mut depth, mut count) = (0usize, 0usize);
    loop {
        match reader.read_event().map_err(|e| e.to_string())? {
            Event::Start(e) => {
                depth += 1;
                if depth > MAX_DEPTH {
                    return Err("the shared-string part is nested too deep".into());
                }
                count += usize::from(depth == 2 && e.local_name().as_ref() == "si");
            }
            Event::Empty(e) => {
                count += usize::from(depth == 1 && e.local_name().as_ref() == "si");
            }
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Eof => return Ok(count),
            _ => {}
        }
    }
}

/// The shared-string part with `added` appended and its counts updated.
pub(super) fn append_strings(
    xml: &str,
    added: &[String],
    unique: usize,
    (live_refs, pristine_refs): (usize, usize),
) -> Option<String> {
    let close = xml.rfind("</sst>")?;
    let mut out = String::with_capacity(xml.len() + added.len() * 16);
    let head = set_count(&xml[..close], "uniqueCount", |_| unique);
    // `count` is the source's own reference count, moved by the change in references: a source
    // whose count was already off stays off by the same amount rather than being recounted.
    let head = set_count(&head, "count", |old| {
        (old + live_refs).saturating_sub(pristine_refs)
    });
    out.push_str(&head);
    for s in added {
        let space = if s.starts_with(char::is_whitespace) || s.ends_with(char::is_whitespace) {
            r#" xml:space="preserve""#
        } else {
            ""
        };
        out.push_str(&format!("<si><t{space}>{}</t></si>", escape_xml(s)));
    }
    out.push_str(&xml[close..]);
    Some(out)
}

/// `head` with the `name="..."` attribute of its `<sst ...>` start tag set to `value` of its old
/// number, when the tag has it and it is a number. Nothing past the start tag is searched.
pub(super) fn set_count(head: &str, name: &str, value: impl Fn(usize) -> usize) -> String {
    let Some(sst) = head
        .match_indices("<sst")
        .map(|(i, _)| i)
        .find(|&i| head[i + 4..].starts_with(|c: char| c.is_whitespace() || c == '>'))
    else {
        return head.to_string();
    };
    let Some(tag_len) = head[sst..].find('>') else {
        return head.to_string();
    };
    let tag = &head[sst..sst + tag_len];
    format!(
        "{}{}{}",
        &head[..sst],
        set_attr(tag, name, value),
        &head[sst + tag_len..]
    )
}

/// The number of cells holding a shared string, as the `count` of `sharedStrings.xml` counts
/// references.
pub(super) fn shared_refs(workbook: &Workbook) -> usize {
    workbook
        .worksheets
        .iter()
        .flat_map(|w| w.sheet_data.cells())
        .filter(|(_, _, c)| matches!(c, Cell::SharedString { .. }))
        .count()
}
