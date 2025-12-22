extern crate alloc;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use deku::prelude::*;
use no_std_io2::io::Cursor;
use librarium::{NewcHeader, OdcHeader, CpioHeader, Header, ArchiveReader, ArchiveWriter};

const ARCHIVE_DATA: &[u8] = include_bytes!("../../test_cpio/sample.cpio");

pub fn test_archive_reader() {
    let cursor = Cursor::new(ARCHIVE_DATA);
    let archive = ArchiveReader::<NewcHeader>::from_reader(cursor).unwrap();

    assert!(archive.objects.inner.len() > 0, "Archive should contain objects");

    let first_object = &archive.objects.inner[0];
    let name = first_object.header.name();
    assert!(name.len() > 0, "First object should have a name");
}

pub fn test_newc_header_creation() {
    let header = Header {
        ino: 42,
        mode: 0o100644,
        uid: 1000,
        gid: 1000,
        nlink: 1,
        mtime: 1234567890,
        dev: None,
        devmajor: Some(0),
        devminor: Some(0),
        rdev: None,
        rdevmajor: Some(0),
        rdevminor: Some(0),
        name: "test.txt".to_string(),
    };

    let newc = NewcHeader::from_header(header, 0);

    assert!(newc.ino() == 42);
    assert!(newc.mode() == 0o100644);
    assert!(newc.name() == "test.txt");
    assert!(newc.filesize() == 0);

    let bytes = newc.to_bytes().unwrap();
    assert!(bytes.len() > 0);

    let (_rest, parsed) = NewcHeader::from_bytes((bytes.as_ref(), 0)).unwrap();
    assert!(parsed.name() == "test.txt");
    assert!(parsed.ino() == 42);
}

pub fn test_odc_header_creation() {
    let header = Header {
        ino: 100,
        mode: 0o100755,
        uid: 0,
        gid: 0,
        nlink: 1,
        mtime: 0,
        dev: Some(1),
        devmajor: None,
        devminor: None,
        rdev: Some(0),
        rdevmajor: None,
        rdevminor: None,
        name: "script.sh".to_string(),
    };

    let odc = OdcHeader::from_header(header, 10);

    assert!(odc.ino() == 100);
    assert!(odc.mode() == 0o100755);
    assert!(odc.name() == "script.sh");
    assert!(odc.filesize() == 10);

    let bytes = odc.to_bytes().unwrap();
    assert!(bytes.len() > 0);

    let (_rest, parsed) = OdcHeader::from_bytes((bytes.as_ref(), 0)).unwrap();
    assert!(parsed.name() == "script.sh");
    assert!(parsed.filesize() == 10);
}

pub fn test_archive_writer() {
    let buffer: Vec<u8> = Vec::new();
    let cursor = Cursor::new(buffer);
    let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(cursor));

    let file_header = Header {
        name: "hello.txt".to_string(),
        mode: 0o100644,
        ..Header::default()
    };

    let data = b"Hello, embedded world!";
    let data_cursor = Cursor::new(data.as_ref());
    writer.push_file(data_cursor, file_header).unwrap();

    writer.write().unwrap();
}
