use deku::prelude::*;

use crate::Header;

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
