//! Writing and reading entries without holding their data in memory whole

use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use librarium::{
    ArchiveReader, ArchiveWriter, CpioError, CpioHeader, CpioReader, Header, NewcCrcHeader,
    NewcHeader, OdcHeader,
};
use proptest::prelude::*;

/// Bytes moved per read inside the library. Sizes near it test the chunk edges.
const CHUNK_LEN: usize = 64 * 1024;

/// A file of `len` zero bytes that takes no memory
struct Zeros {
    len: u64,
    pos: u64,
}

impl Zeros {
    fn new(len: u64) -> Self {
        Self { len, pos: 0 }
    }
}

impl Read for Zeros {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = usize::try_from(self.len - self.pos).map_or(buf.len(), |r| r.min(buf.len()));
        buf[..n].fill(0);
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for Zeros {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::End(d) => self.len.checked_add_signed(d).unwrap(),
            SeekFrom::Current(d) => self.pos.checked_add_signed(d).unwrap(),
        };
        Ok(self.pos)
    }
}

/// Reports `len` bytes on seek, but ends early on read, as a file that shrinks after
/// `push_file` measured it
struct Shrinks {
    len: u64,
    inner: Cursor<Vec<u8>>,
}

impl Read for Shrinks {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for Shrinks {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::End(0) => Ok(self.len),
            other => self.inner.seek(other),
        }
    }
}

/// Counts the bytes written and drops them
#[derive(Default)]
struct CountingSink {
    len: u64,
}

impl Write for CountingSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.len += buf.len() as u64;
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for CountingSink {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        Ok(self.len)
    }
}

fn header(name: &str) -> Header {
    Header { name: name.to_string(), mode: 0o100644, nlink: 1, ..Header::default() }
}

/// Write `entries` with format `C`, read them back, and return the data of each entry
fn round_trip<C: CpioHeader + std::fmt::Debug>(entries: &[(String, Vec<u8>)]) -> Vec<Vec<u8>> {
    let mut image = Vec::new();
    {
        let mut writer = ArchiveWriter::<C>::new(Box::new(Cursor::new(&mut image)));
        for (name, data) in entries {
            writer.push_file(Cursor::new(data.clone()), header(name)).unwrap();
        }
        writer.write().unwrap();
    }

    let mut archive = ArchiveReader::<C>::from_reader(Cursor::new(image)).unwrap();
    let mut found = Vec::new();
    for object in &archive.objects.inner {
        if object.header.name() == "TRAILER!!!" {
            continue;
        }
        let mut out = Cursor::new(Vec::new());
        archive.reader.extract_data(object, &mut out).unwrap();
        found.push(out.into_inner());
    }

    found
}

#[test]
fn round_trip_sizes_at_chunk_edges() {
    let sizes = [0, 1, CHUNK_LEN - 1, CHUNK_LEN, CHUNK_LEN + 1, 3 * CHUNK_LEN + 7];
    let entries: Vec<_> = sizes
        .iter()
        .enumerate()
        .map(|(i, &len)| (format!("f{i}"), (0..len).map(|b| (b % 251) as u8).collect::<Vec<u8>>()))
        .collect();
    let expected: Vec<_> = entries.iter().map(|(_, d)| d.clone()).collect();

    assert_eq!(round_trip::<NewcHeader>(&entries), expected);
    assert_eq!(round_trip::<NewcCrcHeader>(&entries), expected);
    assert_eq!(round_trip::<OdcHeader>(&entries), expected);
}

#[test]
fn crc_checksum_is_byte_sum() {
    let data: Vec<u8> = (0..3 * CHUNK_LEN + 7).map(|b| (b % 251) as u8).collect();
    let expected = data.iter().fold(0u32, |acc, &b| acc.wrapping_add(u32::from(b)));

    let mut image = Vec::new();
    {
        let mut writer = ArchiveWriter::<NewcCrcHeader>::new(Box::new(Cursor::new(&mut image)));
        writer.push_file(Cursor::new(data), header("f")).unwrap();
        writer.write().unwrap();
    }
    let archive = ArchiveReader::<NewcCrcHeader>::from_reader(Cursor::new(image)).unwrap();

    assert_eq!(archive.objects.inner[0].header.check(), Some(expected));
}

#[test]
fn newc_has_no_checksum() {
    let mut image = Vec::new();
    {
        let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(Cursor::new(&mut image)));
        writer.push_file(Cursor::new(vec![0xff; 16]), header("f")).unwrap();
        writer.write().unwrap();
    }
    let archive = ArchiveReader::<NewcHeader>::from_reader(Cursor::new(image)).unwrap();

    assert_eq!(archive.objects.inner[0].header.check(), Some(0));
}

#[test]
fn image_is_padded_to_pad_len() {
    for pad_len in [0, 1, 512, ArchiveWriter::<NewcHeader>::DEFAULT_PAD_LEN, 4096] {
        let mut image = Vec::new();
        {
            let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(Cursor::new(&mut image)));
            writer.set_pad_len(pad_len);
            writer.push_file(Cursor::new(vec![1; 100]), header("f")).unwrap();
            writer.write().unwrap();
        }
        if pad_len > 0 {
            assert_eq!(image.len() % pad_len as usize, 0, "pad_len {pad_len}");
        }
        assert!(ArchiveReader::<NewcHeader>::from_reader(Cursor::new(image)).is_ok());
    }
}

/// An image larger than `u32::MAX` bytes. Earlier releases panicked while they padded it.
#[test]
fn image_larger_than_4_gib() {
    let entry_len = u64::from(u32::MAX) - 1024;
    let mut sink = CountingSink::default();
    {
        let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(&mut sink));
        writer.push_file(Zeros::new(entry_len), header("a")).unwrap();
        writer.push_file(Zeros::new(entry_len), header("b")).unwrap();
        writer.write().unwrap();
    }

    let pad_len = u64::from(ArchiveWriter::<NewcHeader>::DEFAULT_PAD_LEN);
    assert!(sink.len > 2 * entry_len);
    assert_eq!(sink.len % pad_len, 0);
}

#[test]
fn file_larger_than_4_gib_is_an_error() {
    let mut sink = CountingSink::default();
    let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(&mut sink));
    let len = u64::from(u32::MAX) + 1;

    let err = writer.push_file(Zeros::new(len), header("big")).unwrap_err();

    assert!(matches!(err, CpioError::FileTooLarge(n) if n == len), "{err:?}");
}

#[test]
fn file_that_shrinks_before_write_is_an_error() {
    let mut sink = CountingSink::default();
    let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(&mut sink));
    let shrinks = Shrinks { len: 100, inner: Cursor::new(vec![0; 10]) };
    writer.push_file(shrinks, header("f")).unwrap();

    assert!(writer.write().is_err());
}

#[test]
fn extract_from_truncated_archive_is_an_error() {
    let mut image = Vec::new();
    {
        let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(Cursor::new(&mut image)));
        writer.push_file(Cursor::new(vec![7; 1000]), header("f")).unwrap();
        writer.write().unwrap();
    }
    let archive = ArchiveReader::<NewcHeader>::from_reader(Cursor::new(image.clone())).unwrap();
    let object = &archive.objects.inner[0];

    // Same entry table, but the data stops part way through the entry.
    let mut truncated = Cursor::new(image[..500].to_vec());
    let mut out = Cursor::new(Vec::new());

    assert!(truncated.extract_data(object, &mut out).is_err());
}

#[test]
fn extract_entry_not_read_from_archive_is_an_error() {
    let object =
        librarium::Object::new(NewcHeader::from_header(header("f"), 0), librarium::Data::Empty);
    let mut out = Cursor::new(Vec::new());

    let err = Cursor::new(Vec::<u8>::new()).extract_data(&object, &mut out).unwrap_err();

    assert!(matches!(err, CpioError::NoData), "{err:?}");
}

fn entries_strategy() -> impl Strategy<Value = Vec<(String, Vec<u8>)>> {
    prop::collection::vec(
        ("[a-z0-9/._-]{1,40}", prop::collection::vec(any::<u8>(), 0..2 * CHUNK_LEN + 3)),
        0..8,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn round_trip_any_entries(entries in entries_strategy()) {
        let expected: Vec<_> = entries.iter().map(|(_, d)| d.clone()).collect();

        prop_assert_eq!(&round_trip::<NewcHeader>(&entries), &expected);
        prop_assert_eq!(&round_trip::<NewcCrcHeader>(&entries), &expected);
        prop_assert_eq!(&round_trip::<OdcHeader>(&entries), &expected);
    }
}
