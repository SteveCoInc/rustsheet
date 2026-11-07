use std::io::Read;

use super::package::Package;

use rustsheet_engine::types::Metadata;

use crate::error::XlsxError;

use super::util::get_value_or_default;
use super::xml::XmlNode;

/// `Application` when app.xml names none, or there is no app.xml.
const UNKNOWN_APPLICATION: &str = "Unknown application";

struct AppData {
    application: String,
    app_version: String,
}

struct CoreData {
    creator: String,
    last_modified_by: String,
    created: String,
    last_modified: String,
}

fn load_core<R: Read + std::io::Seek>(archive: &mut Package<R>) -> Result<CoreData, XlsxError> {
    let file = archive.open("docProps/core.xml")?;
    let doc = XmlNode::parse(file)?;
    let core_data = &doc;
    // Note the namespace should be "http://purl.org/dc/elements/1.1/"
    let creator = get_value_or_default(core_data, "creator", "Anonymous User");
    // Note namespace is "http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
    let last_modified_by = get_value_or_default(core_data, "lastModifiedBy", "Anonymous User");
    // In these two cases the namespace is "http://purl.org/dc/terms/"
    let created = get_value_or_default(core_data, "created", "");
    let last_modified = get_value_or_default(core_data, "modified", "");

    Ok(CoreData {
        creator,
        last_modified_by,
        created,
        last_modified,
    })
}

fn load_app<R: Read + std::io::Seek>(archive: &mut Package<R>) -> Result<AppData, XlsxError> {
    let file = archive.open("docProps/app.xml")?;
    let doc = XmlNode::parse(file)?;
    let app_data = &doc;

    let application = get_value_or_default(app_data, "Application", UNKNOWN_APPLICATION);
    let app_version = get_value_or_default(app_data, "AppVersion", "");
    Ok(AppData {
        application,
        app_version,
    })
}

/// The workbook's app and core properties. Each part is read on its own: a package without
/// `docProps/app.xml` (poi/xlsx-jdbc.xlsx) still keeps its creator and creation date (T-4909), and
/// a missing or unreadable part leaves its fields at the defaults.
pub(super) fn load_metadata<R: Read + std::io::Seek>(archive: &mut Package<R>) -> Metadata {
    let app = load_app(archive).unwrap_or_else(|_| AppData {
        application: UNKNOWN_APPLICATION.to_string(),
        app_version: String::new(),
    });
    let core = load_core(archive).unwrap_or_else(|_| CoreData {
        creator: String::new(),
        last_modified_by: String::new(),
        created: String::new(),
        last_modified: String::new(),
    });
    Metadata {
        application: app.application,
        app_version: app.app_version,
        creator: core.creator,
        last_modified_by: core.last_modified_by,
        created: core.created,
        last_modified: core.last_modified,
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write as _};

    use super::super::package::Package;
    use super::super::ImportLimits;
    use super::load_metadata;

    #[test]
    fn core_properties_are_kept_without_app_properties() {
        let core = r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/"><dc:creator>Apache POI</dc:creator><dcterms:created>2009-05-23T12:04:04Z</dcterms:created></cp:coreProperties>"#;
        let mut buf = Vec::new();
        {
            let mut zw = zip::ZipWriter::new(Cursor::new(&mut buf));
            zw.start_file(
                "docProps/core.xml",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zw.write_all(core.as_bytes()).unwrap();
            zw.finish().unwrap();
        }
        let mut package = Package::new(Cursor::new(buf), ImportLimits::default()).unwrap();
        let metadata = load_metadata(&mut package);
        assert_eq!(metadata.creator, "Apache POI");
        assert_eq!(metadata.created, "2009-05-23T12:04:04Z");
        assert_eq!(metadata.application, super::UNKNOWN_APPLICATION);
    }
}
