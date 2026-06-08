//! Internal trait for writing mutable structures via deku.

use deku::DekuError;
use deku::writer::Writer;
use no_std_io2::io::{Seek, Write};

/// A deku-compatible writer that mutates `self` during serialization.
///
/// Unlike [`DekuWriter`](deku::DekuWriter), this trait takes `&mut self`,
/// allowing implementations to consume internal state (e.g., reading from an
/// embedded reader) while writing.
pub(crate) trait MutWriter<Ctx = ()> {
    fn to_mutwriter<W: Write + Seek>(
        &mut self,
        deku_writer: &mut Writer<W>,
        ctx: Ctx,
    ) -> core::result::Result<(), DekuError>;
}
