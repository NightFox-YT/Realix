/* The MMIO for Realix
    Copyright(C) 2026 Alexander Silaev <thebinaryblob@gmail.com>
*/
use core::marker::PhantomData;
use core::ptr;

#[repr(transparent)]
pub struct MMIOPtr<T> {
    addr: usize,
    _phantom: PhantomData<*mut T>,
}

impl<T: Copy> MMIOPtr<T> 
{
    pub const unsafe fn new(addr: usize) -> Self 
    {
        Self 
        {
            addr, 
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn read(&self) -> T 
    {
        unsafe { ptr::read_volatile(self.addr as *const T) }
    }

    #[inline(always)]
    pub fn write(&self, val: T) 
    {
        unsafe { ptr::write_volatile(self.addr as *mut T, val) }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut T 
    {
        self.addr as *mut T
    }
}