use multiboot2::{MemoryArea, MemoryAreaIter};

use super::{Frame, FrameAlloc};

pub struct CoreFrameAlloc {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    reserved_frames: [(Frame, Frame); 2],
}

impl CoreFrameAlloc {
    /// Constructs new core frame allocator
    pub fn new(areas: MemoryAreaIter, reserved_frames: [(Frame, Frame); 2]) -> CoreFrameAlloc {
        let mut alloc = CoreFrameAlloc {
            next_free_frame: Frame::containing_address(0),
            current_area: None,
            areas,
            reserved_frames,
        };
        alloc.pick_next_area();
        alloc
    }

    /// Chooses the area with the minimal base address that still has free frames,
    /// and updates next_free_frame to first frame in picked area.
    fn pick_next_area(&mut self) {
        self.current_area = self.areas
            .clone()
            .filter(|area| last_frame_of_area(&area) >= self.next_free_frame)
            .min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }

    /// Returns Some(<first frame after first reserved region>) if frame is reserved
    /// or None, though does not check if returned frame is in available memory area
    /// bounds nor in next reserved region.
    fn check_frame_unreserved(&self, frame: &Frame) -> Option<Frame> {
        for &(ref start, ref end) in self.reserved_frames.iter() {
            if start <= frame && frame <= end {
                return Some(next_frame(frame));
            }
        }

        None
    }
}

impl FrameAlloc for CoreFrameAlloc {
    fn alloc(&mut self) -> Option<Frame> {
        // TODO: Pick first reusable free frame

        // If there are no reusable frames, try to pick untouched one.
        while let Some(area) = self.current_area {
            let frame = self.next_free_frame.clone();

            // Continue if we have reached end of current memory area.
            if frame > last_frame_of_area(area) {
                self.pick_next_area();
                continue;
            }

            // If picked frame is reserved, set next frame number to
            // last frame number of colliding reserved range and continue.
            if let Some(next_frame) = self.check_frame_unreserved(&frame) {
                self.next_free_frame = next_frame;
                continue;
            }

            // Otherwise, picked frame is available to allocating.
            self.next_free_frame = next_frame(&self.next_free_frame);
            return Some(frame);
        }

        // There are no free frames fo reusing and no untouched frames.
        // In other words we are out of memory now.
        None
    }

    fn dealloc(&mut self, _frame: Frame) {
        // TODO: Reusing frames
        println!("FRAME DEALLOCATING NOT IMPLEMENTED YET");
    }
}

#[inline]
fn next_frame(frame: &Frame) -> Frame {
    Frame {
        number: frame.number + 1,
    }
}

#[inline]
fn last_frame_of_area(area: &MemoryArea) -> Frame {
    let addr = area.base_addr + area.length - 1;
    Frame::containing_address(addr as usize)
}
