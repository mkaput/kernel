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
}

// TODO Implement fmt::Write for T: OutputSerial
