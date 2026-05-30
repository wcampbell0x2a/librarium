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

#[cfg(unix)]
mod metadata_conversion {
    use std::os::unix::fs::MetadataExt;

    use librarium::Header;

    /// Verify that a temp file's metadata converts to a Header with matching fields.
    #[test]
    fn file_metadata_to_header() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let metadata = tmp.as_file().metadata().unwrap();
        let header = Header::try_from(&metadata).unwrap();

        assert_eq!(header.ino, metadata.ino() as u32);
        assert_eq!(header.mode, metadata.mode() as u32);
        assert_eq!(header.uid, metadata.uid() as u32);
        assert_eq!(header.gid, metadata.gid() as u32);
        assert_eq!(header.nlink, metadata.nlink() as u32);
        assert_eq!(header.mtime, metadata.mtime() as u32);
        assert_eq!(header.dev, Some(metadata.dev() as u32));
        assert_eq!(header.rdev, Some(metadata.rdev() as u32));
    }

    /// Verify that a directory's metadata preserves the directory mode bits.
    #[test]
    fn directory_metadata_to_header() {
        let tmp = tempfile::tempdir().unwrap();
        let metadata = std::fs::metadata(tmp.path()).unwrap();
        let header = Header::try_from(&metadata).unwrap();

        // S_IFDIR = 0o040000; directory bit must be set
        assert!(header.mode & 0o040000 != 0, "expected directory mode bits to be set");
        assert_eq!(header.mode, metadata.mode() as u32);
    }

    /// Verify that name defaults to an empty string.
    #[test]
    fn name_defaults_to_empty() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let metadata = tmp.as_file().metadata().unwrap();
        let header = Header::try_from(&metadata).unwrap();

        assert_eq!(header.name, "");
    }

    /// Verify major/minor device number fields are populated.
    #[test]
    fn devmajor_devminor_populated() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let metadata = tmp.as_file().metadata().unwrap();
        let header = Header::try_from(&metadata).unwrap();

        assert!(header.devmajor.is_some());
        assert!(header.devminor.is_some());
        assert!(header.rdevmajor.is_some());
        assert!(header.rdevminor.is_some());
    }

    /// Round-trip: create a Header from metadata, write to an archive, read back,
    /// and verify the fields match.
    #[test]
    fn roundtrip_metadata_header() {
        use librarium::{ArchiveReader, ArchiveWriter, CpioHeader, NewcHeader};
        use std::io::Cursor;

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let metadata = tmp.as_file().metadata().unwrap();
        let mut header = Header::try_from(&metadata).unwrap();
        header.name = "test_file".to_string();

        let data = b"hello world";

        // Write archive to buffer
        let mut out_buf = Vec::new();
        {
            let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(Cursor::new(&mut out_buf)));
            writer.push_file(Cursor::new(data.as_slice()), header).unwrap();
            writer.write().unwrap();
        }

        // Read back
        let mut reader_cursor = Cursor::new(&out_buf);
        let archive = ArchiveReader::<NewcHeader>::from_reader(&mut reader_cursor).unwrap();

        // First object is our file; last is TRAILER
        let read_header = archive.objects.inner[0].header.as_header();
        assert_eq!(read_header.name, "test_file");
        assert_eq!(read_header.ino, metadata.ino() as u32);
        assert_eq!(read_header.mode, metadata.mode() as u32);
        assert_eq!(read_header.uid, metadata.uid() as u32);
        assert_eq!(read_header.gid, metadata.gid() as u32);
        assert_eq!(read_header.nlink, metadata.nlink() as u32);
        assert_eq!(read_header.mtime, metadata.mtime() as u32);
    }
}
