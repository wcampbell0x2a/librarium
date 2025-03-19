use crate::{CpioHeader, Header, OctalConversion};
use deku::prelude::*;
use std::ffi::{CStr, CString};
use std::fmt;
use std::io::{Read, Seek, Write};

const ODC_MAGIC: &[u8] = b"070707";

/// Legacy ASCII-based format
#[derive(DekuWrite, DekuRead, Debug)]
pub struct OdcHeader {
    #[deku(assert_eq = "ODC_MAGIC")]
    magic: [u8; 6],
    dev: Octal<u32, 6>,
    ino: Octal<u32, 6>,
    mode: Octal<u32, 6>,
    uid: Octal<u32, 6>,
    gid: Octal<u32, 6>,
    nlink: Octal<u32, 6>,
    rdev: Octal<u32, 6>,

    mtime: Octal<u64, 11>,
    namesize: Octal<u32, 6>,
    filesize: Octal<u64, 11>,
    #[deku(count = "namesize.value")]
    name: Vec<u8>,
}

impl CpioHeader for OdcHeader {
    fn from_header(header: Header, filesize: u64) -> Self {
        let name_bytes = header.name.into_bytes();
        let name_len = name_bytes.len() + 1;

        Self {
            magic: ODC_MAGIC.try_into().unwrap(),
            dev: Octal::new(header.dev.unwrap_or(0)),
            ino: Octal::new(header.ino),
            mode: Octal::new(header.mode),
            uid: Octal::new(header.uid),
            gid: Octal::new(header.gid),
            nlink: Octal::new(header.nlink),
            rdev: Octal::new(header.rdev.unwrap_or(0)),
            mtime: Octal::new(header.mtime.into()),
            namesize: Octal::new(name_len as u32),
            filesize: Octal::new(filesize),
            name: CString::new(name_bytes).unwrap().into_bytes_with_nul(),
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
        self.mtime.value as u32
    }

    fn filesize(&self) -> u32 {
        self.filesize.value as u32
    }

    fn dev(&self) -> Option<u32> {
        Some(self.dev.value)
    }

    fn devmajor(&self) -> Option<u32> {
        None
    }

    fn devminor(&self) -> Option<u32> {
        None
    }

    fn rdev(&self) -> Option<u32> {
        Some(self.rdev.value)
    }

    fn rdevmajor(&self) -> Option<u32> {
        None
    }

    fn rdevminor(&self) -> Option<u32> {
        None
    }

    fn namesize(&self) -> u32 {
        self.namesize.value
    }

    fn check(&self) -> Option<u32> {
        None
    }

    fn name(&self) -> &str {
        CStr::from_bytes_with_nul(&self.name).unwrap().to_str().unwrap()
    }

    fn data_pad(&self) -> usize {
        0
    }
}

#[derive(DekuWrite, DekuRead, Debug, Copy, Clone, Default)]
struct Octal<T: OctalConversion + fmt::Debug, const N: usize> {
    #[deku(reader = "Self::read(deku::reader)", writer = "self.write(deku::writer)")]
    pub value: T,
}

impl<T: OctalConversion + fmt::Debug, const N: usize> Octal<T, N> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    fn read<R: Read + Seek>(reader: &mut Reader<R>) -> Result<T, DekuError> {
        let value = <[u8; N]>::from_reader_with_ctx(reader, ())?;
        let s = std::str::from_utf8(&value).unwrap();
        let value = T::from_octal_string(s);
        Ok(value)
    }

    fn write<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), DekuError> {
        let bytes = self.value.to_octal_bytes(N);
        writer.write_bytes(&bytes)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_octal() {
        let var_name = &[0x30, 0x30, 0x30, 0x30, 0x31, 0x32];
        let dev = Octal::<u32, 6>::from_bytes((var_name, 0)).unwrap().1;
        let bytes = dev.to_bytes().unwrap();
        assert_eq!(var_name, &*bytes);
    }
}
