# Librarium
Library and binaries for the reading, creating, and modification of [CPIO](https://en.wikipedia.org/wiki/Cpio) archives.

## Usage
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

## Read
```rust
let mut file = File::open("archive.cpio").unwrap();
let mut archive = ArchiveReader::from_reader(&mut file).unwrap();

// extract bytes from all in archive
for object in &archive.objects.inner {
    let mut out = OpenOptions::new().write(true).create(true).open(object.name).unwrap();
    archive.reader.extract_data(object, &mut out).unwrap();
}
```

## Write
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
