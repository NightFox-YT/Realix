// © Realix > x86: Frame Allocator
// (27.07.26) v0.1
// ================
// ❗️ Только однопоточный доступ из ядра (не из IRQ-обработчиков)

// Подключение функций
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use crate::{E820_MAX_ENTRIES, memory::pmm, memory};

/// Инициализация аллокатора фреймов на основе карты памяти E820
pub fn init(memory_map: &[pmm::E820Entry; E820_MAX_ENTRIES]) {
    // Очищаем битмап: Все фреймы помечаем как занятые
    memory::bitmap().fill(0xFF);

    // Считываем кол-во записей от загрузчика, но оно может превышать ёмкость массива
    let entry_count: usize = memory_map.len() as usize;

    // Освобождаем фреймы по свободным регионам E820
    for i in 0..entry_count {
        let entry = memory_map[i];

        // Пропускаем не свободные области памяти E820
        if entry.seg_type != memory::E820_TYPE_FREE {
            continue;
        }

        let max_phys = memory::FRAME_MAX_PHYS_MEMORY as u64;

        // Пропускаем адреса, выходящие за наше адресное пространство
        if entry.address >= max_phys {
            continue;
        }

        // Сохраняем адреса из записи карты памяти
        let start: u64 = entry.address;
        let end: u64 = entry.address.saturating_add(entry.size)
            .min(memory::FRAME_MAX_PHYS_MEMORY as u64);

        if start >= end { continue; }

        // Вычисляем диапазон фреймов (Частично свободный фрейм - занятый фрейм)
        let start_frame: usize = start.div_ceil(memory::FRAME_PAGE_SIZE as u64) as usize;
        let end_frame: usize = (end / memory::FRAME_PAGE_SIZE as u64) as usize;

        if start_frame < end_frame {
            // Помечаем фрейм как свободный
            for frame in start_frame..end_frame {
                memory::mark_frame_free(frame);
            }
        }
    }

    // резервируем нижнюю память
    let reserved_frames = memory::RESERVED_LOW_MEMORY / memory::FRAME_PAGE_SIZE;
    for frame in 0..reserved_frames {
        memory::mark_frame_used(frame);
    }

    let free_frames_count: usize = memory::count_free_frames();
    memory::TOTAL_FRAMES.store(free_frames_count, Relaxed);
    memory::FREE_FRAMES.store(free_frames_count, Relaxed);
}

/// Выделение одного фрейма
/// Вывод: физический адрес фрейма (None - свободных нет)
pub fn alloc_frame() -> Option<usize> {
    for (byte_idx, byte) in memory::bitmap().iter_mut().enumerate() {
        // Все 8 фреймов байта заняты
        if *byte == 0xFF {
            continue;
        }

        // Первый свободный (нулевой) бит и его пометка занятым
        let bit: usize = byte.trailing_ones() as usize;
        *byte |= 1 << bit;

        memory::FREE_FRAMES.fetch_sub(1, Relaxed);
        return Some((byte_idx * 8 + bit) * memory::FRAME_PAGE_SIZE);
    }

    // Нет свободной памяти
    None
}

/// Выделение нескольких последовательных фреймов
/// Вывод: физический адрес первого фрейма (None - непрерывного окна нет)
pub fn alloc_frames(count: usize) -> Option<usize> {
    // Проверка: 0 фреймов и выделять не надо
    if count == 0 {
        return None;
    }

    let mut consecutive: usize = 0;
    let mut start_frame: usize = 0;

    // Перебираем фреймы
    for frame in 0..memory::FRAME_COUNT {
        // Занятый фрейм разрывает непрерывное окно
        if !memory::is_bit_free(frame) {
            consecutive = 0;
            continue;
        }

        // Фрейм - начало непрерывного окна
        if consecutive == 0 {
            start_frame = frame;
        }
        consecutive += 1;

        // Нашли нужное кол-во последовательных фреймов
        if consecutive == count {
            for frame in start_frame..start_frame + count {
                memory::mark_frame_used(frame);
            }
            memory::FREE_FRAMES.fetch_sub(count, Relaxed);
            return Some(start_frame * memory::FRAME_PAGE_SIZE);
        }
    }

    None
}

/// Освобождение фрейма по физическому адресу
pub fn free_frame(addr: usize) {
    // Проверка: адрес выходит за предел адресного пространства
    if addr >= memory::FRAME_MAX_PHYS_MEMORY {
        return;
    }

    let frame: usize = addr / memory::FRAME_PAGE_SIZE;

    // Уже свободен: не искажаем счётчик повторным освобождением
    if memory::is_bit_free(frame) {
        return;
    }

    memory::mark_frame_free(frame);
    memory::FREE_FRAMES.fetch_add(1, Relaxed);
}

/// Вывод карты фреймов первых 4 МБ (для отладки)
/// ('.' - свободен, '#' - занят)
pub fn dump_bitmap() {
    use crate::drivers::vga;

    // 8 байт на строку: префикс + 64 символа влезают
    const BYTES_PER_ROW: usize = 8;

    // Сколько байт битмапа показать (Первые 4 МБ)
    const DUMP_BYTES: usize = 4 * 1024 * 1024 / memory::FRAME_PAGE_SIZE / 8;

    vga::print_line("Frame map, first 4 MB ('.' free, '#' used):\n", vga::Color::Cyan);

    for (i, byte) in memory::bitmap().iter().take(DUMP_BYTES).enumerate() {
        // Пишем адрес в начале каждой строки
        if i % BYTES_PER_ROW == 0 {
            let mut buf: [u8; 10] = [0; 10];
            vga::print_line("  ", vga::Color::LightGray);
            vga::print_line(
                crate::utils::u32_to_hex_str(
                    (i * 8 * memory::FRAME_PAGE_SIZE) as u32, &mut buf),
                vga::Color::White,
            );
            vga::print_line(": ", vga::Color::LightGray);
        }

        // Бит 0 - младший фрейм байта, выводим по возрастанию адресов
        for bit in 0..8 {
            if (*byte >> bit) & 1 == 0 {
                vga::print_char(b'.', vga::Color::Green);
            } else {
                vga::print_char(b'#', vga::Color::Red);
            }
        }

        // Переносим крусор на след. строку после BYTES_PER_ROW байт
        if (i + 1) % BYTES_PER_ROW == 0 {
            vga::print_new_line();
        }
    }
}

