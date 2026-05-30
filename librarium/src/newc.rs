use crate::{CpioHeader, Header};
use core::ffi::CStr;
use deku::prelude::*;
use no_std_io2::io::{Read, Seek, Write};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{ffi::CString, string::ToString, vec, vec::Vec};

const NEWC_MAGIC: [u8; 6] = *b"070701";
const NEWC_CRC_MAGIC: [u8; 6] = *b"070702";
// Size of magic field in bytes, derived from DekuSize
const MAGIC_SIZE_BYTES: usize = <[u8; 6]>::SIZE_BYTES.unwrap();

/// Shared field accessors for newc-style headers (070701 and 070702).
trait NewcFields {
    fn ino_field(&self) -> &Ascii;
    fn mode_field(&self) -> &Ascii;
    fn uid_field(&self) -> &Ascii;
    fn gid_field(&self) -> &Ascii;
    fn nlink_field(&self) -> &Ascii;
    fn mtime_field(&self) -> &Ascii;
    fn filesize_field(&self) -> &Ascii;
    fn devmajor_field(&self) -> &Ascii;
    fn devminor_field(&self) -> &Ascii;
    fn rdevmajor_field(&self) -> &Ascii;
    fn rdevminor_field(&self) -> &Ascii;
    fn namesize_field(&self) -> &Ascii;
    fn check_field(&self) -> &Ascii;
    fn name_bytes(&self) -> &[u8];
}

fn newc_as_header(h: &impl NewcFields) -> Header {
    Header {
        ino: h.ino_field().value,
        mode: h.mode_field().value,
        uid: h.uid_field().value,
        gid: h.gid_field().value,
        nlink: h.nlink_field().value,
        mtime: h.mtime_field().value,
        dev: None,
        devmajor: Some(h.devmajor_field().value),
        devminor: Some(h.devminor_field().value),
        rdev: None,
        rdevmajor: Some(h.rdevmajor_field().value),
        rdevminor: Some(h.rdevminor_field().value),
        name: CStr::from_bytes_with_nul(h.name_bytes()).unwrap().to_str().unwrap().to_string(),
    }
}

fn newc_from_header(header: Header, filesize: u64, magic: [u8; 6]) -> NewcRawFields {
    let name_bytes = header.name.into_bytes();
    let name_len = name_bytes.len() + 1; // +1 for null terminator
    NewcRawFields {
        magic,
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
        namesize: Ascii::new(name_len as u32),
        check: Ascii::new(0),
        name: CString::new(name_bytes).unwrap().into_bytes_with_nul(),
        name_pad: vec![0; pad_to_4(MAGIC_SIZE_BYTES + name_len)],
    }
}

/// Intermediate struct returned by `newc_from_header` to initialize either variant.
struct NewcRawFields {
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
    name: Vec<u8>,
    name_pad: Vec<u8>,
}

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

/// CRC variant of the newc cpio Header (magic `070702`)
#[derive(DekuWrite, DekuRead, Debug)]
pub struct NewcCrcHeader {
    #[deku(assert_eq = "NEWC_CRC_MAGIC")]
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

impl NewcFields for NewcHeader {
    fn ino_field(&self) -> &Ascii {
        &self.ino
    }
    fn mode_field(&self) -> &Ascii {
        &self.mode
    }
    fn uid_field(&self) -> &Ascii {
        &self.uid
    }
    fn gid_field(&self) -> &Ascii {
        &self.gid
    }
    fn nlink_field(&self) -> &Ascii {
        &self.nlink
    }
    fn mtime_field(&self) -> &Ascii {
        &self.mtime
    }
    fn filesize_field(&self) -> &Ascii {
        &self.filesize
    }
    fn devmajor_field(&self) -> &Ascii {
        &self.devmajor
    }
    fn devminor_field(&self) -> &Ascii {
        &self.devminor
    }
    fn rdevmajor_field(&self) -> &Ascii {
        &self.rdevmajor
    }
    fn rdevminor_field(&self) -> &Ascii {
        &self.rdevminor
    }
    fn namesize_field(&self) -> &Ascii {
        &self.namesize
    }
    fn check_field(&self) -> &Ascii {
        &self.check
    }
    fn name_bytes(&self) -> &[u8] {
        &self.name
    }
}

impl NewcFields for NewcCrcHeader {
    fn ino_field(&self) -> &Ascii {
        &self.ino
    }
    fn mode_field(&self) -> &Ascii {
        &self.mode
    }
    fn uid_field(&self) -> &Ascii {
        &self.uid
    }
    fn gid_field(&self) -> &Ascii {
        &self.gid
    }
    fn nlink_field(&self) -> &Ascii {
        &self.nlink
    }
    fn mtime_field(&self) -> &Ascii {
        &self.mtime
    }
    fn filesize_field(&self) -> &Ascii {
        &self.filesize
    }
    fn devmajor_field(&self) -> &Ascii {
        &self.devmajor
    }
    fn devminor_field(&self) -> &Ascii {
        &self.devminor
    }
    fn rdevmajor_field(&self) -> &Ascii {
        &self.rdevmajor
    }
    fn rdevminor_field(&self) -> &Ascii {
        &self.rdevminor
    }
    fn namesize_field(&self) -> &Ascii {
        &self.namesize
    }
    fn check_field(&self) -> &Ascii {
        &self.check
    }
    fn name_bytes(&self) -> &[u8] {
        &self.name
    }
}

impl CpioHeader for NewcHeader {
    fn from_header(header: Header, filesize: u64) -> Self {
        let f = newc_from_header(header, filesize, NEWC_MAGIC);
        Self {
            magic: f.magic,
            ino: f.ino,
            mode: f.mode,
            uid: f.uid,
            gid: f.gid,
            nlink: f.nlink,
            mtime: f.mtime,
            filesize: f.filesize,
            devmajor: f.devmajor,
            devminor: f.devminor,
            rdevmajor: f.rdevmajor,
            rdevminor: f.rdevminor,
            namesize: f.namesize,
            check: f.check,
            name: f.name,
            name_pad: f.name_pad,
        }
    }
    fn as_header(&self) -> Header {
        newc_as_header(self)
    }
    fn ino(&self) -> u32 {
        self.ino_field().value
    }
    fn mode(&self) -> u32 {
        self.mode_field().value
    }
    fn uid(&self) -> u32 {
        self.uid_field().value
    }
    fn gid(&self) -> u32 {
        self.gid_field().value
    }
    fn nlink(&self) -> u32 {
        self.nlink_field().value
    }
    fn mtime(&self) -> u32 {
        self.mtime_field().value
    }
    fn filesize(&self) -> u32 {
        self.filesize_field().value
    }
    fn dev(&self) -> Option<u32> {
        None
    }
    fn devmajor(&self) -> Option<u32> {
        Some(self.devmajor_field().value)
    }
    fn devminor(&self) -> Option<u32> {
        Some(self.devminor_field().value)
    }
    fn rdev(&self) -> Option<u32> {
        None
    }
    fn rdevmajor(&self) -> Option<u32> {
        Some(self.rdevmajor_field().value)
    }
    fn rdevminor(&self) -> Option<u32> {
        Some(self.rdevminor_field().value)
    }
    fn namesize(&self) -> u32 {
        self.namesize_field().value
    }
    fn check(&self) -> Option<u32> {
        Some(self.check_field().value)
    }
    fn name(&self) -> &str {
        CStr::from_bytes_with_nul(self.name_bytes()).unwrap().to_str().unwrap()
    }
    fn data_pad(&self) -> usize {
        pad_to_4(self.filesize() as usize)
    }
}

impl CpioHeader for NewcCrcHeader {
    fn from_header(header: Header, filesize: u64) -> Self {
        let f = newc_from_header(header, filesize, NEWC_CRC_MAGIC);
        Self {
            magic: f.magic,
            ino: f.ino,
            mode: f.mode,
            uid: f.uid,
            gid: f.gid,
            nlink: f.nlink,
            mtime: f.mtime,
            filesize: f.filesize,
            devmajor: f.devmajor,
            devminor: f.devminor,
            rdevmajor: f.rdevmajor,
            rdevminor: f.rdevminor,
            namesize: f.namesize,
            check: f.check,
            name: f.name,
            name_pad: f.name_pad,
        }
    }
    fn as_header(&self) -> Header {
        newc_as_header(self)
    }
    fn ino(&self) -> u32 {
        self.ino_field().value
    }
    fn mode(&self) -> u32 {
        self.mode_field().value
    }
    fn uid(&self) -> u32 {
        self.uid_field().value
    }
    fn gid(&self) -> u32 {
        self.gid_field().value
    }
    fn nlink(&self) -> u32 {
        self.nlink_field().value
    }
    fn mtime(&self) -> u32 {
        self.mtime_field().value
    }
    fn filesize(&self) -> u32 {
        self.filesize_field().value
    }
    fn dev(&self) -> Option<u32> {
        None
    }
    fn devmajor(&self) -> Option<u32> {
        Some(self.devmajor_field().value)
    }
    fn devminor(&self) -> Option<u32> {
        Some(self.devminor_field().value)
    }
    fn rdev(&self) -> Option<u32> {
        None
    }
    fn rdevmajor(&self) -> Option<u32> {
        Some(self.rdevmajor_field().value)
    }
    fn rdevminor(&self) -> Option<u32> {
        Some(self.rdevminor_field().value)
    }
    fn namesize(&self) -> u32 {
        self.namesize_field().value
    }
    fn check(&self) -> Option<u32> {
        Some(self.check_field().value)
    }
    fn name(&self) -> &str {
        CStr::from_bytes_with_nul(self.name_bytes()).unwrap().to_str().unwrap()
    }
    fn data_pad(&self) -> usize {
        pad_to_4(self.filesize() as usize)
    }
    fn set_check(&mut self, check: u32) {
        self.check = Ascii::new(check);
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
