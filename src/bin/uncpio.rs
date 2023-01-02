use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::prelude::OpenOptionsExt;
use std::path::PathBuf;

use deku::DekuContainerRead;

fn main() {
    let arg = std::env::args().nth(1).unwrap();
    let bytes = fs::read(arg).unwrap();

    let root_path = "cpio-root";
    fs::create_dir_all(root_path).unwrap();

    let (_, cpio_archive) = cpio_deku::Archive::from_bytes((&bytes, 0)).unwrap();
    for cpio in cpio_archive.objects {
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
            file.write_all(&cpio.file).unwrap();
        } else {
            // dir
            // TODO: set mode bits?
            fs::create_dir(path).unwrap();
        }
    }
}