use deku::prelude::*;

use crate::Header;

/// How a header format checks the file data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Checksum {
    /// The format has no checksum field.
    None,
    /// The sum of all data bytes, wrapped to 32 bits (newc-crc, `070702`).
    ByteSum,
}

/// Common interface for all cpio header formats.
pub trait CpioHeader: for<'a> DekuReader<'a> + DekuWriter {
    /// The checksum this format stores. A writer reads the file data an extra time only
    /// when the format stores one.
    const CHECKSUM: Checksum = Checksum::None;

    /// Convert to the format-independent [`Header`].
    fn as_header(&self) -> Header;
    /// Construct from a format-independent [`Header`] and file size.
    fn from_header(header: Header, filesize: u64) -> Self;
    /// Inode number.
    fn ino(&self) -> u32;
    /// File mode and permissions.
    fn mode(&self) -> u32;
    /// Owner user ID.
    fn uid(&self) -> u32;
    /// Owner group ID.
    fn gid(&self) -> u32;
    /// Number of hard links.
    fn nlink(&self) -> u32;
    /// Last modification time (seconds since epoch).
    fn mtime(&self) -> u32;
    /// Size of the file data in bytes.
    fn filesize(&self) -> u32;
    /// Device number of device creating file.
    fn dev(&self) -> Option<u32>;
    /// Device major number of device creating file.
    fn devmajor(&self) -> Option<u32>;
    /// Device minor number of device creating file.
    fn devminor(&self) -> Option<u32>;
    /// Special-file device number.
    fn rdev(&self) -> Option<u32>;
    /// Special-file major device number.
    fn rdevmajor(&self) -> Option<u32>;
    /// Special-file minor device number.
    fn rdevminor(&self) -> Option<u32>;
    /// Length of the filename (including the null terminator).
    fn namesize(&self) -> u32;
    /// Checksum of the file data, if the format supports it.
    fn check(&self) -> Option<u32>;
    /// Filename of the entry.
    fn name(&self) -> &str;
    /// Number of padding bytes after the file data.
    fn data_pad(&self) -> usize;

    /// Set the checksum field. Only meaningful for CRC variants (070702).
    fn set_check(&mut self, _check: u32) {}
}
