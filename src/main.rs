use std::fs;

use deku::DekuContainerRead;

fn main() {
    let bytes = fs::read("test_cpio/sample.cpio").unwrap();
    println!("{:x?}", bytes);

    let (_, cpio_archive) = cpio_deku::Archive::from_bytes((&bytes, 0)).unwrap();
    println!("{:#x?}", cpio_archive);
}
