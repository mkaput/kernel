use core::mem;
use core::ptr::Unique;

use spin::Mutex;
use volatile::Volatile;

use dev::console::Console;

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
pub struct TextBufferConsole {
    buffer: Unique<VideoMemory>,
}

impl TextBufferConsole {
    fn buffer(&mut self) -> &mut VideoMemory {
        unsafe { self.buffer.as_mut() }
    }
}

impl Console for TextBufferConsole {
    fn width(&self) -> usize {
        SCREEN_WIDTH
    }

    fn height(&self) -> usize {
        SCREEN_HEIGHT
    }

    fn clear(&mut self) {
        let blank = ScreenChar {
            char: 0,
            style: Style::new(Color::White, Color::Blue),
        };

        for row in self.buffer().iter_mut() {
            for char in row.iter_mut() {
                char.write(blank);
            }
        }
    }
}

pub static TEXT_BUFFER_CONSOLE: Mutex<TextBufferConsole> = Mutex::new(TextBufferConsole {
    buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
});
