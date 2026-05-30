use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::time::Duration;

use librarium::{ArchiveReader, ArchiveWriter, CpioHeader, NewcCrcHeader, NewcHeader};
use test_assets_ureq::{TestAsset, dl_test_files_backoff};

/// Read a newc cpio archive created by system cpio, then round-trip it through librarium
#[test_log::test]
fn test_newc_crc_compat() {
    const TEST_PATH: &str = ".";
    let filepath = "test-assets/test_newc_crc_compat/newc-crc-compat.cpio";
    let og_path = format!("{TEST_PATH}/{filepath}");
    let new_path = format!("{TEST_PATH}/test-assets/test_newc_crc_compat/bytes.cpio");

    let mut config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("../test-assets.toml");
    let file_content = std::fs::read_to_string(config_path).unwrap();
    let parsed: TestAsset = toml::from_str(&file_content).unwrap();
    let assets = parsed.values();

    dl_test_files_backoff(&assets, TEST_PATH, Duration::from_secs(1)).unwrap();

    let mut file = BufReader::new(File::open(&og_path).unwrap());
    let mut archive: ArchiveReader<NewcHeader> =
        ArchiveReader::from_reader_with_offset(&mut file, 0).unwrap();

    // Verify expected file data
    let a_assert = b"a\n";
    let b_assert = b"b\n";

    let mut a = Cursor::new(Vec::new());
    archive.extract_by_name("test/a", &mut a).unwrap();
    assert_eq!(a.into_inner(), a_assert);

    let mut b = Cursor::new(Vec::new());
    archive.extract_by_name("test/sub/b", &mut b).unwrap();
    assert_eq!(b.into_inner(), b_assert);

    // Round-trip: write it back out and compare bytes
    let out_file = File::create(&new_path).unwrap();
    let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(out_file));

    // test (dir)
    let header_0 = archive.objects.inner[0].header.as_header();
    writer.push_empty(header_0).unwrap();

    // test/a (file)
    let header_1 = archive.objects.inner[1].header.as_header();
    writer.push_file(Cursor::new(a_assert.as_slice()), header_1).unwrap();

    // test/sub (dir)
    let header_2 = archive.objects.inner[2].header.as_header();
    writer.push_empty(header_2).unwrap();

    // test/sub/b (file)
    let header_3 = archive.objects.inner[3].header.as_header();
    writer.push_file(Cursor::new(b_assert.as_slice()), header_3).unwrap();

    writer.write().unwrap();

    let mut first = vec![];
    File::open(&og_path).unwrap().read_to_end(&mut first).unwrap();
    let mut second = vec![];
    File::open(&new_path).unwrap().read_to_end(&mut second).unwrap();

    assert_eq!(first, second);
}

/// Read a CRC (070702) cpio archive created by system cpio, then round-trip it through librarium
#[test_log::test]
fn test_crc_compat() {
    const TEST_PATH: &str = ".";
    let filepath = "test-assets/test_crc_compat/crc-compat.cpio";
    let og_path = format!("{TEST_PATH}/{filepath}");
    let new_path = format!("{TEST_PATH}/test-assets/test_crc_compat/bytes.cpio");

    let mut config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("../test-assets.toml");
    let file_content = std::fs::read_to_string(config_path).unwrap();
    let parsed: TestAsset = toml::from_str(&file_content).unwrap();
    let assets = parsed.values();

    dl_test_files_backoff(&assets, TEST_PATH, Duration::from_secs(1)).unwrap();

    let mut file = BufReader::new(File::open(&og_path).unwrap());
    let mut archive: ArchiveReader<NewcCrcHeader> =
        ArchiveReader::from_reader_with_offset(&mut file, 0).unwrap();

    // Verify expected file data
    let a_assert = b"a\n";
    let b_assert = b"b\n";

    let mut a = Cursor::new(Vec::new());
    archive.extract_by_name("test/a", &mut a).unwrap();
    assert_eq!(a.into_inner(), a_assert);

    let mut b = Cursor::new(Vec::new());
    archive.extract_by_name("test/sub/b", &mut b).unwrap();
    assert_eq!(b.into_inner(), b_assert);

    // Round-trip: write it back out and compare bytes
    let out_file = File::create(&new_path).unwrap();
    let mut writer = ArchiveWriter::<NewcCrcHeader>::new(Box::new(out_file));

    // test (dir)
    let header_0 = archive.objects.inner[0].header.as_header();
    writer.push_empty(header_0).unwrap();

    // test/a (file)
    let header_1 = archive.objects.inner[1].header.as_header();
    writer.push_file(Cursor::new(a_assert.as_slice()), header_1).unwrap();

    // test/sub (dir)
    let header_2 = archive.objects.inner[2].header.as_header();
    writer.push_empty(header_2).unwrap();

    // test/sub/b (file)
    let header_3 = archive.objects.inner[3].header.as_header();
    writer.push_file(Cursor::new(b_assert.as_slice()), header_3).unwrap();

    writer.write().unwrap();

    let mut first = vec![];
    File::open(&og_path).unwrap().read_to_end(&mut first).unwrap();
    let mut second = vec![];
    File::open(&new_path).unwrap().read_to_end(&mut second).unwrap();

    assert_eq!(first, second);
}
