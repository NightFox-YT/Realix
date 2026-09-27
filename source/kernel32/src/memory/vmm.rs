// © Realix > memory: vmm
// (27.07.26) v0.1
// ================
// ! Реализация двухуровневого страничного аллокатора для i686+ (Page directory + Page Table но без PAE
// и без NX - NX не доступен для i686)
// TODO: Move kernel to higher half (0xC0000000) | Переместить ядро в высшую память.

use core::arch::asm;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use crate::memory::{frame_allocator, dma};
use crate::memory::{FRAME_MAX_PHYS_MEMORY, FRAME_PAGE_SIZE};

// Количество записей в Page Dir / Page Table
const ENTRIES: usize = 1024;

// PDE/PTE флаги(младшие 12 бит. Старшие 20 бит - физ. адрес фрейма).
pub const PTE_PRESENT: u32 = 1 << 0;
pub const PTE_WRITEABLE: u32 = 1 << 1;
pub const PTE_USER: u32 = 1 << 2;
pub const PTE_PWT: u32 = 1 << 3; // Write-Through
pub const PTE_PCD: u32 = 1 << 4; // Cache Disable
pub const PTE_ACCESSED: u32 = 1 << 5;
pub const PTE_DIRTY: u32 = 1 << 6; // Только PTE
pub const PTE_GLOBAL: u32 = 1 << 8; // Только PTE, требует включенного CR4.PGE

const ADDR_MASK: u32 = 0xFFFF_F000;

pub const KERNEL_RW: u32 = PTE_PRESENT | PTE_WRITEABLE;
pub const KERNEL_RO: u32 = PTE_PRESENT;
pub const KERNEL_DMA: u32 = PTE_PRESENT | PTE_WRITEABLE | PTE_PCD | PTE_PWT;

const RECURSIVE_PDE_INDEX: usize = 1023;
const RECURSIVE_PD_VIRT: usize = 0xFFFF_F000;
const RECURSIVE_PT_BASE: usize = 0xFFC0_0000;

#[repr(C, align(4096))]
struct PageDirectory {
    entries: [u32; ENTRIES],
}

static mut KERNEL_PAGE_DIRECTORY: PageDirectory = PageDirectory { entries: [0; ENTRIES] };

static PAGING_ENABLED: AtomicBool = AtomicBool::new(false);

fn get_pd_ptr() -> *mut u32
{
    if PAGING_ENABLED.load(Relaxed) {
        RECURSIVE_PD_VIRT as *mut u32
    } else {
        unsafe { (&raw mut KERNEL_PAGE_DIRECTORY) as *mut u32 }
    }
}

fn get_pt_ptr(pd_index: usize, pde: u32) -> *mut u32
{
    if PAGING_ENABLED.load(Relaxed) {
        (RECURSIVE_PT_BASE + (pd_index << 12)) as *mut u32
    } else {
        (pde & ADDR_MASK) as *mut u32
    }
}

/// Инициализация VMM: identity-map всего пула + включение страниц.
/// ! - Однопоточный вызов из ядра при старте, frame_allocator::init должен быть ДО этой функции.
pub fn init()
{
    unsafe
    {
        let pd_ptr = (&raw mut KERNEL_PAGE_DIRECTORY) as *mut u32;
        for i in 0..ENTRIES { pd_ptr.add(i).write(0) };
    }

    let mut addr: usize = 0;
    while addr < FRAME_MAX_PHYS_MEMORY {
        map_page(addr, addr, KERNEL_RW);
        let next = addr.saturating_add(FRAME_PAGE_SIZE);
        if next == addr {break;}
        addr = next;
    }

    let pd_phys = unsafe { (&raw mut KERNEL_PAGE_DIRECTORY) as u32 };
    unsafe {
        KERNEL_PAGE_DIRECTORY.entries[RECURSIVE_PDE_INDEX] =
            (pd_phys & ADDR_MASK) | PTE_PRESENT | PTE_WRITEABLE;
    }

    let mut tmp: u32;

    unsafe
    {
        asm!(
            "mov cr3, {pd}",
            "mov {tmp}, cr0",
            "or {tmp}, 0x80000000", // CR0.PG - бит 31, включает трансляцию адресов
            "mov cr0, {tmp}",
            pd = in(reg) pd_phys,
            tmp = out(reg) tmp,
            options(nostack)
        );
    }

    PAGING_ENABLED.store(true, Relaxed);
}

pub fn map_page(virt: usize, phys: usize, flags: u32)
{
    let pd_index = (virt >> 22) & 0x3FF;
    let pt_index = (virt >> 12) & 0x3FF;

    let pd = get_pd_ptr();
    let mut pde = unsafe { pd.add(pd_index).read() };

    if pde & PTE_PRESENT == 0 {
        let pt_frame = frame_allocator::alloc_frame()
            .expect("vmm: map_page: No free frames for new Page Table.");

        pde = (pt_frame as u32 & ADDR_MASK) | PTE_PRESENT | PTE_WRITEABLE | PTE_USER;
        unsafe { pd.add(pd_index).write(pde) };

        let pt_ptr = get_pt_ptr(pd_index, pde);
        unsafe
        {
            for i in 0..ENTRIES {
                pt_ptr.add(i).write(0);
            }
        }
    }

    let pt_ptr = get_pt_ptr(pd_index, pde);
    unsafe
    {
        pt_ptr.add(pt_index)
            .write((phys as u32 & ADDR_MASK) | (flags & !ADDR_MASK) | PTE_PRESENT);
    }

    invalidate_page(virt);
}

pub fn unmap_page(virt: usize)
{
    let pd_index = (virt >> 22) & 0x3FF;
    let pt_index = (virt >> 12) & 0x3FF;

    let pd_ptr = get_pd_ptr();
    let pde = unsafe { pd_ptr.add(pd_index).read() };
    if pde & PTE_PRESENT == 0 { return; }

    let pt_ptr = get_pt_ptr(pd_index, pde);
    unsafe { pt_ptr.add(pt_index).write(0) };

    invalidate_page(virt);
}

pub fn virt_to_phys(virt: usize) -> Option<usize>
{
    let pd_index = (virt >> 22) & 0x3FF;
    let pt_index = (virt >> 12) & 0x3FF; // Copy that shit!!!!

    let offset = virt & 0xFFF;

    let pd_ptr = get_pd_ptr();
    let pde = unsafe { pd_ptr.add(pd_index).read() }; // and this
    if pde & PTE_PRESENT == 0 { return None; }

    let pt_ptr = get_pt_ptr(pd_index, pde);
    let pte = unsafe { pt_ptr.add(pt_index).read() };

    if pte & PTE_PRESENT == 0 { return None; }

    Some(((pte & ADDR_MASK) as usize) | offset)
}

pub fn alloc_and_map(virt: usize, flags: u32) -> Option<usize>
{
    let phys = frame_allocator::alloc_frame()?;
    map_page(virt, phys, flags);
    Some(phys)
}

fn invalidate_page(virt: usize)
{
    // До первого mov cr3 инвалидация просто не имеет смысла, просто выходим.
    if !PAGING_ENABLED.load(Relaxed) { return; }

    unsafe
    {
        asm!(
            "invlpg [{0}]", // Трейлер парк...
            in(reg) virt,
            options(nostack, preserves_flags)
        );
    }
}

// DMA, тут нужен физически непрерывный диапазон но и отключённое кэширование.
pub fn dma_alloc(count: usize) -> Option<usize>
{
    let phys = dma::alloc_frames(count)?;

    for i in 0..count {
        let addr = phys + i * FRAME_PAGE_SIZE;
        map_page(addr, addr, KERNEL_DMA);
    }

    Some(phys)
}

pub fn dma_free(phys: usize, count: usize)
{
    for i in 0..count {
        let addr = phys + i * FRAME_PAGE_SIZE;
        unmap_page(addr);
        frame_allocator::free_frame(addr);
    }
}