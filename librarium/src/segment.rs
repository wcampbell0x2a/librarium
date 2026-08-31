//! Navigation of concatenated cpio archives.
//!
//! A file can hold more than one cpio archive, one after the other. The Linux
//! kernel initramfs is the usual example: the boot loader joins an uncompressed
//! archive that holds the CPU microcode with the main archive, and the kernel
//! unpacks each archive in turn into the same root file system.
//!
//! Each archive ends with a `TRAILER!!!` entry, and zero bytes usually pad the
//! archive to an alignment boundary. Use [`ArchiveReader::end_offset`] to find
//! the end of an archive, [`next_segment_offset`] to step over the padding, and
//! [`segment_format`] to learn if the next segment is a cpio archive or
//! compressed data.
//!
//! [`ArchiveReader::end_offset`]: crate::ArchiveReader::end_offset

use no_std_io2::io::{ErrorKind, SeekFrom};

use crate::MAGIC_SIZE_BYTES;
use crate::error::CpioError;
use crate::newc::{NEWC_CRC_MAGIC, NEWC_MAGIC};
use crate::odc::ODC_MAGIC;
use crate::read_seek::ReadSeek;

/// Number of bytes read at a time when the padding between archives is skipped.
const PAD_SCAN_CHUNK: usize = 512;

/// cpio header format found at the start of a segment.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SegmentFormat {
    /// Old ASCII format, magic `070707`.
    Odc,
    /// New ASCII (SVR4) format, magic `070701`.
    Newc,
    /// New ASCII CRC format, magic `070702`.
    NewcCrc,
}

impl SegmentFormat {
    /// Identify the format from the first bytes of a segment.
    ///
    /// Returns `None` if the bytes are not a cpio magic. Compressed segments
    /// give `None`, because the caller must decompress them first.
    #[must_use]
    pub fn from_magic(magic: &[u8]) -> Option<Self> {
        match magic {
            _ if magic == ODC_MAGIC => Some(Self::Odc),
            _ if magic == NEWC_MAGIC => Some(Self::Newc),
            _ if magic == NEWC_CRC_MAGIC => Some(Self::NewcCrc),
            _ => None,
        }
    }
}

/// Find the start of the next segment after `from`.
///
/// Archives are padded with zero bytes, so this returns the offset of the first
/// byte that is not zero at or after `from`. Returns `None` if only zero bytes
/// remain, which means `from` is at the end of the last archive.
///
/// The returned offset is not necessarily a cpio archive. Give it to
/// [`segment_format`] to find out.
pub fn next_segment_offset<R: ReadSeek>(
    reader: &mut R,
    from: u64,
) -> Result<Option<u64>, CpioError> {
    reader.seek(SeekFrom::Start(from))?;

    let mut position = from;
    let mut buf = [0u8; PAD_SCAN_CHUNK];
    loop {
        let read = match reader.read(&mut buf) {
            Ok(0) => return Ok(None),
            Ok(read) => read,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        };

        if let Some(index) = buf[..read].iter().position(|&byte| byte != 0) {
            return Ok(Some(position + index as u64));
        }
        position += read as u64;
    }
}

/// Read the magic at `offset` and identify the cpio format.
///
/// Returns `None` if the bytes are not a cpio magic, or if fewer than six bytes
/// remain. A segment that holds compressed data gives `None`.
pub fn segment_format<R: ReadSeek>(
    reader: &mut R,
    offset: u64,
) -> Result<Option<SegmentFormat>, CpioError> {
    reader.seek(SeekFrom::Start(offset))?;

    let mut magic = [0u8; MAGIC_SIZE_BYTES];
    let mut filled = 0;
    while filled < MAGIC_SIZE_BYTES {
        match reader.read(&mut magic[filled..]) {
            Ok(0) => return Ok(None),
            Ok(read) => filled += read,
            Err(e) if e.kind() == ErrorKind::Interrupted => (),
            Err(e) => return Err(e.into()),
        }
    }

    Ok(SegmentFormat::from_magic(&magic))
}
