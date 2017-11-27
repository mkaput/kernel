use core::fmt;

/// Common interface for output serials for messaging text
pub trait OutputSerial {
    /// Print single ASCII character
    ///
    /// Special characters are expected to be treated as their are intended, e.g. `\n` is expected
    /// to start new line.
    fn put_byte(&mut self, char: u8);

    /// Print while ASCII string
    ///
    /// UTF-8 is not expected to be supported. Default implementation simply calls `put_byte`
    /// for each byte in `s`.
    fn put_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.put_byte(byte)
        }
    }

    /// Returns wrapper that implements `fmt::Write` trait
    fn writer<'a, 'b>(&'a mut self) -> OutputSerialWriter<'b, Self>
    where
        'a: 'b,
    {
        OutputSerialWriter(self)
    }
}

pub struct OutputSerialWriter<'a, T>(&'a mut T)
where
    T: 'a + OutputSerial + ?Sized;

impl<'a, T> fmt::Write for OutputSerialWriter<'a, T>
where
    T: 'a + OutputSerial,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.put_str(s);
        Ok(())
    }
}
