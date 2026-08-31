/*!
Reader and Writer library for cpio archives

## Supported Formats

| Format | Magic | Read | Write |
|--------|-------|------|-------|
| odc (Old ASCII) | `070707` | Yes | Yes |
| newc (New ASCII / SVR4) | `070701` | Yes | Yes |
| newc-crc (New CRC) | `070702` | Yes | Yes |

### Read
```rust, no_run
# use std::ffi::CString;
# use std::io::Cursor;
# use librarium::{Header, ArchiveReader, NewcHeader, CpioReader, CpioHeader};
# use std::fs::{File, OpenOptions};
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
```rust, no_run
# use std::ffi::CString;
# use std::io::Cursor;
# use librarium::{Header, ArchiveWriter, NewcHeader};
# use std::fs::File;
let file = File::create("archive.cpio").unwrap();
let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(file));

// A
let a_data = "a\n".as_bytes();
let a_header = Header { name: "a".to_string(), ..Header::default()};
writer.push_file(Cursor::new(a_data), a_header).unwrap();

// write to archive
writer.write().unwrap();
```

# Features
*/
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
type _ReadmeTest = ();

use core::fmt::Debug;

use no_std_io2::io::{Cursor, Read, Seek, SeekFrom, Write};

use deku::DekuError;
use deku::prelude::*;
use deku::writer::Writer;

const TRAILER: &str = "TRAILER!!!";

/// Number of bytes in the magic field. Every supported format uses six bytes.
pub(crate) const MAGIC_SIZE_BYTES: usize = <[u8; 6]>::SIZE_BYTES.unwrap();

/// Trait for common cpio header operations.
pub mod cpio_header;
pub use cpio_header::CpioHeader;

/// Error types returned by this library.
pub mod error;
pub use error::CpioError;

/// Composable `Read + Seek` trait and helpers.
pub mod read_seek;
pub use read_seek::ReadSeek;
pub(crate) use read_seek::ReaderWithOffset;

/// Navigation of concatenated cpio archives.
pub mod segment;
pub use segment::{SegmentFormat, next_segment_offset, segment_format};

/// New ASCII (SVR4) cpio header formats (`070701` and `070702`).
pub mod newc;
pub use newc::{NewcCrcHeader, NewcHeader};
/// Old ASCII (odc) cpio header format (`070707`).
pub mod odc;
pub use odc::OdcHeader;

pub(crate) mod mut_writer;
pub(crate) use mut_writer::MutWriter;

impl<T: ReadSeek> CpioReader for T {}
/// Extract data from cpio Archive
pub trait CpioReader: ReadSeek {
    /// Extract file data from an [`Object`] and write it to `writer`.
    fn extract_data<W, C: CpioHeader>(
        &mut self,
        object: &Object<C>,
        writer: &mut W,
    ) -> Result<(), CpioError>
    where
        W: Write + Seek,
    {
        // found the file, seek forward
        if let Data::Offset(offset) = object.data {
            self.seek(SeekFrom::Start(offset)).unwrap();
            let mut buf = vec![0; object.header.filesize() as usize];
            self.read_exact(&mut buf).unwrap();
            writer.write_all(&buf)?;
            Ok(())
        } else {
            panic!("no offset! TODO improve this");
        }
    }
}

/// Reader and Writer of data
pub enum Data {
    /// On read: Save current stream_position() as `Offset`, seek `header.filesize`
    /// This will be used to seek this position if we want to extract *just* this file
    Offset(u64),
    /// On write: Write `Reader` to write buffer
    Reader(Box<dyn ReadSeek>),
    /// On write: zero sized file
    Empty,
}

impl DekuReader<'_, u32> for Data {
    fn from_reader_with_ctx<R: Read + Seek>(
        reader: &mut Reader<R>,
        filesize: u32,
    ) -> Result<Data, DekuError> {
        let reader = reader.as_mut();

        // Save the current offset, this is where the file exists for reading later
        #[allow(clippy::seek_from_current)]
        let current_pos = reader.seek(SeekFrom::Current(0)).unwrap();

        // Seek past that file
        let position = filesize as i64;
        let _ = reader.seek(SeekFrom::Current(position));

        Ok(Self::Offset(current_pos))
    }
}

impl MutWriter<u32> for Data {
    fn to_mutwriter<W: Write + Seek>(
        &mut self,
        writer: &mut Writer<W>,
        _: u32,
    ) -> Result<(), DekuError> {
        match self {
            Self::Reader(reader) => {
                // read from reader
                let mut data = vec![];
                reader.read_to_end(&mut data).unwrap();

                // write to deku
                data.to_writer(writer, ())?;
            }
            Self::Empty => (),
            _ => {
                panic!("ah");
            }
        }

        Ok(())
    }
}

/// All objects in archive
#[derive(DekuRead)]
pub struct Objects<C: CpioHeader> {
    /// The list of parsed archive entries, including the trailing sentinel.
    #[deku(until = "Self::is_last")]
    pub inner: Vec<Object<C>>,
}

impl<C: CpioHeader> MutWriter for Objects<C> {
    /// Write all entries
    fn to_mutwriter<W: Write + Seek>(
        &mut self,
        deku_writer: &mut Writer<W>,
        _: (),
    ) -> core::result::Result<(), DekuError> {
        for i in &mut self.inner {
            i.to_mutwriter(deku_writer, ())?;
        }
        Ok(())
    }
}

impl<C: CpioHeader> Objects<C> {
    /// Is Trailer entry
    fn is_last(last_object: &Object<C>) -> bool {
        last_object.header.name().as_bytes() == TRAILER.as_bytes()
    }
}

/// Read cpio Archive and extract data
///
/// # Example
/// Read `archive.cpio` and extract data.
/// ```rust, no_run
/// # use std::ffi::CString;
/// # use std::io::Cursor;
/// # use librarium::{Header, ArchiveReader, NewcHeader, CpioReader, CpioHeader};
/// # use std::fs::{File, OpenOptions};
/// let mut file = File::open("archive.cpio").unwrap();
/// let mut archive = ArchiveReader::<NewcHeader>::from_reader_with_offset(&mut file, 0).unwrap();
///
/// // extract bytes from all in archive
/// for object in &archive.objects.inner {
///    let mut out = OpenOptions::new()
///        .write(true)
///        .create(true)
///        .open(object.header.as_header().name)
///        .unwrap();
///     archive.reader.extract_data(object, &mut out).unwrap();
/// }
/// ```
pub struct ArchiveReader<'b, C: CpioHeader> {
    /// Underlying reader for data extraction.
    pub reader: Box<dyn ReadSeek + 'b>,
    /// Parsed archive entries.
    pub objects: Objects<C>,
    /// Offset of this archive from the start of the underlying stream.
    base_offset: u64,
}

impl<'b, C: CpioHeader> ArchiveReader<'b, C> {
    /// Parse an archive from a reader, starting at offset 0.
    pub fn from_reader(reader: impl ReadSeek + 'b) -> Result<Self, CpioError> {
        Self::from_reader_with_offset(reader, 0)
    }

    /// Parse an archive from a reader, starting at the given byte `offset`.
    pub fn from_reader_with_offset(
        reader: impl ReadSeek + 'b,
        offset: u64,
    ) -> Result<Self, CpioError> {
        let mut reader: Box<dyn ReadSeek> = if offset == 0 {
            // The reader can be at any position, so rewind it to the start.
            let mut reader = reader;
            reader.seek(SeekFrom::Start(0))?;
            Box::new(reader)
        } else {
            let reader = ReaderWithOffset::new(reader, offset)?;
            Box::new(reader)
        };
        let (_, objects) = Objects::from_reader((&mut reader, 0))?;
        Ok(Self { reader, objects, base_offset: offset })
    }

    /// Offset of the first byte after the last entry, from the start of the
    /// underlying stream.
    ///
    /// A file can hold more than one archive, one after the other. This offset
    /// is the start of the padding that follows this archive. Give it to
    /// [`next_segment_offset`] to find the next archive.
    ///
    /// # Example
    /// Read every cpio archive in a concatenated file, such as an initramfs.
    /// ```rust, no_run
    /// # use std::fs::File;
    /// # use librarium::{ArchiveReader, CpioHeader, NewcHeader, SegmentFormat, next_segment_offset, segment_format};
    /// let mut file = File::open("initramfs.img").unwrap();
    /// let mut offset = 0;
    /// while let Some(start) = next_segment_offset(&mut file, offset).unwrap() {
    ///     // A compressed segment gives `None`; the caller must decompress it.
    ///     if segment_format(&mut file, start).unwrap() != Some(SegmentFormat::Newc) {
    ///         break;
    ///     }
    ///     let archive = ArchiveReader::<NewcHeader>::from_reader_with_offset(&mut file, start).unwrap();
    ///     for object in &archive.objects.inner {
    ///         println!("{}", object.header.name());
    ///     }
    ///     offset = archive.end_offset();
    /// }
    /// ```
    #[must_use]
    pub fn end_offset(&self) -> u64 {
        let Some(last) = self.objects.inner.last() else {
            return self.base_offset;
        };

        // `Data::Offset` is relative to `base_offset` and points at the data of
        // the last entry, which is the `TRAILER!!!` sentinel.
        let Data::Offset(data_offset) = last.data else {
            return self.base_offset;
        };

        self.base_offset
            + data_offset
            + u64::from(last.header.filesize())
            + last.header.data_pad() as u64
    }

    /// Extract the first entry matching `name` and write its data to `writer`.
    ///
    /// Returns the [`Header`] on match, or `None` if no entry has that name.
    pub fn extract_by_name<W>(
        &mut self,
        name: &str,
        writer: &mut W,
    ) -> Result<Option<Header>, CpioError>
    where
        W: Write + Seek,
    {
        for object in &self.objects.inner {
            if name == object.header.name() {
                self.reader.extract_data(object, writer)?;
                return Ok(Some(object.header.as_header()));
            }
        }

        Ok(None)
    }
}

/// `Write` + `Seek`
pub trait WriteSeek: Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}

/// Write cpio Archive and add data
///
/// # Example
/// Create new cpio archive of Newc format and one file.
///
/// ```rust, no_run
/// # use std::ffi::CString;
/// # use std::io::Cursor;
/// # use librarium::{Header, ArchiveWriter, NewcHeader};
/// # use std::fs::File;
/// let file = File::create("archive.cpio").unwrap();
/// let mut writer = ArchiveWriter::<NewcHeader>::new(Box::new(file));
///
/// // A
/// let a_data = "a\n".as_bytes();
/// let a_header = Header { name: "a".to_string(), ..Header::default()};
/// writer.push_file(Cursor::new(a_data), a_header).unwrap();
///
/// // write to archive
/// writer.write().unwrap();
/// ```
pub struct ArchiveWriter<'a, C: CpioHeader> {
    writer: Box<dyn WriteSeek + 'a>,
    objects: Objects<C>,
    pad_len: u32,
}

impl<'a, C: CpioHeader + Debug> ArchiveWriter<'a, C> {
    /// Default image padding length
    pub const DEFAULT_PAD_LEN: u32 = 0x400;

    /// Create new `ArchiveWriter` with no objects and image padding length of
    /// `Self::DEFAULT_PAD_LEN`.
    pub fn new(writer: Box<dyn WriteSeek + 'a>) -> Self {
        Self { writer, objects: Objects { inner: vec![] }, pad_len: Self::DEFAULT_PAD_LEN }
    }

    /// Set the image padding alignment in bytes.
    pub fn set_pad_len(&mut self, pad_len: u32) {
        self.pad_len = pad_len;
    }

    /// Add data to Cpio Archive
    pub fn push_file(
        &mut self,
        mut reader: impl ReadSeek + 'a + 'static,
        header: Header,
    ) -> Result<(), CpioError> {
        // stream_len
        let filesize = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        let mut header = C::from_header(header, filesize);

        // Compute checksum (sum of all data bytes) for CRC variants
        if filesize > 0 {
            let mut buf = vec![0u8; filesize as usize];
            reader.read_exact(&mut buf)?;
            let check = buf.iter().fold(0u32, |acc, &b| acc.wrapping_add(u32::from(b)));
            header.set_check(check);
            reader.seek(SeekFrom::Start(0))?;
        }

        let object = Object::new(header, Data::Reader(Box::new(reader)));
        self.objects.inner.push(object);

        Ok(())
    }

    /// Add Empty File (Directory) to Cpio Archive
    pub fn push_empty(&mut self, header: Header) -> Result<(), CpioError> {
        let header = C::from_header(header, 0);
        let object = Object::new(header, Data::Empty);
        self.objects.inner.push(object);

        Ok(())
    }

    /// Finalize and image and write to writer, adding a trailing `TRAILER!!!` entry.
    pub fn write(&mut self) -> Result<(), CpioError> {
        let header = Header { nlink: 1, name: "TRAILER!!!".to_string(), ..Default::default() };

        // empty data
        let data = Cursor::new(vec![]);
        self.push_file(data, header)?;

        let mut writer = Writer::new(&mut self.writer);
        self.objects.to_mutwriter(&mut writer, ()).unwrap();

        // pad bytes if required
        let bytes_used = (writer.bits_written / 8) as u64;
        if let Some(blocks_used) = u32::try_from(bytes_used).unwrap().checked_div(self.pad_len) {
            let total_pad_len = (blocks_used + 1) * self.pad_len;
            let pad_len = total_pad_len - u32::try_from(bytes_used).unwrap();

            // Write 1K at a time
            let mut total_written = 0;
            while ((writer.bits_written / 8) as u64) < (bytes_used + u64::from(pad_len)) {
                let arr = &[0x00; 1024];

                // check if last block to write
                let len = if (pad_len - total_written) < 1024 {
                    (pad_len - total_written) % 1024
                } else {
                    // else, full 1K
                    1024
                };

                writer.write_bytes(&arr[..len.try_into().unwrap()])?;
                total_written += len;
            }
        }

        Ok(())
    }
}

/// Format-independent representation of a cpio header.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Header {
    /// Inode number.
    pub ino: u32,
    /// File mode and permissions.
    pub mode: u32,
    /// Owner user ID.
    pub uid: u32,
    /// Owner group ID.
    pub gid: u32,
    /// Number of hard links.
    pub nlink: u32,
    /// Last modification time (seconds since epoch).
    pub mtime: u32,
    /// Combined device number (odc only).
    pub dev: Option<u32>,
    /// Major device number of the device creating the file.
    pub devmajor: Option<u32>,
    /// Minor device number of the device creating the file.
    pub devminor: Option<u32>,
    /// Combined special-file device number (odc only).
    pub rdev: Option<u32>,
    /// Major special-file device number.
    pub rdevmajor: Option<u32>,
    /// Minor special-file device number.
    pub rdevminor: Option<u32>,
    /// Filename of the entry.
    pub name: String,
}

/// Extract the major device number from a raw `dev_t` value.
///
/// Uses the Linux kernel's device number encoding scheme.
#[cfg(all(feature = "std", unix))]
const fn linux_major(dev: u64) -> u32 {
    (((dev >> 32) & 0xffff_f000) | ((dev >> 8) & 0x0000_0fff)) as u32
}

/// Extract the minor device number from a raw `dev_t` value.
///
/// Uses the Linux kernel's device number encoding scheme.
#[cfg(all(feature = "std", unix))]
const fn linux_minor(dev: u64) -> u32 {
    (((dev >> 12) & 0xffff_ff00) | (dev & 0x0000_00ff)) as u32
}

/// Convert filesystem metadata into a cpio [`Header`].
///
/// The `name` field defaults to an empty string because [`std::fs::Metadata`]
/// does not carry the filename. Callers should set the name after conversion.
///
/// # Example
/// ```rust, no_run
/// use std::fs;
/// use librarium::Header;
///
/// let metadata = fs::metadata("/tmp/some_file").unwrap();
/// let mut header = Header::try_from(&metadata).unwrap();
/// header.name = "some_file".to_string();
/// ```
#[cfg(all(feature = "std", unix))]
#[allow(clippy::unnecessary_cast)]
impl TryFrom<&std::fs::Metadata> for Header {
    type Error = CpioError;

    fn try_from(metadata: &std::fs::Metadata) -> Result<Self, Self::Error> {
        use std::os::unix::fs::MetadataExt;

        let dev = metadata.dev();
        let rdev = metadata.rdev();

        Ok(Self {
            ino: metadata.ino() as u32,
            mode: metadata.mode() as u32,
            uid: metadata.uid() as u32,
            gid: metadata.gid() as u32,
            nlink: metadata.nlink() as u32,
            mtime: metadata.mtime() as u32,
            dev: Some(dev as u32),
            devmajor: Some(linux_major(dev)),
            devminor: Some(linux_minor(dev)),
            rdev: Some(rdev as u32),
            rdevmajor: Some(linux_major(rdev)),
            rdevminor: Some(linux_minor(rdev)),
            name: String::new(),
        })
    }
}

/// Single entry (header + data) in a cpio archive.
#[derive(DekuRead)]
pub struct Object<C: CpioHeader> {
    /// Parsed header for this entry.
    pub header: C,
    #[deku(ctx = "header.filesize()")]
    data: Data,
    #[deku(count = "header.data_pad()")]
    #[allow(dead_code)]
    data_pad: Vec<u8>,
}

impl<C: CpioHeader> Object<C> {
    /// Create a new `Object` with the given header and data.
    pub fn new(header: C, data: Data) -> Self {
        let data_pad = vec![0; header.data_pad()];
        Self { header, data, data_pad }
    }
}

impl<C: CpioHeader> MutWriter for Object<C> {
    fn to_mutwriter<W: Write + Seek>(
        &mut self,
        deku_writer: &mut Writer<W>,
        _: (),
    ) -> core::result::Result<(), DekuError> {
        log::trace!("writing header");
        DekuWriter::to_writer(&self.header, deku_writer, ())?;
        log::trace!("writing data, {}", self.header.filesize());
        self.data.to_mutwriter(deku_writer, self.header.filesize())?;
        // add padding
        log::trace!("adding padding");
        for _ in 0..self.header.data_pad() {
            0_u8.to_writer(deku_writer, ())?;
        }
        Ok(())
    }
}

trait OctalConversion {
    fn to_octal_bytes(&self, n: usize) -> Vec<u8>;
    fn from_octal_string(s: &str) -> Self;
}

impl<T> OctalConversion for T
where
    T: num_traits::PrimInt + num_traits::Zero + Debug,
{
    // Convert any integer type into an octal string
    fn to_octal_bytes(&self, n: usize) -> Vec<u8> {
        let mut num = *self;
        let mut result = Vec::new();
        let mut added = 0;

        if num == T::zero() {
            result.extend(vec![b'0'; n]);
            return result;
        }

        while num > T::zero() {
            let remainder = (num % T::from(8).unwrap()).to_u8().unwrap();
            result.push(b'0' + remainder);
            num = num / T::from(8).unwrap();
            added += 1;
        }

        result.extend(vec![b'0'; n - added]);

        result.reverse();
        result
    }

    // Convert an octal string back to the integer type
    fn from_octal_string(s: &str) -> Self {
        match T::from_str_radix(s, 8) {
            Ok(value) => value,
            Err(_) => T::zero(), // Or handle the error appropriately
        }
    }
}
