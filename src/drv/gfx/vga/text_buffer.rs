// I/O Ports of 6845: http://stanislavs.org/helppc/6845.html

use core::mem;
use core::ptr::Unique;

use spin::Mutex;
use volatile::Volatile;

use dev::output_serial::OutputSerial;
use dev::text_video::{Cursor, TextColor, TextStyle, TextVideo};
use kio::port::Port;

pub const VGA_TEXT_BUFFER_ADDR: usize = 0xb8000;

/// 6845 index register, selects which register [0-11h]
/// is to be accessed through port 3B5/3D5
const CMD_PORT: Port<u8> = unsafe { Port::new(0x3d4) };

/// 6845 data register [0-11h] selected by port 3B4/3D4,
/// registers 0C-0F may be read.  If a read occurs without the
/// adapter installed, FFh is returned.
const DATA_PORT: Port<u8> = unsafe { Port::new(0x3d5) };

/// TODO: What's this?
const WAT_PORT: Port<u8> = unsafe { Port::new(0x3e0) };

/// Represents single VGA color
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

impl From<TextColor> for Color {
    fn from(color: TextColor) -> Self {
        match color {
            TextColor::Black => Color::Black,
            TextColor::Blue => Color::Blue,
            TextColor::Green => Color::Green,
            TextColor::Cyan => Color::Cyan,
            TextColor::Red => Color::Red,
            TextColor::Magenta => Color::Magenta,
            TextColor::Brown => Color::Brown,
            TextColor::LightGray => Color::LightGray,
            TextColor::DarkGray => Color::DarkGray,
            TextColor::LightBlue => Color::LightBlue,
            TextColor::LightGreen => Color::LightGreen,
            TextColor::LightCyan => Color::LightCyan,
            TextColor::LightRed => Color::LightRed,
            TextColor::Pink => Color::Pink,
            TextColor::Yellow => Color::Yellow,
            TextColor::White => Color::White,
        }
    }
}

impl From<Color> for TextColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Black => TextColor::Black,
            Color::Blue => TextColor::Blue,
            Color::Green => TextColor::Green,
            Color::Cyan => TextColor::Cyan,
            Color::Red => TextColor::Red,
            Color::Magenta => TextColor::Magenta,
            Color::Brown => TextColor::Brown,
            Color::LightGray => TextColor::LightGray,
            Color::DarkGray => TextColor::DarkGray,
            Color::LightBlue => TextColor::LightBlue,
            Color::LightGreen => TextColor::LightGreen,
            Color::LightCyan => TextColor::LightCyan,
            Color::LightRed => TextColor::LightRed,
            Color::Pink => TextColor::Pink,
            Color::Yellow => TextColor::Yellow,
            Color::White => TextColor::White,
        }
    }
}

/// Represents foreground-background VGA color pair
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
struct Style(u8);

impl Style {
    /// Creates new VGA character style
    const fn new(foreground: Color, background: Color) -> Style {
        Style((background as u8) << 4 | (foreground as u8))
    }

    /// Gets background color
    // TODO: Make this const fn
    fn background(&self) -> Color {
        unsafe { mem::transmute(((self.0 & 0b11110000) >> 4)) }
    }

    /// Gets foreground color
    // TODO: Make this const fn
    fn foreground(&self) -> Color {
        unsafe { mem::transmute((self.0 & 0b00001111)) }
    }
}

impl From<TextStyle> for Style {
    fn from(style: TextStyle) -> Self {
        Style::new(style.foreground.into(), style.background.into())
    }
}

impl From<Style> for TextStyle {
    fn from(style: Style) -> Self {
        TextStyle {
            foreground: style.foreground().into(),
            background: style.background().into(),
        }
    }
}

/// Represents single, VGA styled character
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
struct ScreenChar {
    /// ASCII character code
    char: u8,
    /// Character style
    style: Style,
}

/// The height of VGA text buffer
const SCREEN_HEIGHT: usize = 25;

/// The width of VGA text buffer
const SCREEN_WIDTH: usize = 80;

/// Represents VGA text buffer (`0xb8000`) memory
type VideoMemory = [[Volatile<ScreenChar>; SCREEN_WIDTH]; SCREEN_HEIGHT];

/// Represents console backed by VGA text memory
pub struct VgaTextVideo {
    current_style: Style,
    cursor: Cursor,
    buffer: Unique<VideoMemory>,
}

impl VgaTextVideo {
    fn buffer(&mut self) -> &mut VideoMemory {
        unsafe { self.buffer.as_mut() }
    }

    fn raw_put_byte(&mut self, char: u8) {
        if char == b'\n' {
            self.newline();
        } else {
            let screen_char = ScreenChar {
                char,
                style: self.current_style,
            };
            let mut cur = self.cursor;

            self.buffer()[cur.row][cur.col].write(screen_char);

            if cur.col == SCREEN_WIDTH - 1 {
                self.newline()
            } else {
                cur.col += 1;
                self.cursor = cur;
            }
        }
    }

    fn newline(&mut self) {
        let mut cur = self.cursor;

        if cur.row == SCREEN_HEIGHT - 1 {
            for row in 1..SCREEN_HEIGHT {
                for col in 0..SCREEN_WIDTH {
                    let buffer = self.buffer();
                    let ch = buffer[row][col].read();
                    buffer[row - 1][col].write(ch);
                }
            }

            let blank = ScreenChar {
                char: 0,
                style: self.current_style,
            };
            for char in self.buffer()[SCREEN_HEIGHT - 1].iter_mut() {
                char.write(blank);
            }

            cur.col = 0;
            self.cursor = cur;
        } else {
            cur.row += 1;
            cur.col = 0;
            self.cursor = cur;
        }
    }

    fn update_cursor(&mut self) {
        let pos = (self.cursor.row * SCREEN_WIDTH + self.cursor.col) as u16;

        // Select "Cursor address (LSB)" register
        CMD_PORT.write(0x0f);

        // Write least significant byte of cursor position
        DATA_PORT.write((pos & 0xff) as u8);

        // Select "Cursor address (MSB) register
        CMD_PORT.write(0x0e);

        // Write most significant byte of cursor position
        DATA_PORT.write(((pos >> 8) & 0xff) as u8);
    }
}

impl OutputSerial for VgaTextVideo {
    fn put_byte(&mut self, char: u8) {
        self.raw_put_byte(char);
        self.update_cursor()
    }

    fn put_str(&mut self, s: &str) {
        for char in s.bytes() {
            self.raw_put_byte(char);
        }
        self.update_cursor()
    }
}

impl TextVideo for VgaTextVideo {
    fn width(&self) -> usize {
        SCREEN_WIDTH
    }

    fn height(&self) -> usize {
        SCREEN_HEIGHT
    }

    fn enable_cursor(&mut self) {
        self.cursor = Cursor::zero();

        // Select "Cursor start (scan line)" register
        CMD_PORT.write(0x0a);

        let scan_line_start = (DATA_PORT.read() & 0xc0) | 14;
        DATA_PORT.write(scan_line_start);

        // Select "Cursor end (scan line)" register
        CMD_PORT.write(0x0b);

        let scan_line_end = (WAT_PORT.read() & 0xe0) | 15;
        DATA_PORT.write(scan_line_end);
    }

    fn disable_cursor(&mut self) {
        // Select "Cursor start (scan line)" register
        CMD_PORT.write(0x0a);

        // Disable cursor (bit 5)
        DATA_PORT.write(0b0010_0000);
    }

    fn cursor(&self) -> Cursor {
        self.cursor
    }

    fn set_cursor(&mut self, new_cursor: Cursor) {
        self.cursor = new_cursor;
        self.update_cursor();
    }

    fn current_style(&self) -> TextStyle {
        self.current_style.into()
    }

    fn set_current_style(&mut self, new_style: TextStyle) {
        self.current_style = new_style.into()
    }

    fn clear(&mut self) {
        let blank = ScreenChar {
            char: 0,
            style: self.current_style,
        };

        for row in self.buffer().iter_mut() {
            for char in row.iter_mut() {
                char.write(blank);
            }
        }

        self.set_cursor(Cursor::zero());
    }
}

/// The instance of VGA text buffer
pub static VGA_TEXT_VIDEO: Mutex<VgaTextVideo> = Mutex::new(VgaTextVideo {
    current_style: Style::new(Color::LightGray, Color::Black),
    cursor: Cursor::zero(),
    buffer: unsafe { Unique::new_unchecked(VGA_TEXT_BUFFER_ADDR as *mut _) },
});
