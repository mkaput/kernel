use super::{PhysicalAddress, PAGE_SIZE};

/// Represents physical memory frame.
///
/// Frames are comparable via their start offsets in physical memory.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Frame {
    pub(super) number: usize,
}

impl Frame {
    // We deliberately do not implement Clone for Frame, because cloning frame
    // does not make real sense. This function is only implementation internal.
    pub(super) const fn clone(&self) -> Frame {
        Frame {
            number: self.number,
        }
    }

    /// Returns `Frame` containing given physical `address`.
    pub const fn containing_address(address: PhysicalAddress) -> Frame {
        Frame {
            number: address / PAGE_SIZE,
        }
    }

    /// Returns frame start physical address.
    pub const fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    /// Returns first physical address after frame end.
    pub const fn end_address(&self) -> PhysicalAddress {
        (self.number + 1) * PAGE_SIZE
    }

    pub(super) fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter { start, end }
    }
}


pub(super) struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}
