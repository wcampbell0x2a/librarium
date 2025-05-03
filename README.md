Librarium
===========

[<img alt="github" src="https://img.shields.io/badge/github-wcampbell0x2a/librarium-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/wcampbell0x2a/librarium)
[<img alt="crates.io" src="https://img.shields.io/crates/v/librarium.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/librarium)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-librarium-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/librarium)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/wcampbell0x2a/librarium/main.yml?branch=master&style=for-the-badge" height="20">](https://github.com/wcampbell0x2a/librarium/actions?query=branch%3Amaster)
[<img alt="Codecov" src="https://img.shields.io/codecov/c/github/wcampbell0x2a/librarium?style=for-the-badge" height="20">](https://app.codecov.io/gh/wcampbell0x2a/librarium)

Library and binaries for the reading, creating, and modification of [cpio](https://en.wikipedia.org/wiki/Cpio) archives.

## Library
*Compiler support: requires rustc 1.84+*

Add the following to your `Cargo.toml` file:
```toml
[dependencies]
librarium = "0.4.0"
```

### Read
```rust
use std::ffi::CString;
use std::io::Cursor;
use std::fs::{File, OpenOptions};
use librarium::{Header, ArchiveReader, NewcHeader, CpioReader, CpioHeader};

let mut file = File::open("archive.cpio").unwrap();
let mut archive = ArchiveReader::<NewcHeader>::from_reader_with_offset(&mut file, 0).unwrap();

// extract bytes from all in archive
for object in &archive.objects.inner {
    let mut out = OpenOptions::new()
        .write(true)
        .create(true)
        .open(object.header.as_header().name)
        .unwrap();
    archive.reader.extract_data(object, &mut out).unwrap();
}
```

### Write
```rust
use std::ffi::CString;
use std::io::Cursor;
use std::fs::File;
use librarium::{Header, ArchiveWriter, NewcHeader};

let file = File::create("archive.cpio").unwrap();
let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(file));

// A
let a_data = "a\n".as_bytes();
let a_header = Header { name: "a".to_string(), ..Header::default()};
writer.push_file(Cursor::new(a_data), a_header).unwrap();

// write to archive
writer.write().unwrap();
```

## Binaries
*Compiler support: requires rustc 1.84+*

These are currently under development and are missing features, MR's welcome!

To install, run `cargo install librarium-cli --locked`, or download from the
[latest github release](https://github.com/wcampbell0x2a/librarium/releases/latest).

See ``--help`` for more information.

### uncpio-librarium
```text
tool to extract and list cpio filesystems

Usage: uncpio-librarium [OPTIONS] <ARCHIVE> <FORMAT>

Arguments:
  <ARCHIVE>  cpio path
  <FORMAT>   [possible values: odc, newc]

Options:
  -o, --offset <BYTES>   Skip BYTES at the start of FILESYSTEM [default: 0]
  -d, --dest <PATHNAME>  Extract to [PATHNAME] [default: out]
  -h, --help             Print help
  -V, --version          Print version
```

