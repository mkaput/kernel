pub trait Console {
    /// Character width of console
    fn width(&self) -> usize;

    /// Character height (number of visible lines) of console
    fn height(&self) -> usize;

    /// Clears console buffer with current style
    fn clear(&mut self);
}
