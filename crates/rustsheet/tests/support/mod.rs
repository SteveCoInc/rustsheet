//! Helpers the facade's integration tests share.

use std::error::Error;
use std::io::{Cursor, Read as _, Write as _};

/// `bytes` with one text replacement in `part`.
pub fn rezip(bytes: &[u8], part: &str, from: &str, to: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut src = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut f = src.by_index(i)?;
        let name = f.name()?.to_string();
        if name.ends_with('/') {
            continue;
        }
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;
        if name == part {
            let text = String::from_utf8(data)?;
            assert!(text.contains(from), "{from}");
            data = text.replacen(from, to, 1).into_bytes();
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())?;
        out.write_all(&data)?;
    }
    Ok(out.finish()?.into_inner())
}
