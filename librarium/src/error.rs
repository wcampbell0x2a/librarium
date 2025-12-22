use thiserror::Error;

/// Errors generated from library
#[derive(Error, Debug)]
pub enum CpioError {
    #[error("std io error: {0}")]
    StdIo(#[from] no_std_io2::io::Error),

    #[error("deku error: {0:?}")]
    Deku(#[from] deku::DekuError),
}
