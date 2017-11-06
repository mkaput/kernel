#![feature(asm)]
#![feature(const_fn)]
#![feature(const_unique_new)]
#![feature(lang_items)]
#![feature(unique)]
#![no_std]

#[allow(unused_extern_crates)]
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;

mod dev;
mod drv;

use core::fmt::Write;

use dev::output_serial::OutputSerial;
use dev::text_video::TextVideo;
use dev::text_video::TextColor::*;
use drv::gfx::vga::text_buffer::VGA_TEXT_VIDEO;

/// Real kernel entry point
#[no_mangle]
pub extern "C" fn krnl_main(_mb_info: usize) {
    // ATTENTION: we have a very small stack and no guard page

    {
        // Set up early console ASAP
        let mut video = VGA_TEXT_VIDEO.lock();
        video.enable_cursor();
        video.clear();

        video.put_str("early console works\n");

        let mut s = video.current_style();
        let colors = [
            Black,
            Blue,
            Green,
            Cyan,
            Red,
            Magenta,
            Brown,
            LightGray,
            DarkGray,
            LightBlue,
            LightGreen,
            LightCyan,
            LightRed,
            Pink,
            Yellow,
            White,
        ];
        for &bg in colors.iter() {
            for &fg in colors.iter() {
                s.foreground = fg;
                s.background = bg;
                video.set_current_style(s);
                write!(video.fmt(), "#").unwrap();
            }
        }
        writeln!(video.fmt()).unwrap();
    }

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
