use std::fs;

use deku::DekuContainerRead;

fn main() {
    let arg = std::env::args().nth(1).unwrap();
    let bytes = fs::read(arg).unwrap();

    let (_, cpio_archive) = cpio_deku::Archive::from_bytes((&bytes, 0)).unwrap();
    for cpio in cpio_archive.objects {
        println!("{:#x?}", cpio.header);
        println!("{:#x?}", cpio.name);
    }
}
