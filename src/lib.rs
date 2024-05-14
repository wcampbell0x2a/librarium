//! Copy in/out file archives

use std::ffi::CString;
use std::io::{self, Cursor};
use std::io::{Read, Seek, SeekFrom, Write};

use deku::prelude::*;
use thiserror::Error;

const MAGIC: [u8; 6] = [b'0', b'7', b'0', b'7', b'0', b'1'];
const TRAILER: &str = "TRAILER!!!";

/// Errors generated from library
#[derive(Error, Debug)]
pub enum CpioError {
    #[error("std io error: {0}")]
    StdIo(#[from] io::Error),

    #[error("deku error: {0:?}")]
    Deku(#[from] deku::DekuError),
}

pub trait ReadSeek: Read + Seek {}
// pub trait BufReadSeek: BufRead + Seek + Send {}
impl<T: Read + Seek> ReadSeek for T {}

/// Private struct containing logic to read the data section from the archive
#[derive(Debug)]
pub(crate) struct ReaderWithOffset<R: ReadSeek> {
    io: R,
    /// Offset from start of file to squashfs
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
pub trait CpioReader: ReadSeek {
    fn extract_data<W>(&mut self, object: &Object, writer: &mut W) -> Result<(), CpioError>
    where
        W: Write + Seek,
    {
        // found the file, seek forward
        if let Data::Offset(offset) = object.data {
            self.seek(SeekFrom::Start(offset)).unwrap();
            let mut buf = vec![0; object.header.filesize.value as usize];
            self.read_exact(&mut buf).unwrap();
            writer.write_all(&buf)?;
            Ok(())
        } else {
            panic!("no offset! TODO improve this");
        }
    }
}

pub enum Data {
    /// On read: Save current stream_position() as `Offset`, seek `header.filesize`
    /// This will be used to seek this this position if we want to extract *just* this file
    Offset(u64),
    /// On write: Write `Reader` to write buffer
    Reader(Box<dyn ReadSeek>),
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
        let position = filesize as i64 + pad_to_4(filesize as usize) as i64;
        let _ = reader.seek(SeekFrom::Current(position));

        Ok(Self::Offset(current_pos))
    }
}

impl Data {
    fn writer<W: Write + Seek>(&mut self, writer: &mut Writer<W>, _: u32) -> Result<(), DekuError> {
        if let Self::Reader(reader) = self {
            // read from reader
            let mut data = vec![];
            reader.read_to_end(&mut data).unwrap();

            // write to deku
            data.to_writer(writer, ())?;

            // add padding
            for _ in 0..pad_to_4(data.len()) {
                0_u8.to_writer(writer, ())?;
            }
        } else {
            panic!("ah");
        }

        Ok(())
    }
}

#[derive(DekuRead)]
pub struct Objects {
    #[deku(until = "Self::until")]
    pub inner: Vec<Object>,
}

impl Objects {
    fn writer<W: ::deku::no_std_io::Write + Seek>(
        &mut self,
        __deku_writer: &mut ::deku::writer::Writer<W>,
        _: (),
    ) -> core::result::Result<(), ::deku::DekuError> {
        for i in &mut self.inner {
            i.writer(__deku_writer, ())?;
        }
        Ok(())
    }
}

impl Objects {
    fn until(last_object: &Object) -> bool {
        last_object.name.to_str() == Ok(TRAILER)
    }
}

pub struct ArchiveReader<'b> {
    pub reader: Box<dyn ReadSeek + 'b>,
    pub objects: Objects,
}

impl<'b> ArchiveReader<'b> {
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
        name: CString,
        writer: &mut W,
    ) -> Result<Option<()>, CpioError>
    where
        W: Write + Seek,
    {
        for object in &self.objects.inner {
            if name == object.name {
                self.reader.extract_data(object, writer)?;
                return Ok(Some(()));
            }
        }

        Ok(None)
    }
}

pub trait WriteSeek: std::io::Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}

pub struct ArchiveWriter<'a> {
    pub writer: Box<dyn WriteSeek + 'a>,
    pub objects: Objects,
    pad_len: u32,
}

impl<'a> ArchiveWriter<'a> {
    pub fn new(writer: Box<dyn WriteSeek + 'a>) -> Self {
        Self { writer, objects: Objects { inner: vec![] }, pad_len: 0x400 }
    }

    pub fn push_file(
        &mut self,
        mut reader: impl ReadSeek + 'a + 'static,
        path: CString,
        header: Header,
    ) -> Result<(), CpioError> {
        // stream_len
        let filesize = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        let namesize = Ascii::new(u32::try_from(path.as_bytes().len() + 1).unwrap());
        let cpio_header = CpioNewcHeader {
            magic: MAGIC,
            ino: Ascii::new(header.ino),
            mode: Ascii::new(header.mode),
            uid: Ascii::new(header.uid),
            gid: Ascii::new(header.gid),
            nlink: Ascii::new(header.nlink),
            mtime: Ascii::new(header.mtime),
            filesize: Ascii::new(u32::try_from(filesize).unwrap()),
            devmajor: Ascii::new(header.devmajor),
            devminor: Ascii::new(header.devminor),
            rdevmajor: Ascii::new(header.rdevmajor),
            rdevminor: Ascii::new(header.rdevminor),
            namesize,
            check: Ascii::new(0),
        };

        let object = Object {
            header: cpio_header,
            name: CString::new(path.as_bytes()).unwrap(),
            name_pad: vec![0; pad_to_4(6 + namesize.value as usize)],
            data: Data::Reader(Box::new(reader)),
        };

        self.objects.inner.push(object);

        Ok(())
    }

    /// Before writing to Writer, a "TRAILER!!!" entry must be added
    pub fn write(&mut self) -> Result<(), CpioError> {
        let header = Header { nlink: 1, ..Default::default() };

        // empty data
        let data = Cursor::new(vec![]);
        let path = CString::new("TRAILER!!!").unwrap();
        self.push_file(data, path, header)?;

        let mut writer = Writer::new(&mut self.writer);
        self.objects.writer(&mut writer, ()).unwrap();

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

#[derive(Default)]
pub struct Header {
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u32,
    pub mtime: u32,
    pub devmajor: u32,
    pub devminor: u32,
    pub rdevmajor: u32,
    pub rdevminor: u32,
}

#[derive(DekuRead)]
pub struct Object {
    pub header: CpioNewcHeader,

    #[deku(assert = "name.as_bytes().len() == header.namesize.value as usize - 1")]
    pub name: CString,

    #[deku(count = "pad_to_4(6 + header.namesize.value as usize)")]
    name_pad: Vec<u8>,

    #[deku(ctx = "header.filesize.value")]
    data: Data,
}

impl Object {
    #[allow(unused_variables)]
    fn writer<W: ::deku::no_std_io::Write + Seek>(
        &mut self,
        __deku_writer: &mut ::deku::writer::Writer<W>,
        _: (),
    ) -> core::result::Result<(), ::deku::DekuError> {
        DekuWriter::to_writer(&self.header, __deku_writer, ())?;

        if self.name.as_bytes().len() != self.header.namesize.value as usize - 1 {
            panic!("add assert here");
        }
        DekuWriter::to_writer(&self.name, __deku_writer, ())?;
        DekuWriter::to_writer(&self.name_pad, __deku_writer, ())?;
        self.data.writer(__deku_writer, self.header.filesize.value)?;
        Ok(())
    }
}

/// The new (SVR4) portable format, which supports file systems having more than 65536 i-nodes. (4294967295 bytes)
#[derive(DekuWrite, DekuRead, Debug)]
pub struct CpioNewcHeader {
    #[deku(assert_eq = "MAGIC")]
    pub magic: [u8; 6],
    pub ino: Ascii,
    pub mode: Ascii,
    pub uid: Ascii,
    pub gid: Ascii,
    pub nlink: Ascii,
    pub mtime: Ascii,
    pub filesize: Ascii,
    pub devmajor: Ascii,
    pub devminor: Ascii,
    pub rdevmajor: Ascii,
    pub rdevminor: Ascii,
    pub namesize: Ascii,
    pub check: Ascii,
}

/// pad out to a multiple of 4 bytes
fn pad_to_4(len: usize) -> usize {
    match len % 4 {
        0 => 0,
        x => 4 - x,
    }
}

#[derive(DekuWrite, DekuRead, Debug, Copy, Clone, Default)]
pub struct Ascii {
    #[deku(reader = "Self::read(deku::reader)", writer = "self.write(deku::writer)")]
    pub value: u32,
}

impl Ascii {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    fn read<R: Read + Seek>(reader: &mut Reader<R>) -> Result<u32, DekuError> {
        let value = <[u8; 8]>::from_reader_with_ctx(reader, ())?;
        let s = core::str::from_utf8(&value).unwrap();
        let value = u32::from_str_radix(s, 16).unwrap();
        Ok(value)
    }

    // [30, 30, 38, 42, 32, 38, 37, 34]
    // "008B2874"
    fn write<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), DekuError> {
        let bytes = self.value.to_be_bytes();
        for b in bytes {
            let left = (b & 0xf0) >> 4;
            let right = b & 0x0f;

            let left = if left > 9 { left + 0x37 } else { left + 0x30 };
            let right = if right > 9 { right + 0x37 } else { right + 0x30 };

            writer.write_bytes(&[left])?;
            writer.write_bytes(&[right])?;
        }

        Ok(())
    }
}
