use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;

use librarium::CpioHeader;
use librarium::NewcHeader;
use librarium::OdcHeader;
use librarium::{ArchiveReader, ArchiveWriter};
use test_assets::TestAssetDef;

// cpio -o -H newc > cpio-in.cpio
#[test_log::test]
fn test_simple_in_out_newc_files() {
    const TEST_PATH: &str = "test-assets/test_simple_in_out_newc/";
    let filepath = "cpio-in.cpio";
    let og_path = format!("{TEST_PATH}/{filepath}");
    let new_path = format!("{TEST_PATH}/bytes.squashfs");

    const FILE_NAME: &str = "cpio-in.cpio";
    let asset_defs = [TestAssetDef {
        filename: FILE_NAME.to_string(),
        hash: "39c7a5817e62fa451fb57638137bcfdd6add7fe706394d42c60c5c9314dc6cf2".to_string(),
        url: format!(
            "https://wcampbell.dev/cpio/testing/test_simple_in_out_newc_files/{FILE_NAME}"
        ),
    }];

    test_assets::download_test_files(&asset_defs, TEST_PATH, true).unwrap();

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
    const TEST_PATH: &str = "test-assets/test_simple_in_out_odc/";
    let filepath = "odc.cpio";
    let og_path = format!("{TEST_PATH}/{filepath}");
    let new_path = format!("{TEST_PATH}/bytes.squashfs");

    const FILE_NAME: &str = "odc.cpio";
    let asset_defs = [TestAssetDef {
        filename: FILE_NAME.to_string(),
        hash: "4cee2af1ecfec5ba14eabfb5821716782e79961d369a4279546fbff1be6d7bef".to_string(),
        url: format!("https://wcampbell.dev/cpio/testing/test_simple_in_out_odc_files/{FILE_NAME}"),
    }];

    test_assets::download_test_files(&asset_defs, TEST_PATH, true).unwrap();

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
