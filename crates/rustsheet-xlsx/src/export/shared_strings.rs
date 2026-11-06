use rustsheet_engine::types::Workbook;

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

/// XML consumers may normalize the whitespace around element text unless `xml:space="preserve"`
/// is set; Excel writes it on every run that starts or ends with whitespace (T-0497,
/// Ported-From: ironcalc@06f12748).
fn needs_space_preserve(text: &str) -> bool {
    text.chars().next().is_some_and(char::is_whitespace)
        || text.chars().next_back().is_some_and(char::is_whitespace)
}

pub(crate) fn get_shared_strings_xml(model: &Workbook) -> String {
    let mut shared_strings: Vec<String> = vec![];
    let count = &model.shared_strings.len();
    let unique_count = &model.shared_strings.len();
    for shared_string in &model.shared_strings {
        shared_strings.push(format!(
            "<si><t{}>{}</t></si>",
            if needs_space_preserve(shared_string) {
                r#" xml:space="preserve""#
            } else {
                ""
            },
            escape_xml(shared_string)
        ));
    }
    format!("{}\n\
      <sst xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" count=\"{count}\" uniqueCount=\"{unique_count}\">\
        {}\
      </sst>", XML_DECLARATION, shared_strings.join(""))
}
