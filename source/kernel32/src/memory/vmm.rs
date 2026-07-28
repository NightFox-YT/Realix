use crate::memory::memory;
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;

pub struct Spinlock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        // Ждем, пока lock станет false, и атомарно меняем на true
        while self.lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        SpinlockGuard { starving_lock: self }
    }
}

pub struct SpinlockGuard<'a, T> {
    starving_lock: &'a Spinlock<T>,
}

impl<T> core::ops::Deref for SpinlockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.starving_lock.data.get() }
    }
}

impl<T> core::ops::DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.starving_lock.data.get() }
    }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        self.starving_lock.lock.store(false, Ordering::Release);
    }
}

pub static KERNEL_VMM: Spinlock<memory::vmm_t> = Spinlock::new(memory::vmm_t {
    pgdir: core::ptr::null_mut(),
    vmas: core::ptr::null_mut(),
});

pub struct KernelVmm;

impl KernelVmm 
{
    pub fn map_mmio_region(phys_addr: u32, size: u32) -> Result<u32, ()> 
    {
        let mut vmmguard = KERNEL_VMM.lock();
        let vmm_ptr: *mut memory::vmm_t = &mut *vmmguard as *mut memory::vmm_t;

        unsafe 
        {
            // Запрашиваем виртуальный регион в адресном пространстве ядра
            let flags = memory::VMA_READ | memory::VMA_WRITE | memory::VMA_KERNEL;
            let virt_base = memory::vmm_mmap(vmm_ptr, 0, size, flags);

            if virt_base == 0 
            {
                return Err(());
            }

            let pages = (size + 4095) / 4096;
            for i in 0..pages 
            {
                let offset = i * 4096;
                let page_flags = memory::PTE_PRESENT | memory::PTE_WRITE | memory::PTE_PCD | memory::PTE_PWT;
                
                if !memory::vmm_map_page(vmm_ptr, virt_base + offset, phys_addr + offset, page_flags) 
                {
                    // В случае ошибки снимаем всё
                    memory::vmm_munmap(vmm_ptr, virt_base, size);
                    return Err(());
                }
            }

            Ok(virt_base)
        }
    }
}