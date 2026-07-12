// © Realix > Frame Allocator
// (12.07.26) v0.09
// ================

use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use crate::x86::memory::{E820Map, E820Entry};

// Константы
const PAGE_SIZE: usize = 4096;          // 4 КБ
const MAX_PHYSICAL_MEMORY: usize = 128 * 1024 * 1024; // 128 МБ
const BITMAP_SIZE: usize = MAX_PHYSICAL_MEMORY / PAGE_SIZE / 8; // Битмап на 128 МБ

// Статический битмап для отслеживания занятых фреймов
static mut FRAME_BITMAP: [u8; BITMAP_SIZE] = [0; BITMAP_SIZE];
static TOTAL_FRAMES: AtomicUsize = AtomicUsize::new(0);
static FREE_FRAMES: AtomicUsize = AtomicUsize::new(0);

/// Инициализация аллокатора фреймов на основе карты памяти E820
pub fn init(memory_map: &E820Map) {
    unsafe {
        // Очищаем битмап
        for byte in FRAME_BITMAP.iter_mut() {
            *byte = 0xFF; // Все фреймы помечаем как занятые
        }
    }

    let mut total_frames: usize = 0;
    let mut free_frames: usize = 0;

    // Проходим по всем записям карты памяти
    for i in 0..memory_map.entry_count as usize {
        let entry: &E820Entry = &memory_map.map[i];
        
        // Нас интересуют только свободные области (type 1)
        if entry.seg_type != 1 {
            continue;
        }

        let start_addr: usize = entry.address as usize;
        let end_addr: usize = (entry.address + entry.size) as usize;

        // Ограничиваем адресное пространство
        if start_addr >= MAX_PHYSICAL_MEMORY {
            continue;
        }
        let end: usize = if end_addr > MAX_PHYSICAL_MEMORY {
            MAX_PHYSICAL_MEMORY
        } else {
            end_addr
        };

        // Вычисляем диапазон фреймов
        let start_frame: usize = start_addr / PAGE_SIZE;
        let end_frame: usize = end / PAGE_SIZE;

        // Помечаем фреймы как свободные
        for frame in start_frame..end_frame {
            mark_frame_free(frame);
            free_frames += 1;
        }
        total_frames += end_frame - start_frame;
    }

    // Резервируем первые 1 МБ (для загрузчика, ядра и т.д.)
    for frame in 0..(1024 * 1024 / PAGE_SIZE) {
        mark_frame_used(frame);
        if free_frames > 0 {
            free_frames -= 1;
        }
    }

    // Резервируем область, где находится само ядро
    // (адреса 0x100000 - 0x100000 + размер ядра)
    let kernel_start: usize = 0x100000;
    let kernel_end: usize = 0x110000; // Примерно 64 КБ для ядра
    
    for frame in (kernel_start / PAGE_SIZE)..(kernel_end / PAGE_SIZE) {
        mark_frame_used(frame);
        if free_frames > 0 {
            free_frames -= 1;
        }
    }

    // Резервируем область для стека ядра (0x90000 - 0x9FFFF)
    let stack_start: usize = 0x90000;
    let stack_end: usize = 0xA0000;
    
    for frame in (stack_start / PAGE_SIZE)..(stack_end / PAGE_SIZE) {
        mark_frame_used(frame);
        if free_frames > 0 {
            free_frames -= 1;
        }
    }

    // Резервируем область VGA (0xA0000 - 0xBFFFF)
    let vga_start: usize = 0xA0000;
    let vga_end: usize = 0xC0000;
    
    for frame in (vga_start / PAGE_SIZE)..(vga_end / PAGE_SIZE) {
        mark_frame_used(frame);
        if free_frames > 0 {
            free_frames -= 1;
        }
    }

    // Резервируем область BIOS (0xF0000 - 0xFFFFF)
    let bios_start: usize = 0xF0000;
    let bios_end: usize = 0x100000;
    
    for frame in (bios_start / PAGE_SIZE)..(bios_end / PAGE_SIZE) {
        mark_frame_used(frame);
        if free_frames > 0 {
            free_frames -= 1;
        }
    }

    TOTAL_FRAMES.store(total_frames, Relaxed);
    FREE_FRAMES.store(free_frames, Relaxed);
}

/// Выделение одного фрейма (4 КБ)
pub fn alloc_frame() -> Option<usize> {
    unsafe {
        for (byte_idx, byte) in FRAME_BITMAP.iter().enumerate() {
            if *byte != 0 {
                // Ищем первый свободный бит
                for bit in 0..8 {
                    if (*byte >> bit) & 1 == 1 {
                        let frame: usize = byte_idx * 8 + bit;
                        mark_frame_used(frame);
                        FREE_FRAMES.fetch_sub(1, Relaxed);
                        return Some(frame * PAGE_SIZE);
                    }
                }
            }
        }
    }
    None // Нет свободной памяти
}

/// Освобождение фрейма
pub fn free_frame(addr: usize) {
    let frame: usize = addr / PAGE_SIZE;
    mark_frame_free(frame);
    FREE_FRAMES.fetch_add(1, Relaxed);
}

/// Выделение нескольких последовательных фреймов
pub fn alloc_frames(count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }

    unsafe {
        let mut consecutive: usize = 0;
        let mut start_frame: usize = 0;

        for (byte_idx, byte) in FRAME_BITMAP.iter().enumerate() {
            for bit in 0..8 {
                if (*byte >> bit) & 1 == 1 {
                    // Свободный фрейм
                    if consecutive == 0 {
                        start_frame = byte_idx * 8 + bit;
                    }
                    consecutive += 1;
                    
                    if consecutive == count {
                        // Нашли нужное количество последовательных фреймов
                        for i in 0..count {
                            mark_frame_used(start_frame + i);
                        }
                        FREE_FRAMES.fetch_sub(count, Relaxed);
                        return Some(start_frame * PAGE_SIZE);
                    }
                } else {
                    consecutive = 0;
                }
            }
        }
    }
    None
}

/// Получение статистики
pub fn get_stats() -> (usize, usize) {
    (TOTAL_FRAMES.load(Relaxed), FREE_FRAMES.load(Relaxed))
}

/// Проверка, свободен ли фрейм
pub fn is_frame_free(addr: usize) -> bool {
    let frame: usize = addr / PAGE_SIZE;
    let byte_idx: usize = frame / 8;
    let bit: usize = frame % 8;
    
    unsafe {
        (FRAME_BITMAP[byte_idx] >> bit) & 1 == 1
    }
}

/// Получение количества свободной памяти в байтах
pub fn get_free_memory() -> usize {
    FREE_FRAMES.load(Relaxed) * PAGE_SIZE
}

/// Получение количества всей памяти в байтах
pub fn get_total_memory() -> usize {
    TOTAL_FRAMES.load(Relaxed) * PAGE_SIZE
}

// Вспомогательные функции
fn mark_frame_used(frame: usize) {
    let byte_idx: usize = frame / 8;
    let bit: usize = frame % 8;
    unsafe {
        FRAME_BITMAP[byte_idx] &= !(1 << bit);
    }
}

fn mark_frame_free(frame: usize) {
    let byte_idx: usize = frame / 8;
    let bit: usize = frame % 8;
    unsafe {
        FRAME_BITMAP[byte_idx] |= 1 << bit;
    }
}

/// Дефрагментация битмапа (опционально)
pub fn defragment() {
    // В текущей реализации не требуется, так как мы используем простой
    // first-fit алгоритм. Для более сложных сценариев можно добавить
    // компактирование свободных фреймов.
}

/// Вывод карты занятых фреймов (для отладки)
pub fn dump_bitmap() {
    use crate::drivers::vga;
    
    vga::print_line("Frame Bitmap (first 256 bytes):\n", vga::Color::Cyan);
    
    unsafe {
        for (i, byte) in FRAME_BITMAP.iter().take(256).enumerate() {
            if i % 32 == 0 {
                let mut buf: [u8; 10] = [0; 10];
                vga::print_line("  Offset 0x", vga::Color::LightGray);
                vga::print_line(
                    crate::utils::u32_to_hex_str((i * 8 * PAGE_SIZE) as u32, &mut buf),
                    vga::Color::White,
                );
                vga::print_line(": ", vga::Color::LightGray);
            }
            
            // Выводим байт в двоичном виде
            for bit in (0..8).rev() {
                if (*byte >> bit) & 1 == 1 {
                    vga::print_char(b'.', vga::Color::Green);  // Свободно
                } else {
                    vga::print_char(b'#', vga::Color::Red);    // Занято
                }
            }
            
            if (i + 1) % 32 == 0 {
                vga::new_line();
            }
        }
    }
    vga::new_line();
}
