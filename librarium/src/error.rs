use thiserror::Error;

/// Errors returned by this library.
#[derive(Error, Debug)]
pub enum CpioError {
    /// An I/O error occurred.
    #[error("std io error: {0}")]
    StdIo(#[from] no_std_io2::io::Error),

    /// The file is too large for the 32-bit size field of a cpio header.
    #[error("file of {0} bytes is larger than the cpio maximum of 4294967295 bytes")]
    FileTooLarge(u64),

    /// The entry has no data location, because it was not read from an archive.
    #[error("entry was not read from an archive, so it has no data to extract")]
    NoData,

    /// A parsing or serialization error from deku.
    #[error("deku error: {0:?}")]
    Deku(#[from] deku::DekuError),
}
