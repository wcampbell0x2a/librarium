use std::io;

use thiserror::Error;

/// Errors generated from library
#[derive(Error, Debug)]
pub enum CpioError {
    #[error("std io error: {0}")]
    StdIo(#[from] io::Error),

    #[error("deku error: {0:?}")]
    Deku(#[from] deku::DekuError),
}
