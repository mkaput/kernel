//! Kernel Input Output subsystem

use core::fmt::{self, Write};

use dev::text_video::{TextStyle, TextVideo};
use dev::output_serial::OutputSerial;
use drv::gfx::vga::text_buffer::VGA_TEXT_VIDEO;

/// Macro for printing to current kernel output serial device. This is the
/// preferred way for doing kernel logging.
///
/// Equivalent to the [`kprintln!`] macro except that a newline is not printed at
/// the end of the message.
///
/// Note that kernel logging output is sometimes line-buffered by default and
/// may be necessary to flush output buffer to ensure the output is emitted immediately.
/// There is no standard way for doing it though.
///
/// Use the `format!` syntax to write data to the standard output.
///
/// [`kprintln!`]: ./macro.kprintln.html
///
/// # Panics
///
/// Panics if writing to kernel output fails.
///
/// # Examples
///
/// ```
/// kprint!("this ");
/// kprint!("will ");
/// kprint!("be ");
/// kprint!("on ");
/// kprint!("the ");
/// kprint!("same ");
/// kprint!("line ");
///
/// kprint!("this string has a newline, why not choose println! instead?\n");
/// ```
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::kio::_kprint(format_args!($($arg)*)));
}

/// Macro for printing to current kernel output serial device, with a newline. This is the
/// preferred way for doing kernel logging.
///
/// *Newline* means the LINE FEED character (`\n`/`U+000A`) alone
/// (no additional CARRIAGE RETURN (`\r`/`U+000D`).
///
/// Use the `format!` syntax to write data to the standard output.
///
/// # Panics
///
/// Panics if writing to kernel output fails.
///
/// # Examples
///
/// ```
/// kprintln!(); // prints just a newline
/// kprintln!("hello there!");
/// kprintln!("format {} arguments", "some");
/// ```
#[macro_export]
macro_rules! kprintln {
    () => (kprint!("\n"));
    ($fmt:expr) => (kprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (kprint!(concat!($fmt, "\n"), $($arg)*));
}

/// Performs early initialization of KIO subsystem, setting up
/// so called *early console* which enables usage of [`kprintln!`] family macros.
///
/// No other subsystem is required to be initialized yet.
///
/// **This function should only be called once.**
///
/// [`kprintln!`]: ./macro.kprintln.html
pub unsafe fn early_init() {
    {
        let mut video = VGA_TEXT_VIDEO.lock();
        video.enable_cursor();
        video.clear();
    }

    kprintln!("early console works");
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
///     kprintln!("PANIC in {}:{}", file, line);
///     kprintln!("  {}", fmt);
/// });
/// ```
pub fn with_output_style(text_style: TextStyle, f: impl FnOnce() -> ()) {
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
pub fn _kprint(args: fmt::Arguments) {
    VGA_TEXT_VIDEO
        .lock()
        .writer()
        .write_fmt(args)
        .expect("KIO: Kernel Output write failure");
}
