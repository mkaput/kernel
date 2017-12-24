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
