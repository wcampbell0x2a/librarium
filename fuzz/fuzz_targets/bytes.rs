#![no_main]

use libfuzzer_sys::fuzz_target;
use librarium::{ArchiveReader, NewcHeader};

fuzz_target!(|data: Vec<u8>| {
    let mut reader = std::io::Cursor::new(data);

    // doesn't crash
    ArchiveReader::<NewcHeader>::from_reader_with_offset(&mut reader, 0);
});
