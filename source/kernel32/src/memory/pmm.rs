// © Realix > x86: Memory (PMM)
// (09.08.26) v0.11
// ================
// ❗️ Раскладка должна совпадать с bios-api/memory/high.asm
#![allow(dead_code)]

use core::ffi::c_void;

/// Структура записи карты памяти E820 (24 байта)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct E820Entry {
    pub address: u64,     // Начало региона
    pub size: u64,        // Длина региона (байт)
    pub seg_type: u32,    // Тип региона (1 - свободный)
    pub attributes: u32,  // Расширенные атрибуты ACPI 3.X
}

// Состояние аллокатора
static mut PAGE_POOL_START: usize = 0;
static mut PAGE_POOL_END: usize = 0;
const PAGE_SIZE: usize = 4096;

/// Инициализация PMM: Нахождение первого пригодного региона
#[no_mangle]
pub extern "C" fn init_kernel_page_allocator() {
    let pcinfo = unsafe { crate::pcinfo() };
    let mmap_count: usize = (pcinfo.mmap_count as usize).min(crate::E820_MAX_ENTRIES);
 
    let mut found: bool = false;
    for i in 0..mmap_count {
        let entry: E820Entry = pcinfo.mmap_entry(i);

        // Находим первый подходящий регион под аллокатор
        if entry.seg_type == 1 && entry.address >= 0x100000 {
            unsafe {
                PAGE_POOL_START = ((entry.address + 4095) & !4095) as usize;
                PAGE_POOL_END = ((entry.address + entry.size) & !4095) as usize;
                found = true;
                break;
            }
        }
    }
 
    if !found {
        unsafe {
            PAGE_POOL_START = 0x500000;
            PAGE_POOL_END = 0x2000000;
        }
    }
}
 
/// Выделяет одну физическую страницу (4K).
/// Возвращает null, если пул исчерпан.
#[no_mangle]
pub unsafe extern "C" fn rlxalloc_page() -> *mut c_void {
    // Память переполнена
    if PAGE_POOL_START >= PAGE_POOL_END {
        return core::ptr::null_mut();
    }

    let page: usize = PAGE_POOL_START;
    PAGE_POOL_START += PAGE_SIZE;
    return page as *mut c_void;
}