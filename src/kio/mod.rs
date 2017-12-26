//! Kernel Input Output subsystem

#[macro_use]
mod macros;

pub mod idt;
pub mod port;

use core::fmt::{self, Write};

use dev::text_video::{TextStyle, TextVideo};
use dev::output_serial::OutputSerial;
use drv::gfx::vga::text_buffer::VGA_TEXT_VIDEO;

/// Performs early initialization of KIO subsystem, setting up
/// so called *early console* which enables usage of [`println!`] family macros.
///
/// No other subsystem is required to be initialized yet.
///
/// **This function should only be called once.**
///
/// [`println!`]: ./macro.println.html
pub unsafe fn early_init() {
    {
        let mut video = VGA_TEXT_VIDEO.lock();
        video.enable_cursor();
        video.clear();
    }

    println!("early console works");
}

/// Temporarily applies text style to current kernel output device.
///
/// The style is applied only if output device implements `TextVideo` trait,
/// otherwise this function is no-op and only calls `f`.
///
/// ## Examples
///
/// ```
/// use dev::text_video::{TextColor, TextStyle}
///
/// let red = TextStyle { foreground: TextColor::Red, background: TextColor::Black };
///
/// with_output_style(red, || {
///     println!("PANIC in {}:{}", file, line);
///     println!("  {}", fmt);
/// });
/// ```
pub fn with_output_style(text_style: TextStyle, f: impl FnOnce()) {
    let prev_style = {
        let mut video = VGA_TEXT_VIDEO.lock();
        let prev_style = video.current_style();
        video.set_current_style(text_style);
        prev_style
    };

    f();

    {
        let mut video = VGA_TEXT_VIDEO.lock();
        video.set_current_style(prev_style);
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    VGA_TEXT_VIDEO
        .lock()
        .writer()
        .write_fmt(args)
        .expect("KIO: Kernel Output write failure");
}
