mod core_frame_alloc;

use super::Frame;

pub use self::core_frame_alloc::*;

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
