// © Realix > Memory (Map + PMM)
// (04.07.26) v0.09
// ================
#![allow(dead_code)]

use core::ptr::addr_of_mut;

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

pub const FRAME_SIZE: usize = 4096;
const FRAME_SIZE_U64: u64 = FRAME_SIZE as u64;

// Полный 32-битный физический диапазон:
// 4 GiB / 4 KiB = 1_048_576 фреймов.
// Bitmap: 1_048_576 бит = 131_072 байта.
const MAX_FRAMES: usize = 1024 * 1024;
const BITMAP_WORDS: usize = MAX_FRAMES / 32;
const MAX_PHYS_ADDR_EXCLUSIVE: u64 = 0x1_0000_0000;

const RESERVED_LOW_MEMORY_END: usize = 0x0010_0000;
const VGA_TEXT_MEMORY_START: usize = 0x000B_8000;
const VGA_TEXT_MEMORY_END: usize = 0x000C_0000;

#[derive(Clone, Copy)]
pub struct Frame {
    pub addr: usize,
}

#[derive(Clone, Copy)]
pub struct MemoryStats {
    pub total_frames: usize,
    pub used_frames: usize,
    pub free_frames: usize,
    pub total_memory_kib: usize,
    pub free_memory_kib: usize,
}

struct Bitmap {
    words: *mut u32,
    bits: usize,
}

impl Bitmap {
    const unsafe fn new(words: *mut u32, bits: usize) -> Self {
        Self { words, bits }
    }

    unsafe fn fill(&mut self, value: bool) {
        let word_value: u32 = if value { u32::MAX } else { 0 };
        let words_count: usize = (self.bits + 31) / 32;
        let mut index: usize = 0;

        while index < words_count {
            self.words.add(index).write_volatile(word_value);
            index += 1;
        }
    }

    unsafe fn set(&mut self, bit: usize) {
        if bit >= self.bits { return; }

        let word_index: usize = bit / 32;
        let bit_index: usize = bit % 32;
        let ptr: *mut u32 = self.words.add(word_index);
        let value: u32 = ptr.read_volatile() | (1u32 << bit_index);
        ptr.write_volatile(value);
    }

    unsafe fn clear(&mut self, bit: usize) {
        if bit >= self.bits { return; }

        let word_index: usize = bit / 32;
        let bit_index: usize = bit % 32;
        let ptr: *mut u32 = self.words.add(word_index);
        let value: u32 = ptr.read_volatile() & !(1u32 << bit_index);
        ptr.write_volatile(value);
    }

    unsafe fn test(&self, bit: usize) -> bool {
        if bit >= self.bits { return true; }

        let word_index: usize = bit / 32;
        let bit_index: usize = bit % 32;
        let value: u32 = self.words.add(word_index).read_volatile();
        (value & (1u32 << bit_index)) != 0
    }

    unsafe fn find_zero(&self) -> Option<usize> {
        let words_count: usize = (self.bits + 31) / 32;
        let mut word_index: usize = 0;

        while word_index < words_count {
            let word: u32 = self.words.add(word_index).read_volatile();

            if word != u32::MAX {
                let mut bit_index: usize = 0;

                while bit_index < 32 {
                    let bit: usize = word_index * 32 + bit_index;
                    if bit >= self.bits { return None; }

                    if (word & (1u32 << bit_index)) == 0 {
                        return Some(bit);
                    }

                    bit_index += 1;
                }
            }

            word_index += 1;
        }

        None
    }
}

static mut BITMAP_STORAGE: [u32; BITMAP_WORDS] = [u32::MAX; BITMAP_WORDS];
static mut TOTAL_FRAMES: usize = 0;
static mut USED_FRAMES: usize = 0;
static mut INITIALIZED: bool = false;

#[inline]
fn align_down(value: usize, align: usize) -> usize {
    value & !(align - 1)
}

#[inline]
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[inline]
fn align_down_u64(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}

#[inline]
fn align_up_u64(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

#[inline]
unsafe fn bitmap() -> Bitmap {
    Bitmap::new(addr_of_mut!(BITMAP_STORAGE) as *mut u32, MAX_FRAMES)
}

#[inline]
unsafe fn read_u16(addr: usize) -> u16 {
    (addr as *const u16).read_unaligned()
}

#[inline]
unsafe fn read_u32(addr: usize) -> u32 {
    (addr as *const u32).read_unaligned()
}

#[inline]
unsafe fn read_u64(addr: usize) -> u64 {
    (addr as *const u64).read_unaligned()
}

/// Initialize PMM from PCINFO pointer passed by the bootloader.
/// PCINFO layout: low_mem:u16, disk:u8, e820_count:u16, e820_entries...
pub unsafe fn init_from_pcinfo(pcinfo_addr: usize) {
    let mut bm = bitmap();
    bm.fill(true);

    TOTAL_FRAMES = 0;
    USED_FRAMES = 0;

    let entries_count: usize = read_u16(pcinfo_addr + 3) as usize;
    let mut entry_addr: usize = pcinfo_addr + 5;
    let mut index: usize = 0;

    while index < entries_count && index < 64 {
        let base: u64 = read_u64(entry_addr);
        let size: u64 = read_u64(entry_addr + 8);
        let seg_type: u32 = read_u32(entry_addr + 16);

        if seg_type == 1 && size != 0 && base < MAX_PHYS_ADDR_EXCLUSIVE {
            let end: u64 = base.saturating_add(size).min(MAX_PHYS_ADDR_EXCLUSIVE);
            free_region_u64(base, end - base);
        }

        entry_addr += core::mem::size_of::<E820Entry>();
        index += 1;
    }

    reserve_region(0, RESERVED_LOW_MEMORY_END);
    reserve_region(VGA_TEXT_MEMORY_START, VGA_TEXT_MEMORY_END - VGA_TEXT_MEMORY_START);
    INITIALIZED = true;
}

pub fn is_initialized() -> bool {
    unsafe { INITIALIZED }
}

pub fn alloc_frame() -> Option<Frame> {
    unsafe {
        if !INITIALIZED { return None; }

        let mut bm = bitmap();
        let frame_index: usize = bm.find_zero()?;

        bm.set(frame_index);
        USED_FRAMES += 1;

        Some(Frame { addr: frame_index * FRAME_SIZE })
    }
}

pub fn free_frame(frame: Frame) {
    unsafe {
        if !INITIALIZED { return; }
        if frame.addr % FRAME_SIZE != 0 { return; }
        if frame.addr < RESERVED_LOW_MEMORY_END { return; }

        let frame_index: usize = frame.addr / FRAME_SIZE;
        if frame_index >= MAX_FRAMES { return; }

        let mut bm = bitmap();
        if bm.test(frame_index) {
            bm.clear(frame_index);
            USED_FRAMES = USED_FRAMES.saturating_sub(1);
        }
    }
}

pub fn reserve_region(base: usize, length: usize) {
    if length == 0 { return; }

    unsafe {
        let mut bm = bitmap();
        let start: usize = align_down(base, FRAME_SIZE) / FRAME_SIZE;
        let end: usize = align_up(base.saturating_add(length), FRAME_SIZE) / FRAME_SIZE;
        let mut frame: usize = start;

        while frame < end && frame < MAX_FRAMES {
            if !bm.test(frame) {
                bm.set(frame);
                USED_FRAMES += 1;
            }

            frame += 1;
        }
    }
}

pub fn free_region(base: usize, length: usize) {
    free_region_u64(base as u64, length as u64);
}

fn free_region_u64(base: u64, length: u64) {
    if length == 0 { return; }

    unsafe {
        let mut bm = bitmap();
        let end_addr: u64 = base.saturating_add(length).min(MAX_PHYS_ADDR_EXCLUSIVE);
        let start: usize = (align_up_u64(base, FRAME_SIZE_U64) / FRAME_SIZE_U64) as usize;
        let end: usize = (align_down_u64(end_addr, FRAME_SIZE_U64) / FRAME_SIZE_U64) as usize;
        let mut frame: usize = start;

        while frame < end && frame < MAX_FRAMES {
            if bm.test(frame) {
                bm.clear(frame);
                TOTAL_FRAMES += 1;
            }

            frame += 1;
        }
    }
}

pub fn stats() -> MemoryStats {
    unsafe {
        let used_frames: usize = if USED_FRAMES > TOTAL_FRAMES {
            TOTAL_FRAMES
        } else {
            USED_FRAMES
        };
        let free_frames: usize = TOTAL_FRAMES.saturating_sub(used_frames);

        MemoryStats {
            total_frames: TOTAL_FRAMES,
            used_frames,
            free_frames,
            total_memory_kib: TOTAL_FRAMES * (FRAME_SIZE / 1024),
            free_memory_kib: free_frames * (FRAME_SIZE / 1024),
        }
    }
}
