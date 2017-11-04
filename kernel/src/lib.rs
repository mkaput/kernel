#![feature(asm)]
#![feature(const_fn)]
#![feature(const_unique_new)]
#![feature(inclusive_range_syntax)]
#![feature(lang_items)]
#![feature(unique)]
#![no_std]

#[allow(unused_extern_crates)]
extern crate rlibc;
extern crate spin;
extern crate volatile;

mod dev;
mod drv;

use dev::console::Console;

/// Real kernel entry point
#[no_mangle]
pub extern "C" fn krnl_main(mb_info: usize) {
    // ATTENTION: we have a very small stack and no guard page

    drv::gfx::vga::text_buffer::TEXT_BUFFER_CONSOLE.lock().clear();

    loop {}
}

/// TODO: The heck is this?
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

/// Kernel panic handler
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt() -> ! {
    loop {}
}
