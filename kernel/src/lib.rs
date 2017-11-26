#![feature(asm)]
#![feature(const_fn)]
#![feature(const_unique_new)]
#![feature(lang_items)]
#![feature(unique)]
#![feature(universal_impl_trait)]
#![no_std]

// FIXME: this crate is multiboot2 1.6 compliant while we use 2.0
extern crate multiboot2;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;

pub mod dev;
pub mod drv;
#[macro_use]
pub mod kio;
pub mod mem;

use dev::text_video::{TextColor, TextStyle};

/// Real kernel entry point
#[no_mangle]
pub extern "C" fn krnl_main(mb_info_addr: usize) {
    // ATTENTION: we have a very small stack and no guard page

    unsafe {
        // Set up early console ASAP, so we will be able to use `kprintln!`
        kio::early_init();

        let boot_info = multiboot2::load(mb_info_addr);

        mem::init(boot_info);
    }

    unreachable!();
}

/// TODO: The heck is this?
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

/// Kernel panic handler
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    let red = TextStyle { foreground: TextColor::Red, background: TextColor::Black };
    kio::with_output_style(red, || {
        kprintln!("PANIC in {}:{}", file, line);
        kprintln!("  {}", fmt);
    });
    loop {}
}
