use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use crate::memory::memory;

pub struct RealixSlabAllocator;

unsafe impl GlobalAlloc for RealixSlabAllocator 
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 
    {
        let size = layout.size() as u32;
        let ptr = memory::kmalloc(size);
        ptr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) 
    {
        if !ptr.is_null() 
        {
            memory::kfree(ptr as *mut core::ffi::c_void);
        }
    }
}

#[global_allocator]
static ALLOCATOR: RealixSlabAllocator = RealixSlabAllocator;