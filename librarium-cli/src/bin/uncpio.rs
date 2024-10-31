use std::fs::{self, File, OpenOptions};
use std::io::SeekFrom;
use std::io::{BufReader, Seek};
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use librarium::{ArchiveReader, CpioHeader, CpioReader, NewcHeader, OdcHeader};
use log::{error, info};

use clap::builder::styling::*;
pub fn styles() -> clap::builder::Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
        .valid(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .invalid(AnsiColor::Yellow.on_default() | Effects::BOLD)
}

#[derive(Copy, Clone, ValueEnum)]
enum Format {
    Odc,
    Newc,
}

/// tool to extract and list cpio filesystems
#[derive(Parser)]
#[command(author,
          version,
          name = "uncpio",
          max_term_width = 98,
          styles = styles(),
)]
struct Args {
    /// CPIO path
    archive: PathBuf,

    // #[arg(short, long, default_value_t = "newc")]
    format: Format,

    /// Skip BYTES at the start of FILESYSTEM
    #[arg(short, long, default_value_t = 0, name = "BYTES")]
    offset: u64,

    /// Extract to [PATHNAME]
    #[arg(short, long, default_value = "out", name = "PATHNAME")]
    dest: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let mut file = BufReader::new(File::open(args.archive).unwrap());
    file.seek(SeekFrom::Start(args.offset)).unwrap();

    // Extract all
    match args.format {
        Format::Odc => {
            let mut archive: ArchiveReader<OdcHeader> =
                match ArchiveReader::from_reader_with_offset(&mut file, args.offset) {
                    Ok(a) => a,
                    Err(e) => {
                        error!("could not read archive: {e}");
                        return;
                    }
                };
            let len = archive.objects.inner.len();
            for object in &archive.objects.inner[..len - 1] {
                let filepath = Path::new(&args.dest).join(object.header.name());

                info!("extracting: {:?} -> {:02x?}", object.header.name(), filepath);
                if object.header.filesize() != 0 {
                    let _ = fs::create_dir_all(filepath.parent().unwrap());
                    let mut out = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(filepath)
                        .unwrap();
                    archive.reader.extract_data(object, &mut out).unwrap();
                }
            }
        }
        Format::Newc => {
            let mut archive: ArchiveReader<NewcHeader> =
                match ArchiveReader::from_reader_with_offset(&mut file, args.offset) {
                    Ok(a) => a,
                    Err(e) => {
                        error!("could not read archive: {e}");
                        return;
                    }
                };
            let len = archive.objects.inner.len();
            for object in &archive.objects.inner[..len - 1] {
                let filepath = Path::new(&args.dest).join(object.header.name());

                info!("extracting: {:?} -> {:02x?}", object.header.name(), filepath);
                if object.header.filesize() != 0 {
                    let _ = fs::create_dir_all(filepath.parent().unwrap());
                    let mut out = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(filepath)
                        .unwrap();
                    archive.reader.extract_data(object, &mut out).unwrap();
                }
            }
        }
    }
}
