/*!
Reader and Writer library for cpio archives

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
*/

#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
type _ReadmeTest = ();

use std::fmt::Debug;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use deku::DekuError;
use deku::prelude::*;
use deku::writer::Writer;
use thiserror::Error;

const TRAILER: &str = "TRAILER!!!";

pub mod newc;
pub use newc::NewcHeader;
pub mod odc;
pub use odc::OdcHeader;

/// Common information between types of cpio headers
pub trait CpioHeader: for<'a> DekuReader<'a> + DekuWriter {
    fn as_header(&self) -> Header;
    fn from_header(header: Header, filesize: u64) -> Self;
    fn ino(&self) -> u32;
    fn mode(&self) -> u32;
    fn uid(&self) -> u32;
    fn gid(&self) -> u32;
    fn nlink(&self) -> u32;
    fn mtime(&self) -> u32;
    fn filesize(&self) -> u32;
    /// Device number of device creating file
    fn dev(&self) -> Option<u32>;
    /// Device major number of device creating file
    fn devmajor(&self) -> Option<u32>;
    /// Device minor number of device creating file
    fn devminor(&self) -> Option<u32>;

    fn rdev(&self) -> Option<u32>;
    fn rdevmajor(&self) -> Option<u32>;
    fn rdevminor(&self) -> Option<u32>;

    fn namesize(&self) -> u32;
    fn check(&self) -> Option<u32>;
    fn name(&self) -> &str;
    fn data_pad(&self) -> usize;
}

/// DekuWriter, but can write to self
trait MutWriter<Ctx = ()> {
    fn to_mutwriter<W: Write + Seek>(
        &mut self,
        deku_writer: &mut Writer<W>,
        ctx: Ctx,
    ) -> core::result::Result<(), DekuError>;
}

/// Errors generated from library
#[derive(Error, Debug)]
pub enum CpioError {
    #[error("std io error: {0}")]
    StdIo(#[from] io::Error),

    #[error("deku error: {0:?}")]
    Deku(#[from] deku::DekuError),
}

/// `Read` + `Seek`
pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

/// Private struct containing logic to read the data section from the archive
#[derive(Debug)]
pub(crate) struct ReaderWithOffset<R: ReadSeek> {
    io: R,
    /// Offset from start of file to data
    offset: u64,
}

impl<R: ReadSeek> ReaderWithOffset<R> {
    pub fn new(mut io: R, offset: u64) -> std::io::Result<Self> {
        io.seek(SeekFrom::Start(offset))?;
        Ok(Self { io, offset })
    }
}

impl<R: ReadSeek> Read for ReaderWithOffset<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.io.read(buf)
    }
}

impl<R: ReadSeek> Seek for ReaderWithOffset<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let seek = match pos {
            SeekFrom::Start(start) => SeekFrom::Start(self.offset + start),
            seek => seek,
        };
        self.io.seek(seek).map(|x| x - self.offset)
    }
}

impl<T: ReadSeek> CpioReader for T {}
/// Extract data from cpio Archive
pub trait CpioReader: ReadSeek {
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
        let current_pos = reader.stream_position().unwrap();

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
    pub reader: Box<dyn ReadSeek + 'b>,
    pub objects: Objects<C>,
}

impl<'b, C: CpioHeader> ArchiveReader<'b, C> {
    pub fn from_reader(reader: impl ReadSeek + 'b) -> Result<Self, CpioError> {
        Self::from_reader_with_offset(reader, 0)
    }

    pub fn from_reader_with_offset(
        reader: impl ReadSeek + 'b,
        offset: u64,
    ) -> Result<Self, CpioError> {
        let mut reader: Box<dyn ReadSeek> = if offset == 0 {
            Box::new(reader)
        } else {
            let reader = ReaderWithOffset::new(reader, offset)?;
            Box::new(reader)
        };
        let (_, objects) = Objects::from_reader((&mut reader, 0))?;
        Ok(Self { reader, objects })
    }

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
pub trait WriteSeek: std::io::Write + Seek {}
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

        let header = C::from_header(header, filesize);
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
        if self.pad_len != 0 {
            // Pad out block_size to 4K
            let blocks_used: u32 = u32::try_from(bytes_used).unwrap() / self.pad_len;
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

/// Common representation of cpio Header
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Header {
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u32,
    pub mtime: u32,
    pub dev: Option<u32>,
    pub devmajor: Option<u32>,
    pub devminor: Option<u32>,
    pub rdev: Option<u32>,
    pub rdevmajor: Option<u32>,
    pub rdevminor: Option<u32>,
    pub name: String,
}

/// Object in cpio archive
#[derive(DekuRead)]
pub struct Object<C: CpioHeader> {
    pub header: C,
    #[deku(ctx = "header.filesize()")]
    data: Data,
    #[deku(count = "header.data_pad()")]
    #[allow(dead_code)]
    data_pad: Vec<u8>,
}

impl<C: CpioHeader> Object<C> {
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
