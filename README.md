Librarium
===========

[<img alt="github" src="https://img.shields.io/badge/github-wcampbell0x2a/librarium-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/wcampbell0x2a/librarium)
[<img alt="crates.io" src="https://img.shields.io/crates/v/librarium.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/librarium)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-librarium-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/librarium)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/wcampbell0x2a/librarium/main.yml?branch=master&style=for-the-badge" height="20">](https://github.com/wcampbell0x2a/librarium/actions?query=branch%3Amaster)
[<img alt="Codecov" src="https://img.shields.io/codecov/c/github/wcampbell0x2a/librarium?style=for-the-badge" height="20">](https://app.codecov.io/gh/wcampbell0x2a/librarium)

Library and binaries for the reading, creating, and modification of [CPIO](https://en.wikipedia.org/wiki/Cpio) archives.

## Library
*Compiler support: requires rustc 1.72.1+*

Add the following to your `Cargo.toml` file:
```toml
[dependencies]
librarium = "0.2.0"
```

### Read
```rust
let mut file = File::open("archive.cpio").unwrap();
let mut archive = ArchiveReader::from_reader(&mut file).unwrap();

// extract bytes from all in archive
for object in &archive.objects.inner {
    let mut out = OpenOptions::new().write(true).create(true).open(object.name).unwrap();
    archive.reader.extract_data(object, &mut out).unwrap();
}
```

### Write
```rust
let file = File::create(&new_path).unwrap();
let mut writer = ArchiveWriter::new(Box::new(file));

// A
let a_data = "a\n".as_bytes();
let a_header = Header {
    ino: 9119860,
    mode: 33188,
    uid: 1000,
    gid: 1000,
    nlink: 1,
    mtime: 1703901104,
    devmajor: 0,
    devminor: 38,
    rdevmajor: 0,
    rdevminor: 0,
};
writer.push_file(Cursor::new(a_data), CString::new("cpio-in/a").unwrap(), a_header).unwrap();

// write to archive
writer.write().unwrap();
```

## Binaries
*Compiler support: requires rustc 1.77+*

### uncpio-librarium
```
tool to extract and list cpio filesystems

Usage: uncpio [OPTIONS] [ARCHIVE]

Arguments:
  [ARCHIVE]  CPIO path

Options:
  -o, --offset <BYTES>   Skip BYTES at the start of FILESYSTEM [default: 0]
  -d, --dest <PATHNAME>  Extract to [PATHNAME] [default: out]
  -h, --help             Print help
  -V, --version          Print version
```

