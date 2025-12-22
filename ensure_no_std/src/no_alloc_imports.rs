use librarium::{NewcHeader, CpioHeader};
use deku::prelude::*;

const ARCHIVE_DATA: &[u8] = include_bytes!("../../test_cpio/sample.cpio");

pub fn test_archive_read() {
    let offset = 0;
    let (rest, header) = NewcHeader::from_bytes((ARCHIVE_DATA, offset)).unwrap();

    let name = header.name();
    assert!(name.len() > 0, "Header should have a non-empty name");

    let _mode = header.mode();
    let _ino = header.ino();
    let _filesize = header.filesize();

    assert!(rest.1 > offset, "Parsing should advance offset");

    let common = header.as_header();
    assert!(common.name == name, "Name should match in common header");
}

pub fn test_header_fields() {
    let (rest, header) = NewcHeader::from_bytes((ARCHIVE_DATA, 0)).unwrap();

    let _ino = header.ino();
    let _mode = header.mode();
    let _uid = header.uid();
    let _gid = header.gid();
    let _nlink = header.nlink();
    let _mtime = header.mtime();
    let _filesize = header.filesize();
    let _devmajor = header.devmajor();
    let _devminor = header.devminor();
    let _rdevmajor = header.rdevmajor();
    let _rdevminor = header.rdevminor();
    let _namesize = header.namesize();
    let _check = header.check();
    let name = header.name();

    assert!(name.len() > 0);

    let offset = rest.1;
    assert!(offset > 0);
}
