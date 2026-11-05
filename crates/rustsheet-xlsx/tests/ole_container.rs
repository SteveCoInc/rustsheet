//! OLE compound files handed to the xlsx importer: an encrypted xlsx, a legacy binary workbook and
//! any other container each get their own typed error, and a malformed container is an error,
//! never a panic (decision 0279, SPEC-0031 D1, T-4264).

use std::error::Error;
use std::io::{Cursor, Write};

use rustsheet_xlsx::error::{ImportLimit, XlsxError};
use rustsheet_xlsx::import::{
    classify_ole_compound_file, load_from_xlsx_bytes, load_from_xlsx_bytes_with_limits,
    ImportLimits,
};

type TestResult = Result<(), Box<dyn Error>>;

/// Sector size of a version 3 container, the version Excel writes (MS-CFB 2.2).
const SECTOR: usize = 512;
/// Size of one directory entry (MS-CFB 2.6).
const DIR_ENTRY: usize = 128;

/// A version 3 compound file holding `streams` (name, content) in its root storage and `storages`
/// as empty storages.
fn container(streams: &[(&str, &[u8])], storages: &[&str]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file =
        cfb::CompoundFile::create_with_version(cfb::Version::V3, Cursor::new(Vec::new()))?;
    for name in storages {
        file.create_storage(format!("/{name}"))?;
    }
    for (name, content) in streams {
        file.create_stream(format!("/{name}"))?.write_all(content)?;
    }
    file.flush()?;
    Ok(file.into_inner().into_inner())
}

fn load(bytes: &[u8]) -> Option<XlsxError> {
    load_from_xlsx_bytes(bytes, "case", "en", "UTC").err()
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, Box<dyn Error>> {
    let word = bytes.get(offset..offset + 4).ok_or("offset past the end")?;
    Ok(u32::from_le_bytes(word.try_into()?))
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) -> TestResult {
    bytes
        .get_mut(offset..offset + 4)
        .ok_or("offset past the end")?
        .copy_from_slice(&value.to_le_bytes());
    Ok(())
}

/// Byte offset of sector `sector` (the header takes the first sector's slot).
fn sector_offset(sector: u32) -> usize {
    (sector as usize + 1) * SECTOR
}

/// Byte offset of directory entry `index`, for a directory that fits in its first sector.
fn dir_entry_offset(bytes: &[u8], index: usize) -> Result<usize, Box<dyn Error>> {
    // Header field `_sectDirStart` at offset 48 (MS-CFB 2.2).
    Ok(sector_offset(u32_at(bytes, 48)?) + index * DIR_ENTRY)
}

#[test]
fn an_encrypted_xlsx_is_its_own_error() -> TestResult {
    // The directory of poi 58616.xlsx and protected_passtika.xlsx (version 3 containers, read
    // with `cfb` from eval/.cache/suites/poi; both now give this error in `rustsheet-eval show`).
    // Only the stream sizes differ from the fixtures; their contents are not read yet.
    let bytes = container(
        &[
            ("\u{6}DataSpaces/Version", &[0; 76]),
            ("\u{6}DataSpaces/DataSpaceMap", &[0; 112]),
            (
                "\u{6}DataSpaces/DataSpaceInfo/StrongEncryptionDataSpace",
                &[0; 64],
            ),
            (
                "\u{6}DataSpaces/TransformInfo/StrongEncryptionTransform/\u{6}Primary",
                &[0; 200],
            ),
            ("EncryptionInfo", &[4, 0, 4, 0]),
            ("EncryptedPackage", &[0; 16]),
        ],
        &[
            "\u{6}DataSpaces",
            "\u{6}DataSpaces/DataSpaceInfo",
            "\u{6}DataSpaces/TransformInfo",
            "\u{6}DataSpaces/TransformInfo/StrongEncryptionTransform",
        ],
    )?;
    assert_eq!(load(&bytes), Some(XlsxError::EncryptedPackage));
    // Names compare case-insensitively, as in MS-CFB.
    let bytes = container(&[("encryptioninfo", &[1]), ("ENCRYPTEDPACKAGE", &[2])], &[])?;
    assert_eq!(load(&bytes), Some(XlsxError::EncryptedPackage));
    // One of the two streams alone is not an encrypted package.
    for stream in ["EncryptionInfo", "EncryptedPackage"] {
        let bytes = container(&[(stream, &[1])], &[])?;
        assert_eq!(load(&bytes), Some(XlsxError::OleCompoundFile), "{stream}");
    }
    Ok(())
}

#[test]
fn a_legacy_binary_workbook_is_its_own_error() -> TestResult {
    // BIFF8 (Excel 97-2003) keeps its record stream in `Workbook`, BIFF5 in `Book`.
    for stream in ["Workbook", "Book", "WORKBOOK"] {
        let bytes = container(
            &[
                (stream, &[0x09, 0x08, 0x10, 0x00]),
                ("\u{5}SummaryInformation", &[0]),
            ],
            &[],
        )?;
        assert_eq!(
            load(&bytes),
            Some(XlsxError::LegacyBinaryWorkbook),
            "{stream}"
        );
    }
    // A storage of that name is not a workbook stream.
    let bytes = container(&[], &["Workbook"])?;
    assert_eq!(load(&bytes), Some(XlsxError::OleCompoundFile));
    Ok(())
}

#[test]
fn any_other_container_keeps_the_generic_error() -> TestResult {
    // A Word document, and an empty container.
    let bytes = container(&[("WordDocument", &[0xEC, 0xA5])], &[])?;
    assert_eq!(load(&bytes), Some(XlsxError::OleCompoundFile));
    assert_eq!(
        load(&container(&[], &[])?),
        Some(XlsxError::OleCompoundFile)
    );
    // A stream named `Workbook` below the root storage is not the workbook stream.
    let mut file = cfb::CompoundFile::create(Cursor::new(Vec::new()))?;
    file.create_storage("/Nested")?;
    file.create_stream("/Nested/Workbook")?.write_all(&[0])?;
    file.flush()?;
    let bytes = file.into_inner().into_inner();
    assert_eq!(load(&bytes), Some(XlsxError::OleCompoundFile));
    Ok(())
}

#[test]
fn the_signature_alone_decides_what_reaches_the_container_parser() {
    // T-3972: the signature, or the signature and a zeroed header, is an unreadable container.
    let mut ole = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    assert_eq!(load(&ole), Some(XlsxError::OleCompoundFile));
    ole.extend_from_slice(&[0; 504]);
    assert_eq!(load(&ole), Some(XlsxError::OleCompoundFile));
    // Neither zip nor OLE (a short file, a near-miss signature): still a zip error.
    for bytes in [
        &b"PK"[..],
        &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE0][..],
    ] {
        let err = load(bytes);
        assert!(matches!(err, Some(XlsxError::Zip(_))), "{err:?}");
    }
}

#[test]
fn a_container_over_the_byte_budget_is_the_limit_error() -> TestResult {
    let bytes = container(
        &[("EncryptionInfo", &[1]), ("EncryptedPackage", &[0; 8192])],
        &[],
    )?;
    let limits = ImportLimits {
        max_uncompressed_bytes: 4096,
        ..ImportLimits::default()
    };
    let over = Some(XlsxError::Limit {
        limit: ImportLimit::UncompressedBytes,
        max: 4096,
    });
    assert_eq!(
        load_from_xlsx_bytes_with_limits(&bytes, "case", "en", "UTC", &limits).err(),
        over
    );
    // A file within the budget whose `EncryptedPackage` declares more than it.
    let mut small = container(
        &[("EncryptionInfo", &[1]), ("EncryptedPackage", &[0; 7000])],
        &[],
    )?;
    let limits = ImportLimits {
        max_uncompressed_bytes: small.len() as u64,
        ..ImportLimits::default()
    };
    assert_eq!(
        classify_ole_compound_file(&small, &limits),
        XlsxError::EncryptedPackage
    );
    set_stream_size(&mut small, 2, limits.max_uncompressed_bytes + 1)?;
    assert_eq!(
        classify_ole_compound_file(&small, &limits),
        XlsxError::Limit {
            limit: ImportLimit::UncompressedBytes,
            max: limits.max_uncompressed_bytes,
        }
    );
    Ok(())
}

/// Sets the size of directory entry `index`: the u64 at offset 120 of the entry (MS-CFB 2.6.3).
fn set_stream_size(bytes: &mut [u8], index: usize, size: u64) -> TestResult {
    let at = dir_entry_offset(bytes, index)? + 120;
    bytes
        .get_mut(at..at + 8)
        .ok_or("offset past the end")?
        .copy_from_slice(&size.to_le_bytes());
    Ok(())
}

#[test]
fn a_stream_declared_past_the_end_of_the_file_is_classified_without_reading_it() -> TestResult {
    // `cfb` checks a stream's sector chain against its size only when the stream is read, and
    // nothing is read here: a declared size past the end of the file is classified by name and
    // size alone (T-4265, which reads the streams, must treat a short read as malformed).
    let mut bytes = container(
        &[
            ("EncryptionInfo", &[1; 600]),
            ("EncryptedPackage", &[2; 600]),
        ],
        &[],
    )?;
    // 4 GiB - 1, over the default 256 MiB budget.
    set_stream_size(&mut bytes, 2, u64::from(u32::MAX))?;
    assert_eq!(
        load(&bytes),
        Some(XlsxError::Limit {
            limit: ImportLimit::UncompressedBytes,
            max: ImportLimits::default().max_uncompressed_bytes,
        })
    );
    // A version 3 file's sizes are 32 bits: the high half is ignored (MS-CFB 2.6.3), so this
    // declares 1,000 bytes, past the 600 the chain holds but within the budget.
    set_stream_size(&mut bytes, 2, (1 << 40) + 1000)?;
    assert_eq!(load(&bytes), Some(XlsxError::EncryptedPackage));
    Ok(())
}

#[test]
fn a_directory_cycle_is_an_error() -> TestResult {
    let mut bytes = container(&[("Workbook", &[1]), ("Other", &[2])], &[])?;
    // Point every entry's left and right sibling and child at entry 1, the root's child
    // (offsets 68, 72 and 76 of a directory entry, MS-CFB 2.6.1).
    for index in 1..3 {
        let entry = dir_entry_offset(&bytes, index)?;
        for field in [68, 72, 76] {
            put_u32(&mut bytes, entry + field, 1)?;
        }
    }
    assert_eq!(load(&bytes), Some(XlsxError::OleCompoundFile));
    Ok(())
}

#[test]
fn a_sector_chain_cycle_is_an_error() -> TestResult {
    let mut bytes = container(&[("Workbook", &[1; 5000])], &[])?;
    // The directory's own chain made to loop: its FAT entry points back at itself. The first FAT
    // sector is the first DIFAT entry of the header (offset 76).
    let fat = sector_offset(u32_at(&bytes, 76)?);
    let dir = u32_at(&bytes, 48)?;
    put_u32(&mut bytes, fat + 4 * dir as usize, dir)?;
    assert_eq!(load(&bytes), Some(XlsxError::OleCompoundFile));
    Ok(())
}

#[test]
fn a_directory_past_the_entry_cap_is_refused() -> TestResult {
    // 4,096 entries in all (the root and 4,095 streams) are classified; one more is refused.
    let names: Vec<String> = (0..4095).map(|i| format!("s{i}")).collect();
    let mut streams: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), &[][..])).collect();
    streams[0] = ("Workbook", &[1]);
    assert_eq!(
        load(&container(&streams, &[])?),
        Some(XlsxError::LegacyBinaryWorkbook)
    );
    streams.push(("one more", &[]));
    assert_eq!(
        load(&container(&streams, &[])?),
        Some(XlsxError::OleCompoundFile)
    );
    Ok(())
}

#[test]
fn the_new_errors_name_the_cause() {
    // The facade and the harness show these messages (`!InvalidCase` in eval/baseline/poi.txt).
    assert!(XlsxError::EncryptedPackage
        .user_message()
        .contains("password-protected"));
    assert!(XlsxError::LegacyBinaryWorkbook
        .user_message()
        .contains(".xls"));
}

#[test]
fn every_single_byte_corruption_is_an_error_not_a_panic() -> TestResult {
    // A deterministic sweep the `cfb_container` fuzz target extends: each byte of an encrypted
    // and a binary container set to 0x00, 0xFF and its complement, classified under a small
    // budget. Every result is one of the refusals; none panics.
    let small = ImportLimits {
        max_uncompressed_bytes: 64 << 10,
        ..ImportLimits::default()
    };
    for original in [
        container(
            &[
                ("EncryptionInfo", &[4; 248]),
                ("EncryptedPackage", &[5; 700]),
            ],
            &["\u{6}DataSpaces"],
        )?,
        container(&[("Workbook", &[9; 5000])], &[])?,
    ] {
        for at in 0..original.len() {
            for value in [0x00, 0xFF, !original[at]] {
                let mut bytes = original.clone();
                bytes[at] = value;
                let err = classify_ole_compound_file(&bytes, &small);
                assert!(
                    matches!(
                        err,
                        XlsxError::OleCompoundFile
                            | XlsxError::EncryptedPackage
                            | XlsxError::LegacyBinaryWorkbook
                            | XlsxError::Limit { .. }
                    ),
                    "byte {at} = {value}: {err:?}"
                );
            }
        }
    }
    Ok(())
}
