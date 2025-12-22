//! Validates librarium compiles for embedded ARM targets
//!
//! Build with:
//! ```sh
//! cargo +nightly build --target thumbv7em-none-eabihf
//! cargo +nightly build --target thumbv6m-none-eabi
//! ```

#![no_std]
#![no_main]

use no_std_lib::*;
use core::panic::PanicInfo;
use cortex_m_rt::entry;
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[entry]
fn main() -> ! {
    use core::mem::MaybeUninit;
    use core::ptr::addr_of_mut;
    const HEAP_SIZE: usize = 4096;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe {
        let heap_ptr = addr_of_mut!(HEAP_MEM);
        HEAP.init((*heap_ptr).as_ptr() as usize, HEAP_SIZE)
    }

    no_alloc_imports::test_archive_read();
    no_alloc_imports::test_header_fields();
    with_alloc_imports::test_archive_reader();
    with_alloc_imports::test_newc_header_creation();
    with_alloc_imports::test_odc_header_creation();
    with_alloc_imports::test_archive_writer();

    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
