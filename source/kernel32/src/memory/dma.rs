// © Realix > memory: dma
// (27.07.26) v0.1
// ================

use crate::memory::frame_allocator;
use crate::memory;
use core::sync::atomic::Ordering::Relaxed;

/// Выделение физ. непрерывных фреймов для DMA (в пределах первых 16МБ)
pub fn alloc_frames(count: usize) -> Option<usize>
{
    if count == 0 { return None; }

    let mut consecutive: usize = 0;
    let mut start_frame: usize = 0;

    let search_start = memory::RESERVED_LOW_MEMORY / memory::FRAME_PAGE_SIZE;

    for frame in search_start..memory::DMA_MAX_FRAME {
        if !memory::is_bit_free(frame) {
            consecutive = 0;
            continue;
        }

        if consecutive == 0 { start_frame = frame; }
        consecutive += 1;

        if consecutive == count {
            for f in start_frame..start_frame + count { memory::mark_frame_used(f); }
            memory::FREE_FRAMES.fetch_sub(count, Relaxed);
            return Some(start_frame * memory::FRAME_PAGE_SIZE);
        }
    }
    None
}