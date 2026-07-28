// © Realix > Memory (Map)
// (04.07.26) v0.08
// ================
#![allow(dead_code)]
#![allow(non_camel_case_types)]
use core::ffi::c_void;
/// Структура записи карты памяти E820
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct E820Entry {
    pub address: u64,
    pub size: u64,
    pub seg_type: u32,
    pub attributes: u32,
}

/// Структура карты памяти E820 (64 - запас)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct E820Map {
    pub entry_count: u16,
    pub map: [E820Entry; 64],
}

pub const PCINFO_ADDR: usize = 0x4500;

pub const PTE_PRESENT: u32 = 1 << 0;
pub const PTE_WRITE: u32 = 1 << 1;
pub const PTE_USER: u32 = 1 << 2;
pub const PTE_PWT: u32 = 1 << 3;
pub const PTE_PCD: u32 = 1 << 4;
pub const PTE_ACCESSED: u32 = 1 << 5;
pub const PTE_DIRTY: u32 = 1 << 6;

pub const VMA_READ: u32 = 1 << 0;
pub const VMA_WRITE: u32 = 1 << 1;
pub const VMA_EXEC: u32 = 1 << 2;
pub const VMA_KERNEL: u32 = 1 << 3;
pub const VMA_USER: u32 = 1 << 4;

#[repr(C)]
pub struct vma_t {
    pub virt_start: u32,
    pub virt_end: u32,
    pub flags: u32,
    pub next: *mut vma_t,
}

#[repr(C)]
pub struct vmm_t {
    pub pgdir: *mut u32,
    pub vmas: *mut vma_t,
}

// --- SLAB ---

#[repr(C)]
pub struct kmem_cache_t {
    _private: [u8; 0], // Opaque handle
}

extern "C" {
    // Page Allocator
    pub fn init_kernel_page_allocator();
    pub fn rlxalloc_page() -> *mut c_void;

    // Slab / Kmalloc
    pub fn kmalloc_init();
    pub fn kmalloc(size: u32) -> *mut c_void;
    pub fn kfree(ptr: *mut c_void);

    pub fn kmem_cache_create(name: *const i8, obj_size: u32) -> *mut kmem_cache_t;
    pub fn kmem_cache_alloc(cache: *mut kmem_cache_t) -> *mut c_void;
    pub fn kmem_cache_free(cache: *mut kmem_cache_t, obj: *mut c_void);
    pub fn kmem_cache_destroy(cache: *mut kmem_cache_t);

    // VMM
    pub static mut kernel_vmm: *mut vmm_t;

    pub fn vmm_init();
    pub fn vmm_create() -> *mut vmm_t;
    pub fn vmm_destroy(vmm: *mut vmm_t);
    pub fn vmm_switch(vmm: *mut vmm_t);

    pub fn vmm_map_page(vmm: *mut vmm_t, virt: u32, phys: u32, flags: u32) -> bool;
    pub fn vmm_unmap_page(vmm: *mut vmm_t, virt: u32);
    pub fn vmm_mmap(vmm: *mut vmm_t, virt: u32, size: u32, flags: u32) -> u32;
    pub fn vmm_munmap(vmm: *mut vmm_t, virt: u32, size: u32);
    pub fn vmm_virt_to_phys(vmm: *mut vmm_t, virt: u32) -> u32;
}