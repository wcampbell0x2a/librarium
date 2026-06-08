use thiserror::Error;

/// Errors returned by this library.
#[derive(Error, Debug)]
pub enum CpioError {
    /// An I/O error occurred.
    #[error("std io error: {0}")]
    StdIo(#[from] no_std_io2::io::Error),

    /// A parsing or serialization error from deku.
    #[error("deku error: {0:?}")]
    Deku(#[from] deku::DekuError),
}
