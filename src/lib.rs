use std::ffi::CString;

use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

const MAGIC: [u8; 6] = [0x30, 0x37, 0x30, 0x37, 0x30, 0x31];
const TRAILER: &str = "TRAILER!!!";

#[derive(DekuWrite, DekuRead, Debug, Clone)]
pub struct Archive {
    #[deku(until = "Self::until")]
    pub objects: Vec<Object>,
}

impl Archive {
    fn until(last_object: &Object) -> bool {
        last_object.name.to_str() == Ok(TRAILER)
    }
}

#[derive(DekuWrite, DekuRead, Debug, Clone)]
pub struct Object {
    pub header: CpioNewcHeader,
    #[deku(assert = "name.as_bytes().len() == header.namesize.inner as usize - 1")]
    pub name: CString,
    #[deku(count = "pad_to_4(6 + header.namesize.inner as usize)")]
    name_pad: Vec<u8>,
    #[deku(count = "header.filesize.inner")]
    pub file: Vec<u8>,
    #[deku(count = "pad_to_4(header.filesize.inner as usize)")]
    file_pad: Vec<u8>,
}

#[derive(DekuWrite, DekuRead, Debug, Clone)]
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

#[derive(DekuWrite, DekuRead, Debug, Clone, Default)]
pub struct Ascii {
    #[deku(
        reader = "Self::read(deku::rest)",
        writer = "Self::write(deku::output)"
    )]
    inner: u32,
}

impl Ascii {
    fn read(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, u32), DekuError> {
        let (rest, value) = <[u8; 8]>::read(rest, ())?;
        let s = core::str::from_utf8(&value).unwrap();
        let value = u32::from_str_radix(s, 16).unwrap();
        Ok((rest, value))
    }

    //TODO: impl write
    fn write(_output: &mut BitVec<u8, Msb0>) -> Result<(), DekuError> {
        Ok(())
    }
}
