use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::time::Duration;

use librarium::CpioHeader;
use librarium::NewcHeader;
use librarium::OdcHeader;
use librarium::{ArchiveReader, ArchiveWriter};
use test_assets_ureq::{TestAsset, dl_test_files_backoff};

// cpio -o -H newc > cpio-in.cpio
#[test_log::test]
fn test_simple_in_out_newc_files() {
    const TEST_PATH: &str = ".";
    let filepath = "test-assets/test_simple_in_out_newc/cpio-in.cpio";
    let og_path = format!("{TEST_PATH}/{filepath}");
    let new_path = format!("{TEST_PATH}/test-assets/test_simple_in_out_newc/bytes.squashfs");

    let mut config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("../test-assets.toml");
    let file_content = std::fs::read_to_string(config_path).unwrap();
    let parsed: TestAsset = toml::from_str(&file_content).unwrap();
    let assets = parsed.values();

    dl_test_files_backoff(&assets, TEST_PATH, Duration::from_secs(1)).unwrap();

    let mut file = BufReader::new(File::open(&og_path).unwrap());
    let mut archive: ArchiveReader<NewcHeader> =
        ArchiveReader::from_reader_with_offset(&mut file, 0).unwrap();

    let a_assert = "a\n".as_bytes();
    let b_assert = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n".as_bytes();
    let c_assert = "cccccccccccccccccccccccccccccc\ncccc\nc\nc\nc\nc\nc\n".as_bytes();

    let mut a = Cursor::new(Vec::new());
    let _header_a = archive.extract_by_name("cpio-in/a", &mut a).unwrap().unwrap();
    assert_eq!(a.into_inner(), a_assert);

    let mut b = Cursor::new(Vec::new());
    let _header_b = archive.extract_by_name("cpio-in/b", &mut b).unwrap().unwrap();
    assert_eq!(b.into_inner(), b_assert);

    let mut c = Cursor::new(Vec::new());
    let _header_c = archive.extract_by_name("cpio-in/c", &mut c).unwrap().unwrap();
    assert_eq!(c.into_inner(), c_assert);

    let file = File::create(&new_path).unwrap();
    let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(file));

    for objects in &archive.objects.inner {
        println!("Z: {:02x?}", objects.header.as_header());
    }
    // a
    let header_a = archive.objects.inner[0].header.as_header();
    writer.push_file(Cursor::new(a_assert), header_a).unwrap();

    // b
    let header_b = archive.objects.inner[1].header.as_header();
    writer.push_file(Cursor::new(b_assert), header_b).unwrap();

    // c
    let header_c = archive.objects.inner[2].header.as_header();
    writer.push_file(Cursor::new(c_assert), header_c).unwrap();

    writer.write().unwrap();

    let mut og_file = File::open(&og_path).unwrap();
    let mut new_file = File::open(&new_path).unwrap();

    let mut first = vec![];
    og_file.read_to_end(&mut first).unwrap();
    let mut second = vec![];
    new_file.read_to_end(&mut second).unwrap();

    assert_eq!(first, second);
}

// cpio -o -H newc > cpio-in.cpio
#[test_log::test]
fn test_simple_in_out_odc_files() {
    const TEST_PATH: &str = ".";
    let filepath = "test-assets/test_simple_in_out_odc/odc.cpio";
    let og_path = format!("{TEST_PATH}/{filepath}");
    let new_path = format!("{TEST_PATH}/test-assets/test_simple_in_out_odc/bytes.squashfs");

    let mut config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("../test-assets.toml");
    let file_content = std::fs::read_to_string(config_path).unwrap();
    let parsed: TestAsset = toml::from_str(&file_content).unwrap();
    let assets = parsed.values();

    dl_test_files_backoff(&assets, TEST_PATH, Duration::from_secs(1)).unwrap();

    let mut file = BufReader::new(File::open(&og_path).unwrap());
    let mut archive: ArchiveReader<OdcHeader> =
        ArchiveReader::from_reader_with_offset(&mut file, 0).unwrap();

    let a_assert = "a\n".as_bytes();
    let b_assert = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n".as_bytes();
    let c_assert = "cccccccccccccccccccccccccccccc\ncccc\nc\nc\nc\nc\nc\n".as_bytes();

    let mut a = Cursor::new(Vec::new());
    archive.extract_by_name("cpio-in/a", &mut a).unwrap();
    assert_eq!(a.into_inner(), a_assert);

    let mut b = Cursor::new(Vec::new());
    archive.extract_by_name("cpio-in/b", &mut b).unwrap();
    assert_eq!(b.into_inner(), b_assert);

    let mut c = Cursor::new(Vec::new());
    archive.extract_by_name("cpio-in/c", &mut c).unwrap();
    assert_eq!(c.into_inner(), c_assert);

    let file = File::create(&new_path).unwrap();
    let mut writer: ArchiveWriter<OdcHeader> = ArchiveWriter::new(Box::new(file));

    // .
    let header_dot = archive.objects.inner[0].header.as_header();
    writer.push_empty(header_dot).unwrap();

    // cpio-in
    let header_dir = archive.objects.inner[1].header.as_header();
    writer.push_empty(header_dir).unwrap();

    // a
    let header_a = archive.objects.inner[2].header.as_header();
    writer.push_file(Cursor::new(a_assert), header_a).unwrap();

    // b
    let header_b = archive.objects.inner[3].header.as_header();
    writer.push_file(Cursor::new(b_assert), header_b).unwrap();

    // c
    let header_c = archive.objects.inner[4].header.as_header();
    writer.push_file(Cursor::new(c_assert), header_c).unwrap();

    writer.write().unwrap();

    let mut og_file = File::open(&og_path).unwrap();
    let mut new_file = File::open(&new_path).unwrap();

    let mut first = vec![];
    og_file.read_to_end(&mut first).unwrap();
    let mut second = vec![];
    new_file.read_to_end(&mut second).unwrap();

    assert_eq!(first, second);
}
