mod array_refs;
mod call_cap;
mod columns;
mod conditional_formatting;
mod encryption;
mod external_links;
mod filter_criteria;
mod iso_date;
mod merged_cells;
mod metadata;
mod ole;
mod package;
mod pivots;
pub(crate) mod relocation_key;
pub mod rich_errors;
pub(crate) mod shared_strings;
mod sheet_data;
mod styles;
mod tables;
mod theme;
mod util;
mod workbook;
mod worksheets;
mod xml;

use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Cursor, Read, Seek},
};

use xml::XmlNode;

use rustsheet_engine::{
    expressions::{
        parser::{new_parser_english, stringify::to_english_string},
        types::CellReferenceRC,
    },
    types::{Metadata, Table, Workbook, WorkbookSettings, WorkbookView},
    Model,
};

use crate::error::XlsxError;

pub use encryption::{
    decrypt_package, MAX_ENCRYPTION_INFO_BYTES, MAX_PASSWORD_UNITS, MAX_SPIN_COUNT,
};
pub use iso_date::{iso8601_to_date_system_serial, iso8601_to_serial};
pub use ole::{classify_ole_compound_file, decrypt_ole_package};
pub use package::{is_ole_compound_file, part_key, ImportLimits, PartNames};
pub(crate) use pivots::SUBTOTAL_FUNCTIONS as PIVOT_SUBTOTAL_FUNCTIONS;
pub use pivots::{read_pivot_parts, PivotParts, MAX_PIVOTS};
pub use shared_strings::decode_xlsx_escapes;
use shared_strings::read_shared_strings;
pub use workbook::date_system_from_attribute;

use call_cap::refuse_too_many_calls;
use external_links::load_external_books;
use metadata::load_metadata;
use package::{resolve_target, Package};
use pivots::{load_pivots, PivotSheet};
use styles::load_styles;
use util::get_attribute;
use workbook::load_workbook;
use worksheets::load_sheets;

/// The workbook's relationships part, which the importer reads and the export writes.
pub(crate) const WORKBOOK_RELS_PART: &str = "xl/_rels/workbook.xml.rels";
/// The package's content types.
pub(crate) const CONTENT_TYPES_PART: &str = "[Content_Types].xml";

/// A `<Relationship>` of a relationships part.
pub(crate) struct Relationship {
    pub(crate) target: String,
    pub(crate) rel_type: String,
    /// `TargetMode="External"`: the target is outside the package.
    pub(crate) external: bool,
}

fn load_relationships<R: Read + Seek>(
    archive: &mut Package<R>,
) -> Result<HashMap<String, Relationship>, XlsxError> {
    let file = archive.open(WORKBOOK_RELS_PART)?;
    Ok(parse_relationships(&XmlNode::parse(file)?)?
        .into_iter()
        .collect())
}

/// The relationships of a part in document order; none when the part has no relationships part.
pub(super) fn part_relationships<R: Read + Seek>(
    archive: &mut Package<R>,
    rels_path: &str,
) -> Result<Vec<(String, Relationship)>, XlsxError> {
    match archive.open_optional(rels_path)? {
        Some(file) => parse_relationships(&XmlNode::parse(file)?),
        None => Ok(Vec::new()),
    }
}

/// The `<Relationship>` elements of a relationships part with their `Id`s, in document order.
fn parse_relationships(doc: &XmlNode) -> Result<Vec<(String, Relationship)>, XlsxError> {
    let mut rels = Vec::new();
    for node in doc.descendants().filter(|n| n.has_tag_name("Relationship")) {
        rels.push((
            get_attribute(node, "Id")?.to_string(),
            Relationship {
                rel_type: get_attribute(node, "Type")?.to_string(),
                target: get_attribute(node, "Target")?.to_string(),
                external: node.attribute("TargetMode") == Some("External"),
            },
        ));
    }
    Ok(rels)
}

fn resolve_theme_path(rels: &HashMap<String, Relationship>) -> Option<String> {
    let target = &rels
        .values()
        .find(|r| r.rel_type.ends_with("/theme"))?
        .target;
    Some(resolve_target("xl", target))
}

// FIXME: This is a bit of a HACK. We basically re-parse all the defined names assuming the context is `A1`
// and there are no defined names.
fn reparse_formula_hack(
    name: &str,
    formula: &str,
    worksheets: &[String],
    tables: &HashMap<String, Table>,
) -> Result<String, XlsxError> {
    let defined_names = Vec::new();
    let mut parser = new_parser_english(worksheets.to_owned(), defined_names, tables.clone());
    // The body is a file's stored text: a padded bare column `T[ Col ]` loads as `#REF!`, as
    // Excel loads it (probes/t3258-name-padded-column-0..2, T-3258, T-4022).
    parser.set_loaded(true);
    let cell_reference = CellReferenceRC {
        sheet: worksheets[0].clone(),
        column: 1,
        row: 1,
    };
    let t = parser.parse(formula, &cell_reference);
    // A name body is held to the cell formulas' 65 nested calls: Excel refuses a workbook whose
    // name body nests 66 (probes/t3143-name-body-65, T-3143; syntax.md "Formula nesting depth cap").
    refuse_too_many_calls(&parser, &format!("definedName {name} body"))?;

    Ok(to_english_string(&t, &cell_reference))
}

/// The parts the importer reads a package's worksheets and shared strings from, in model order:
/// what the source-preserving export must patch to change them (SPEC-0034 D3). Resolved exactly as
/// [`load_package`] resolves them, so both name the same parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PartPaths {
    /// The part of each model worksheet, in the model's sheet order.
    pub(crate) worksheets: Vec<String>,
    /// The shared-strings part the importer reads.
    pub(crate) shared_strings: &'static str,
}

/// [`PartPaths`] of the package `bytes`, read under `limits`.
///
/// # Errors
/// When the package does not import as far as its workbook and relationships.
pub(crate) fn part_paths(bytes: &[u8], limits: &ImportLimits) -> Result<PartPaths, XlsxError> {
    let mut archive = Package::new(Cursor::new(bytes), *limits)?;
    let workbook = load_workbook(&mut archive)?;
    let rels = load_relationships(&mut archive)?;
    let mut worksheets = Vec::new();
    for sheet in &workbook.worksheets {
        let rel = util::get_relationship(&rels, &sheet.id)?;
        if rel.rel_type.ends_with("worksheet") {
            worksheets.push(resolve_target("xl", &rel.target));
        }
    }
    Ok(PartPaths {
        worksheets,
        shared_strings: shared_strings::SHARED_STRINGS_PART,
    })
}

fn load_xlsx_from_reader<R: Read + Seek>(
    name: String,
    reader: R,
    locale: &str,
    tz: &str,
    limits: &ImportLimits,
) -> Result<Workbook, XlsxError> {
    let mut archive = Package::new(reader, *limits)?;
    let workbook = load_package(&mut archive, name, locale, tz);
    // A part that ran over the byte budget may have been skipped as optional or surfaced as an
    // XML error; either way the overrun is what is reported.
    match archive.overrun() {
        Some(overrun) => Err(overrun),
        None => workbook,
    }
}

fn load_package<R: Read + Seek>(
    archive: &mut Package<R>,
    name: String,
    locale: &str,
    tz: &str,
) -> Result<Workbook, XlsxError> {
    let mut shared_strings = read_shared_strings(archive)?;
    let mut workbook = load_workbook(archive)?;
    let date_system = workbook.date_system;
    let rels = load_relationships(archive)?;
    let theme_path = resolve_theme_path(&rels);
    let theme = theme::load(archive, theme_path.as_deref());
    let mut tables = HashMap::new();
    // Styles must be loaded before the worksheets: conditional-formatting rules
    // stored in x14 `extLst` extensions carry inline `<x14:dxf>` formats that we
    // append to `styles.dxfs`, referencing them back by index from the rule.
    let mut styles = load_styles(archive, &theme)?;
    let (worksheets, selected_sheet) = load_sheets(
        archive,
        &rels,
        &workbook,
        &mut tables,
        &mut shared_strings,
        &theme,
        &mut styles.dxfs,
    )?;
    // Color filters name a differential format, so this waits for `styles.dxfs` (T-4558).
    tables::drop_dangling_color_filters(&mut tables, styles.dxfs.len());
    let pivot_sheets: Vec<PivotSheet> = workbook
        .worksheets
        .iter()
        .filter_map(|sheet| {
            let rel = rels.get(&sheet.id)?;
            rel.rel_type.ends_with("worksheet").then(|| PivotSheet {
                sheet_id: sheet.sheet_id,
                path: resolve_target("xl", &rel.target),
            })
        })
        .collect();
    let (pivot_caches, pivot_tables) = load_pivots(
        archive,
        &rels,
        &workbook.pivot_caches,
        &pivot_sheets,
        date_system,
    );
    // reparse formulas in defined names, since they may refer to sheets and tables that have been loaded
    let worksheet_names = worksheets
        .iter()
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    for dn in &mut workbook.defined_names {
        dn.formula = reparse_formula_hack(&dn.name, &dn.formula, &worksheet_names, &tables)?;
    }
    let external_books = load_external_books(archive, &rels, &workbook.external_references);
    let metadata = match load_metadata(archive) {
        Ok(metadata) => metadata,
        Err(_) => {
            // In case there is no metadata, add some
            Metadata {
                application: "Unknown application".to_string(),
                app_version: "".to_string(),
                creator: "".to_string(),
                last_modified_by: "".to_string(),
                created: "".to_string(),
                last_modified: "".to_string(),
            }
        }
    };
    let mut views = HashMap::new();
    views.insert(
        0,
        WorkbookView {
            sheet: selected_sheet,
            window_width: 800,
            window_height: 600,
        },
    );
    Ok(Workbook {
        shared_strings: shared_strings.into_strings(),
        defined_names: workbook.defined_names,
        worksheets,
        styles,
        name,
        settings: WorkbookSettings {
            tz: tz.to_string(),
            locale: locale.to_string(),
            date_system,
            iteration: workbook.iteration,
        },
        metadata,
        tables,
        views,
        theme,
        external_books,
        pivot_caches,
        pivot_tables,
    })
}

// Imports a file from disk into an internal representation
pub(crate) fn load_from_excel(
    file_name: &str,
    locale: &str,
    tz: &str,
) -> Result<Workbook, XlsxError> {
    let file_path = std::path::Path::new(file_name);
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let name = file_path
        .file_stem()
        .ok_or_else(|| XlsxError::IO("Could not extract workbook name".to_string()))?
        .to_string_lossy()
        .to_string();
    load_xlsx_from_reader(name, reader, locale, tz, &ImportLimits::default())
}

/// Loads a [Workbook] from the bytes of an xlsx file, under the default [`ImportLimits`].
/// This is useful, for instance, when bytes are transferred over the network
pub fn load_from_xlsx_bytes(
    bytes: &[u8],
    name: &str,
    locale: &str,
    tz: &str,
) -> Result<Workbook, XlsxError> {
    load_from_xlsx_bytes_with_limits(bytes, name, locale, tz, &ImportLimits::default())
}

/// Loads a [Workbook] from the bytes of an xlsx file, rejecting it with
/// [`XlsxError::Limit`] if it exceeds `limits`.
pub fn load_from_xlsx_bytes_with_limits(
    bytes: &[u8],
    name: &str,
    locale: &str,
    tz: &str,
    limits: &ImportLimits,
) -> Result<Workbook, XlsxError> {
    load_xlsx_from_reader(name.to_string(), Cursor::new(bytes), locale, tz, limits)
}

/// Loads a [Model] from an xlsx file
pub fn load_from_xlsx(
    file_name: &str,
    locale: &str,
    tz: &str,
    language: &str,
) -> Result<Model, XlsxError> {
    let workbook = load_from_excel(file_name, locale, tz)?;
    Model::from_workbook(workbook, language).map_err(XlsxError::Workbook)
}

/// Loads a [Model] from an `ic` file (a file in the IronCalc internal representation)
pub fn load_from_icalc(file_name: &str, language_id: &str) -> Result<Model, XlsxError> {
    let contents = fs::read(file_name)
        .map_err(|e| XlsxError::IO(format!("Could not extract workbook name: {e}")))?;
    let workbook: Workbook = bitcode::decode(&contents)
        .map_err(|e| XlsxError::IO(format!("Failed to decode file: {e}")))?;
    Model::from_workbook(workbook, language_id).map_err(XlsxError::Workbook)
}
