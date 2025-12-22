use no_std_io2::io::{Read, Seek, SeekFrom};

/// `Read` + `Seek`
pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

/// Private struct containing logic to read the data section from the archive
#[derive(Debug)]
pub(crate) struct ReaderWithOffset<R: ReadSeek> {
    io: R,
    /// Offset from start of file to data
    offset: u64,
}

impl<R: ReadSeek> ReaderWithOffset<R> {
    pub fn new(mut io: R, offset: u64) -> Result<Self, no_std_io2::io::Error> {
        io.seek(SeekFrom::Start(offset))?;
        Ok(Self { io, offset })
    }
}

impl<R: ReadSeek> Read for ReaderWithOffset<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, no_std_io2::io::Error> {
        self.io.read(buf)
    }
}

impl<R: ReadSeek> Seek for ReaderWithOffset<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, no_std_io2::io::Error> {
        let seek = match pos {
            SeekFrom::Start(start) => SeekFrom::Start(self.offset + start),
            seek => seek,
        };
        self.io.seek(seek).map(|x| x - self.offset)
    }
}
