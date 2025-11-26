use crate::{CpioHeader, Header};
use deku::prelude::*;
use std::ffi::CStr;
use std::io::{Read, Seek, Write};

const NEWC_MAGIC: [u8; 6] = [b'0', b'7', b'0', b'7', b'0', b'1'];
// Size of magic field in bytes, derived from DekuSize
const MAGIC_SIZE_BYTES: usize = <[u8; 6]>::SIZE_BYTES.unwrap();

/// Improved cpio Header, also known as "SVR4" or "New ASCII"
#[derive(DekuWrite, DekuRead, Debug)]
pub struct NewcHeader {
    #[deku(assert_eq = "NEWC_MAGIC")]
    magic: [u8; 6],
    ino: Ascii,
    mode: Ascii,
    uid: Ascii,
    gid: Ascii,
    nlink: Ascii,
    mtime: Ascii,
    filesize: Ascii,
    devmajor: Ascii,
    devminor: Ascii,
    rdevmajor: Ascii,
    rdevminor: Ascii,
    namesize: Ascii,
    check: Ascii,
    #[deku(count = "namesize.value")]
    name: Vec<u8>,
    #[deku(count = "pad_to_4(MAGIC_SIZE_BYTES + namesize.value as usize)")]
    name_pad: Vec<u8>,
}

impl CpioHeader for NewcHeader {
    fn from_header(header: Header, filesize: u64) -> Self {
        let name_bytes = header.name.into_bytes();
        let name_len = name_bytes.len();
        NewcHeader {
            magic: NEWC_MAGIC,
            ino: Ascii::new(header.ino),
            mode: Ascii::new(header.mode),
            uid: Ascii::new(header.uid),
            gid: Ascii::new(header.gid),
            nlink: Ascii::new(header.nlink),
            mtime: Ascii::new(header.mtime),
            filesize: Ascii::new(u32::try_from(filesize).unwrap()),
            devmajor: Ascii::new(header.devmajor.unwrap_or(0)),
            devminor: Ascii::new(header.devminor.unwrap_or(0)),
            rdevmajor: Ascii::new(header.rdevmajor.unwrap_or(0)),
            rdevminor: Ascii::new(header.rdevminor.unwrap_or(0)),
            namesize: Ascii::new(name_len as u32 + 1),
            check: Ascii::new(0),
            name: name_bytes.to_vec(),
            name_pad: vec![0; pad_to_4(MAGIC_SIZE_BYTES + name_len)],
        }
    }

    fn as_header(&self) -> Header {
        Header {
            ino: self.ino(),
            mode: self.mode(),
            uid: self.uid(),
            gid: self.gid(),
            nlink: self.nlink(),
            mtime: self.mtime(),
            dev: self.dev(),
            devmajor: self.devmajor(),
            devminor: self.devminor(),
            rdev: self.rdev(),
            rdevmajor: self.rdevmajor(),
            rdevminor: self.rdevminor(),
            name: self.name().to_string(),
        }
    }

    fn ino(&self) -> u32 {
        self.ino.value
    }

    fn mode(&self) -> u32 {
        self.mode.value
    }

    fn uid(&self) -> u32 {
        self.uid.value
    }

    fn gid(&self) -> u32 {
        self.gid.value
    }

    fn nlink(&self) -> u32 {
        self.nlink.value
    }

    fn mtime(&self) -> u32 {
        self.mtime.value
    }

    fn filesize(&self) -> u32 {
        self.filesize.value
    }

    fn dev(&self) -> Option<u32> {
        None
    }

    fn devmajor(&self) -> Option<u32> {
        Some(self.devmajor.value)
    }

    fn devminor(&self) -> Option<u32> {
        Some(self.devminor.value)
    }

    fn rdev(&self) -> Option<u32> {
        None
    }

    fn rdevmajor(&self) -> Option<u32> {
        Some(self.rdevmajor.value)
    }

    fn rdevminor(&self) -> Option<u32> {
        Some(self.rdevminor.value)
    }

    fn namesize(&self) -> u32 {
        self.namesize.value + 1
    }

    fn check(&self) -> Option<u32> {
        Some(self.check.value)
    }

    fn name(&self) -> &str {
        CStr::from_bytes_with_nul(&self.name).unwrap().to_str().unwrap()
    }

    fn data_pad(&self) -> usize {
        pad_to_4(self.filesize() as usize)
    }
}

/// pad out to a multiple of 4 bytes
fn pad_to_4(len: usize) -> usize {
    match len % 4 {
        0 => 0,
        x => 4 - x,
    }
}

#[derive(DekuWrite, DekuRead, DekuSize, Debug, Copy, Clone, Default)]
struct Ascii {
    #[deku(reader = "Self::read(deku::reader)", writer = "self.write(deku::writer)")]
    pub value: u32,
}

impl Ascii {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    // [2024-10-29T15:41:58Z DEBUG librarium] [30, 30, 38, 42, 32, 38, 37, 34]
    // [2024-10-29T15:41:58Z DEBUG librarium] 008B2874
    // [2024-10-29T15:41:58Z DEBUG librarium] 8b2874
    fn read<R: Read + Seek>(reader: &mut Reader<R>) -> Result<u32, DekuError> {
        let value = <[u8; 8]>::from_reader_with_ctx(reader, ())?;
        log::debug!("{:02x?}", value);
        let s = core::str::from_utf8(&value).unwrap();
        log::debug!("{}", s);
        let value = u32::from_str_radix(s, 16).unwrap();
        log::debug!("{:02x?}", value);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acsii() {
        let bytes = &[0x30, 0x30, 0x38, 0x42, 0x32, 0x38, 0x37, 0x34];
        let (_, a) = Ascii::from_bytes((bytes, 0)).unwrap();
        let written = a.to_bytes().unwrap();
        assert_eq!(*bytes, *written);
    }
}
