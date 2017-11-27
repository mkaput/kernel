use super::output_serial::OutputSerial;

/// Represents single text color
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TextColor {
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
}

/// Represents foreground-background text color pair
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TextStyle {
    pub foreground: TextColor,
    pub background: TextColor,
}

/// Represents cursor position
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    /// Convenience shortcut
    #[inline(always)]
    pub const fn new(row: usize, col: usize) -> Cursor {
        Cursor { row, col }
    }

    /// Convenience shortcut for `{ 0, 0 }` cursor position
    #[inline(always)]
    pub const fn zero() -> Cursor {
        Cursor::new(0, 0)
    }
}

pub trait TextVideo: OutputSerial {
    /// Character width of video buffer
    fn width(&self) -> usize;

    /// Character height (number of visible lines) of video buffer
    fn height(&self) -> usize;

    /// Enables cursor
    ///
    /// Should do nothing if this operation is not supported. This function is expected to
    /// set cursor position to `{ 0, 0 }`.
    fn enable_cursor(&mut self);

    /// Disables cursor
    ///
    /// Should do nothing if this operation is not supported.
    fn disable_cursor(&mut self);

    /// Returns current cursor position
    ///
    /// Should always return `{ 0, 0 }` if cursor is not supported.
    fn cursor(&self) -> Cursor;

    /// Updates cursor position
    ///
    /// Should do nothing if this operation is not supported.
    fn set_cursor(&mut self, new_cursor: Cursor);

    /// Gets current text style
    fn current_style(&self) -> TextStyle;

    /// Sets current text style
    fn set_current_style(&mut self, new_style: TextStyle);

    /// Clears buffer with current style
    fn clear(&mut self);
}
