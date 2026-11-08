//! A part's elements as written, for the sidecar records whose every attribute is evidence
//! (T-4956): a pivot's `<pivotField>`s and `<filter>`s, its cache's extensions, and the slicer and
//! timeline parts. Kept as a tree of qualified names and attribute values, so a claim about what
//! Excel saves (an attribute it adds, keeps or drops) can be read off a committed sidecar.
//!
//! Untrusted input (directive 5): the tree is read with an explicit stack, at most `MAX_DEPTH`
//! deep, and every element and attribute is charged against the snapshot's workbook budget.

use std::collections::BTreeMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use serde::{Deserialize, Serialize};

use super::spend;

/// How deep a kept tree may nest; deeper is malformed for the snapshot (the parts it reads nest
/// about eight deep: a filter's `extLst/ext/x15:pivotFilter`, an `autoFilter`'s criteria).
pub(super) const MAX_DEPTH: usize = 32;

/// One element as written: its qualified name, its attributes (namespace declarations left out)
/// and its child elements. Text content is not kept: none of the parts read here carries any.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct XmlElement {
    /// The qualified name as written, e.g. `x15:pivotFilter`.
    pub name: String,
    /// Attribute values by qualified name as written, e.g. `xr10:uid`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attrs: BTreeMap<String, String>,
    /// Child elements, in document order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<XmlElement>,
}

impl XmlElement {
    /// The name without its prefix.
    #[must_use]
    pub fn local(&self) -> &str {
        self.name.rsplit_once(':').map_or(&self.name, |(_, l)| l)
    }

    /// The children whose local name is `local`.
    pub fn children_named<'a>(&'a self, local: &'a str) -> impl Iterator<Item = &'a XmlElement> {
        self.children.iter().filter(move |c| c.local() == local)
    }

    /// The attribute whose local name is `local` (an unprefixed one first).
    #[must_use]
    pub fn attr(&self, local: &str) -> Option<&str> {
        self.attrs.get(local).map(String::as_str)
    }

    /// The budget cost of keeping another copy of this tree, as [`read_element`] charges it.
    pub(super) fn cost(&self) -> usize {
        1 + self.name.len()
            + self
                .attrs
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>()
            + self.children.iter().map(XmlElement::cost).sum::<usize>()
    }
}

/// `e`'s attributes by qualified name (namespace declarations left out), each charged its bytes;
/// `None` on a malformed attribute or when the budget runs out.
pub(super) fn attr_map(e: &BytesStart<'_>, budget: &mut usize) -> Option<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    for a in e.attributes() {
        let a = a.ok()?;
        let key = a.key.as_ref();
        if key == "xmlns" || key.starts_with("xmlns:") {
            continue;
        }
        let value = a.normalized_value(XmlVersion::Implicit1_0).ok()?;
        spend(budget, key.len() + value.len())?;
        map.insert(key.to_owned(), value.into_owned());
    }
    Some(map)
}

/// The element `e` opens, without children, charged one unit plus its name's bytes.
fn element(e: &BytesStart<'_>, budget: &mut usize) -> Option<XmlElement> {
    let name = e.name().as_ref().to_owned();
    spend(budget, 1 + name.len())?;
    Some(XmlElement {
        name,
        attrs: attr_map(e, budget)?,
        children: Vec::new(),
    })
}

/// The whole element `start` opens (`empty` when it was `<.../>`), its children read from
/// `reader` up to its end tag. `None` when the XML is malformed or ends early, the tree nests past
/// `MAX_DEPTH`, or the budget runs out.
pub(super) fn read_element(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    empty: bool,
    budget: &mut usize,
) -> Option<XmlElement> {
    let root = element(start, budget)?;
    if empty {
        return Some(root);
    }
    let mut stack = vec![root];
    loop {
        match reader.read_event().ok()? {
            Event::Start(e) => {
                if stack.len() >= MAX_DEPTH {
                    return None;
                }
                stack.push(element(&e, budget)?);
            }
            Event::Empty(e) => {
                let child = element(&e, budget)?;
                stack.last_mut()?.children.push(child);
            }
            Event::End(_) => {
                let done = stack.pop()?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(done),
                    None => return Some(done),
                }
            }
            Event::Eof => return None,
            _ => {}
        }
    }
}

/// A part's root element and everything below it; `None` as [`read_element`].
pub(super) fn parse_root(xml: &str, budget: &mut usize) -> Option<XmlElement> {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event().ok()? {
            Event::Start(e) => return read_element(&mut reader, &e, false, budget),
            Event::Empty(e) => return read_element(&mut reader, &e, true, budget),
            Event::Eof => return None,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_qualified_names_and_attributes_but_not_namespace_declarations() {
        let xml = r#"<?xml version="1.0"?><filter xmlns="m" xmlns:x15="x" fld="2" type="dateBetween">
            <autoFilter ref="A1"><filterColumn colId="0"/></autoFilter>
            <extLst><ext uri="{U}"><x15:pivotFilter useWholeDay="1"/></ext></extLst></filter>"#;
        let mut budget = usize::MAX;
        let tree = parse_root(xml, &mut budget).expect("parses");
        assert_eq!(tree.name, "filter");
        assert_eq!(tree.attr("type"), Some("dateBetween"));
        assert!(!tree.attrs.contains_key("xmlns:x15"));
        let ext = &tree.children_named("extLst").next().expect("ext").children[0];
        assert_eq!(ext.children[0].name, "x15:pivotFilter");
        assert_eq!(ext.children[0].local(), "pivotFilter");
        assert_eq!(ext.children[0].attr("useWholeDay"), Some("1"));
        // The budget charged is the tree's cost.
        assert_eq!(usize::MAX - budget, tree.cost());
    }

    #[test]
    fn malformed_deep_or_over_budget_trees_are_none() {
        let mut budget = usize::MAX;
        assert!(parse_root("<a><b></a>", &mut budget).is_none());
        assert!(parse_root("<a><b/>", &mut budget).is_none());
        let deep = format!(
            "{}{}",
            "<a>".repeat(MAX_DEPTH + 1),
            "</a>".repeat(MAX_DEPTH + 1)
        );
        assert!(parse_root(&deep, &mut budget).is_none());
        let ok = format!("{}{}", "<a>".repeat(MAX_DEPTH), "</a>".repeat(MAX_DEPTH));
        assert!(parse_root(&ok, &mut budget).is_some());
        let tree = parse_root(r#"<a k="vv"><b/></a>"#, &mut usize::MAX.clone()).expect("parses");
        assert!(parse_root(r#"<a k="vv"><b/></a>"#, &mut (tree.cost() - 1)).is_none());
        assert!(parse_root(r#"<a k="vv"><b/></a>"#, &mut tree.cost()).is_some());
    }
}
