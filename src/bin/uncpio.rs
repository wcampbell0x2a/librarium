use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Write};
use std::os::unix::prelude::OpenOptionsExt;
use std::path::PathBuf;

use deku::DekuContainerRead;

fn main() {
    let arg = std::env::args().nth(1).unwrap();
    let file = File::options().read(true).open(arg).unwrap();
    let mut reader = BufReader::new(file);

    let root_path = "cpio-root";
    fs::create_dir_all(root_path).unwrap();

    let (_, cpio_archive) = cpio_deku::Archive::from_reader((&mut reader, 0)).unwrap();
    for cpio in &cpio_archive.objects {
        println!("{:#x?}", cpio.header);
        println!("{:#x?}", cpio.name);

        let mut path: PathBuf = root_path.into();
        path.push(cpio.name.to_str().unwrap());

        // file
        if cpio.header.filesize.value > 0 {
            let mut options = OpenOptions::new();
            options.mode(cpio.header.mode.value);
            options.create(true);
            options.write(true);
            let mut file = options.open(path).unwrap();
            let _ = file.write_all(&cpio.file);
        } else {
            // dir
            // TODO: set mode bits?
            let _ = fs::create_dir(path);
        }
    }
}
