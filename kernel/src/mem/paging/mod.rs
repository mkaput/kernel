mod core_frame_alloc;

pub use self::core_frame_alloc::CoreFrameAlloc;

pub type VirtualAddress = usize;
pub type PhysicalAddress = usize;

pub const PAGE_SIZE: usize = 4096;

/// Represents physical memory frame.
///
/// Frames are comparable via their start offsets in physical memory.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

impl Frame {
    // We deliberately do not implement Clone for Frame, because cloning frame
    // does not make real sense. This function is only implementation internal.
    const fn clone(&self) -> Frame {
        Frame { number: self.number }
    }

    /// Returns `Frame` containing given physical `address`.
    pub const fn containing_address(address: PhysicalAddress) -> Frame {
        Frame { number: address / PAGE_SIZE }
    }
}

/// Common interface for *frame allocators*.
///
/// Frame allocator is a routine which manages used and free frames,
/// it is responsible for providing free frames on request.
pub trait FrameAlloc {
    /// Allocate unused frame, or return `None` if all frames are used.
    fn alloc(&mut self) -> Option<Frame>;

    /// Return frame back to free frames pool.
    fn dealloc(&mut self, frame: Frame);
}
