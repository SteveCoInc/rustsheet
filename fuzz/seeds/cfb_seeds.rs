//! Writes the seed corpus of the `cfb_container` fuzz target: small MS-CFB containers laid out
//! like an encrypted xlsx, a legacy `.xls` and another OLE file, since random bytes rarely parse
//! as a container (T-4264).
//!
//! Usage: `cargo run --bin cfb_seeds -- <directory>` (`seed_corpus.py` runs it).

use std::io::{Cursor, Write};
use std::path::PathBuf;

/// Each seed: its storages (created first, in order) and its streams, as (path, length).
type Seed = (&'static [&'static str], &'static [(&'static str, usize)]);

const SEEDS: &[Seed] = &[
    // An encrypted xlsx: the directory of poi 58616.xlsx and protected_passtika.xlsx.
    (
        &[
            "\u{6}DataSpaces",
            "\u{6}DataSpaces/DataSpaceInfo",
            "\u{6}DataSpaces/TransformInfo",
            "\u{6}DataSpaces/TransformInfo/StrongEncryptionTransform",
        ],
        &[
            ("\u{6}DataSpaces/Version", 76),
            ("\u{6}DataSpaces/DataSpaceMap", 112),
            ("\u{6}DataSpaces/DataSpaceInfo/StrongEncryptionDataSpace", 64),
            (
                "\u{6}DataSpaces/TransformInfo/StrongEncryptionTransform/\u{6}Primary",
                200,
            ),
            ("EncryptionInfo", 248),
            ("EncryptedPackage", 4104),
        ],
    ),
    // A BIFF8 workbook with its property streams, and a BIFF5 one.
    (
        &[],
        &[
            ("Workbook", 5000),
            ("\u{5}SummaryInformation", 200),
            ("\u{5}DocumentSummaryInformation", 200),
        ],
    ),
    (&[], &[("Book", 600)]),
    // Some other OLE file, and an empty one.
    (&[], &[("WordDocument", 600), ("1Table", 100)]),
    (&[], &[]),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory: PathBuf = std::env::args_os()
        .nth(1)
        .ok_or("usage: cfb_seeds <directory>")?
        .into();
    std::fs::create_dir_all(&directory)?;
    for (index, (storages, streams)) in SEEDS.iter().enumerate() {
        // Version 3 (512-byte sectors), the version Excel writes.
        let mut file =
            cfb::CompoundFile::create_with_version(cfb::Version::V3, Cursor::new(Vec::new()))?;
        for storage in *storages {
            file.create_storage(format!("/{storage}"))?;
        }
        for (path, len) in *streams {
            let filler = u8::try_from(index)?;
            file.create_stream(format!("/{path}"))?
                .write_all(&vec![filler; *len])?;
        }
        file.flush()?;
        std::fs::write(
            directory.join(format!("seed-{index}.cfb")),
            file.into_inner().into_inner(),
        )?;
    }
    Ok(())
}
