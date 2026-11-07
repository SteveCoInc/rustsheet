//! The builder's refusals for author-supplied attributes and raw XML (`_attrs` with its managed
//! names, and `_raw`, of `tools/src/rustsheet_tools/pivot_definition.py`), for the slots T-4130
//! added: calculated items and page fields. A spec is untrusted input (directive 5), so the raw
//! XML a definition carries into the parts is one element of the expected tag, with no
//! declaration, processing instruction or bare `&`.

use serde_json::Value as Json;

use super::attrs;

/// `attrs`, refusing the names the builder writes itself (`managed`).
pub(super) fn managed_attrs(raw: Option<&Json>, managed: &[&str]) -> Result<String, String> {
    if let Some(name) = (raw.and_then(Json::as_object).into_iter().flatten())
        .map(|(name, _)| name)
        .find(|name| managed.contains(&name.as_str()))
    {
        return Err(format!("attribute {name:?} is the builder's"));
    }
    attrs(raw)
}

/// `_raw`: `text` trimmed, when it is one `<tag>` element (`<tag .../>` or `<tag ...>...</tag>`)
/// without `<!`, `<?` or an `&` that does not start one of XML's five entity references.
pub(super) fn raw_element(text: &str, tag: &str) -> Result<String, String> {
    let text = text.trim();
    let bad = || format!("raw XML must be one <{tag}> element without declarations");
    let rest = text
        .strip_prefix('<')
        .and_then(|t| t.strip_prefix(tag))
        .ok_or_else(bad)?;
    let head_ok = rest.starts_with(['>', '/']) || rest.starts_with(char::is_whitespace);
    let close = format!("</{tag}>");
    let whole = if let Some(at) = rest.find('>') {
        let head = &rest[..at];
        // The start tag holds no `<`, as `[^<>]*` in `_raw`.
        !head.contains('<')
            && (head.ends_with('/') && at + 1 == rest.len() || text.ends_with(&close))
    } else {
        false
    };
    let entities = ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"];
    let bare_amp = text
        .match_indices('&')
        .any(|(at, _)| !entities.iter().any(|e| text[at..].starts_with(e)));
    if !head_ok || !whole || text.contains("<!") || text.contains("<?") || bare_amp {
        return Err(bad());
    }
    Ok(text.to_string())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]
    use super::*;

    #[test]
    fn raw_elements_are_one_element_of_their_tag() {
        for ok in [
            "<pivotArea/>",
            " <pivotArea outline=\"0\"/> ",
            "<pivotArea a=\"&amp;\"><references/></pivotArea>",
        ] {
            assert!(raw_element(ok, "pivotArea").is_ok(), "{ok}");
        }
        for bad in [
            "<references/>",
            "<pivotAreas/>",
            "<pivotArea><!-- x --></pivotArea>",
            "<?xml version=\"1.0\"?><pivotArea/>",
            "<pivotArea a=\"&x;\"/>",
            "<pivotArea/><pivotArea/>",
            "<pivotArea>",
            "text",
        ] {
            assert!(raw_element(bad, "pivotArea").is_err(), "{bad}");
        }
    }

    #[test]
    fn managed_attributes_are_refused() {
        let raw: Json = serde_json::from_str(r#"{"cap": "W", "fld": 1}"#).unwrap();
        assert!(managed_attrs(Some(&raw), &["fld", "item", "hier"]).is_err());
        let raw: Json = serde_json::from_str(r#"{"cap": "W"}"#).unwrap();
        assert_eq!(managed_attrs(Some(&raw), &["fld"]).unwrap(), r#" cap="W""#);
    }
}
