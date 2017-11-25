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

mod dev;
mod drv;
#[macro_use]
mod kio;

use dev::text_video::{TextColor, TextStyle, TextVideo};
use drv::gfx::vga::text_buffer::VGA_TEXT_VIDEO;

/// Real kernel entry point
#[no_mangle]
pub extern "C" fn krnl_main(mb_info_addr: usize) {
    // ATTENTION: we have a very small stack and no guard page

    // Set up early console ASAP, so we will be able to use `kprintln!`
    init_early_console();

    kprintln!("{:x}", mb_info_addr);

    let boot_info = unsafe { multiboot2::load(mb_info_addr) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    kprintln!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        kprintln!(
            "  start: 0x{:x}, length: 0x{:x}",
            area.base_addr,
            area.length
        );
    }

    panic!("Kernel dead end reached!");
}

fn init_early_console() {
    {
        let mut video = VGA_TEXT_VIDEO.lock();
        video.enable_cursor();
        video.clear();
    }
    kprintln!("early console works");
}

/// TODO: The heck is this?
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

/// Kernel panic handler
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    kio::with_output_style(
        TextStyle {
            foreground: TextColor::Red,
            background: TextColor::Black,
        },
        || {
            kprintln!("PANIC in {}:{}", file, line);
            kprintln!("  {}", fmt);
        },
    );
    loop {}
}
