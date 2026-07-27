// © Realix > x86: Frame Allocator
// (27.07.26) v0.1
// ================
// ❗️ Только однопоточный доступ из ядра (не из IRQ-обработчиков)

// Подключение функций
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use crate::x86::memory::E820Map;

// Размер фрейма и границы адресного пространства
const PAGE_SIZE: usize = 4096;                         // 4 КБ
const MAX_PHYSICAL_MEMORY: usize = 128 * 1024 * 1024;  // ! 128 МБ для теста
const FRAME_COUNT: usize = MAX_PHYSICAL_MEMORY / PAGE_SIZE;

// Битмап: 1 байт на 8 фреймов
const BITMAP_SIZE: usize = FRAME_COUNT / 8;

// Первый 1 МБ резервируется целиком:
// IVT+BDA, загрузчик, kernel32, стек ядра, видеопамять и BIOS ROM
const RESERVED_LOW_MEMORY: usize = 1024 * 1024;

/// Тип свободного региона в карте памяти E820
const E820_TYPE_FREE: u32 = 1;

// Статический битмап фреймов: бит 0 - свободен, бит 1 - занят
// (Изначально всё занято: свобода выдаётся по карте памяти E820)
static mut FRAME_BITMAP: [u8; BITMAP_SIZE] = [0xFF; BITMAP_SIZE];
static TOTAL_FRAMES: AtomicUsize = AtomicUsize::new(0);
static FREE_FRAMES: AtomicUsize = AtomicUsize::new(0);

/// Доступ к битмапу через raw-указатель
fn bitmap() -> &'static mut [u8; BITMAP_SIZE] {
    unsafe { &mut *(&raw mut FRAME_BITMAP) }
}

/// Инициализация аллокатора фреймов на основе карты памяти E820
pub fn init(memory_map: &E820Map) {
    // Очищаем битмап: Все фреймы помечаем как занятые
    bitmap().fill(0xFF);

    // Считываем кол-во записей от загрузчика, но оно может превышать ёмкость массива
    let entry_count: usize = (memory_map.entry_count as usize).min(memory_map.map.len());

    // Освобождаем фреймы по свободным регионам E820
    for i in 0..entry_count {
        let entry = memory_map.map[i];

        // Пропускаем не свободные области памяти E820
        if entry.seg_type != E820_TYPE_FREE {
            continue;
        }

        // Пропускаем адреса, выходящие за наше адресное пространство
        if entry.address >= MAX_PHYSICAL_MEMORY as u64 {
            continue;
        }

        // Сохраняем адреса из записи карты памяти
        let start: u64 = entry.address;
        let end: u64 = entry.address.saturating_add(entry.size)
            .min(MAX_PHYSICAL_MEMORY as u64);

        // Вычисляем диапазон фреймов (Частично свободный фрейм - занятый фрейм)
        let start_frame: usize = start.div_ceil(PAGE_SIZE as u64) as usize;
        let end_frame: usize = (end / PAGE_SIZE as u64) as usize;

        // Помечаем фрейм как свободный
        for frame in start_frame..end_frame {
            mark_frame_free(frame);
        }
    }

    // Всего полезных фреймов, освобождённых по карте E820
    let total_frames: usize = count_free_frames();

    // Резервируем нижнюю память (см. RESERVED_LOW_MEMORY)
    for frame in 0..(RESERVED_LOW_MEMORY / PAGE_SIZE) {
        mark_frame_used(frame);
    }

    TOTAL_FRAMES.store(total_frames, Relaxed);
    FREE_FRAMES.store(count_free_frames(), Relaxed);
}

/// Выделение одного фрейма
/// Вывод: физический адрес фрейма (None - свободных нет)
pub fn alloc_frame() -> Option<usize> {
    for (byte_idx, byte) in bitmap().iter_mut().enumerate() {
        // Все 8 фреймов байта заняты
        if *byte == 0xFF {
            continue;
        }

        // Первый свободный (нулевой) бит и его пометка занятым
        let bit: usize = byte.trailing_ones() as usize;
        *byte |= 1 << bit;

        FREE_FRAMES.fetch_sub(1, Relaxed);
        return Some((byte_idx * 8 + bit) * PAGE_SIZE);
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
    for frame in 0..FRAME_COUNT {
        // Занятый фрейм разрывает непрерывное окно
        if !is_bit_free(frame) {
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
                mark_frame_used(frame);
            }
            FREE_FRAMES.fetch_sub(count, Relaxed);
            return Some(start_frame * PAGE_SIZE);
        }
    }

    None
}

/// Освобождение фрейма по физическому адресу
pub fn free_frame(addr: usize) {
    // Проверка: адрес выходит за предел адресного пространства
    if addr >= MAX_PHYSICAL_MEMORY {
        return;
    }

    let frame: usize = addr / PAGE_SIZE;

    // Уже свободен: не искажаем счётчик повторным освобождением
    if is_bit_free(frame) {
        return;
    }

    mark_frame_free(frame);
    FREE_FRAMES.fetch_add(1, Relaxed);
}

/// Проверка, свободен ли фрейм по физическому адресу
pub fn is_frame_free(addr: usize) -> bool {
    // Проверка: адрес выходит за предел адресного пространства
    if addr >= MAX_PHYSICAL_MEMORY {
        return false;
    }
    is_bit_free(addr / PAGE_SIZE)
}

/// Получение статистики: (всего фреймов, свободно фреймов)
pub fn get_stats() -> (usize, usize) {
    (TOTAL_FRAMES.load(Relaxed), FREE_FRAMES.load(Relaxed))
}

/// Получение количества свободной памяти в байтах
pub fn get_free_memory() -> usize {
    FREE_FRAMES.load(Relaxed) * PAGE_SIZE
}

/// Получение количества всей полезной памяти в байтах
pub fn get_total_memory() -> usize {
    TOTAL_FRAMES.load(Relaxed) * PAGE_SIZE
}

// > Вспомогательные функции (номер фрейма, без валидации)
fn is_bit_free(frame: usize) -> bool {
    (bitmap()[frame / 8] >> (frame % 8)) & 1 == 0
}

fn mark_frame_used(frame: usize) {
    bitmap()[frame / 8] |= 1 << (frame % 8);
}

fn mark_frame_free(frame: usize) {
    bitmap()[frame / 8] &= !(1 << (frame % 8));
}

/// Подсчёт свободных фреймов по битмапу
fn count_free_frames() -> usize {
    bitmap().iter().map(|byte| byte.count_zeros() as usize).sum()
}

/// Вывод карты фреймов первых 4 МБ (для отладки)
/// ('.' - свободен, '#' - занят)
pub fn dump_bitmap() {
    use crate::drivers::vga;

    // 8 байт на строку: префикс + 64 символа влезают
    const BYTES_PER_ROW: usize = 8;

    // Сколько байт битмапа показать (Первые 4 МБ)
    const DUMP_BYTES: usize = 4 * 1024 * 1024 / PAGE_SIZE / 8;

    vga::print_line("Frame map, first 4 MB ('.' free, '#' used):\n", vga::Color::Cyan);

    for (i, byte) in bitmap().iter().take(DUMP_BYTES).enumerate() {
        // Пишем адрес в начале каждой строки
        if i % BYTES_PER_ROW == 0 {
            let mut buf: [u8; 10] = [0; 10];
            vga::print_line("  ", vga::Color::LightGray);
            vga::print_line(
                crate::utils::u32_to_hex_str(
                    (i * 8 * PAGE_SIZE) as u32, &mut buf),
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
            vga::new_line();
        }
    }
}
