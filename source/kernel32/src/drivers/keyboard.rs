// © Realix > Driver: Keyboard
// (27.07.26) v0.1
// ================

// Подключение функций
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering::{Acquire, Relaxed, Release}};

// Константы
pub const KEYBOARD_DATA_PORT: u16 = 0x60;
const QUEUE_SIZE: usize = 32;
const PREFIX_EXTENDED_KEY: u8 = 0xE0;
pub const SCANCODE_RELEASE: u8 = 0x80;

// Кольцевой буфер
static mut QUEUE: [u8; QUEUE_SIZE] = [0; QUEUE_SIZE];
static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);

/// Клавиша
#[derive(Clone, Copy)]
pub enum Key { Char(u8), Up, Down, Escape, F7 }

/// Перевод scancode в ASCII
pub fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    match scancode {
        // Буквы
        0x10 => Some(b'q'), 0x11 => Some(b'w'), 0x12 => Some(b'e'),
        0x13 => Some(b'r'), 0x14 => Some(b't'), 0x15 => Some(b'y'),
        0x16 => Some(b'u'), 0x17 => Some(b'i'), 0x18 => Some(b'o'),
        0x19 => Some(b'p'), 0x1E => Some(b'a'), 0x1F => Some(b's'),
        0x20 => Some(b'd'), 0x21 => Some(b'f'), 0x22 => Some(b'g'),
        0x23 => Some(b'h'), 0x24 => Some(b'j'), 0x25 => Some(b'k'),
        0x26 => Some(b'l'), 0x2C => Some(b'z'), 0x2D => Some(b'x'),
        0x2E => Some(b'c'), 0x2F => Some(b'v'), 0x30 => Some(b'b'),
        0x31 => Some(b'n'), 0x32 => Some(b'm'),

        // Цифры
        0x02 => Some(b'1'), 0x03 => Some(b'2'), 0x04 => Some(b'3'),
        0x05 => Some(b'4'), 0x06 => Some(b'5'), 0x07 => Some(b'6'),
        0x08 => Some(b'7'), 0x09 => Some(b'8'), 0x0A => Some(b'9'),
        0x0B => Some(b'0'),

        // Спецсимволы (Shift пока не обрабатываем — будут строчные)
        0x0C => Some(b'-'), 0x0D => Some(b'='), 0x27 => Some(b';'),
        0x0E => Some(b'\x08'),  // Backspace
        0x0F => Some(b'\t'),    // Tab
        0x1C => Some(b'\n'),    // Enter
        0x39 => Some(b' '),     // Пробел
        0x28 => Some(b'\''), 0x29 => Some(b'`'),
        0x2B => Some(b'\\'), 0x35 => Some(b'/'),
        0x33 => Some(b','),  0x34 => Some(b'.'),
        0x1A => Some(b'['),  0x1B => Some(b']'),
        0x01 => Some(0x1B),   // ESC
        _ => None,
    }
}

/// Перевод scancode в функциональную клавишу
fn scancode_to_key(scancode: u8) -> Option<Key> {
    match scancode {
        0x01 => Some(Key::Escape),
        0x41 => Some(Key::F7),

        // Остальное: Символ из существующей таблицы (`scancode_to_ascii`)
        _ => scancode_to_ascii(scancode).map(Key::Char),
    }
}

/// Перевод расширенного scancode (с префиксом 0xE0)
fn extended_to_key(scancode: u8) -> Option<Key> {
    match scancode {
        0x48 => Some(Key::Up),
        0x50 => Some(Key::Down),
        _ => None,
    }
}

/// Вызывается из irq_handler (`isr.rs`)
pub fn on_scancode(scancode: u8) {
    let head: usize = HEAD.load(Relaxed);
    let next: usize = (head + 1) % QUEUE_SIZE;

    // Проверка на переполнение
    if next != TAIL.load(Acquire) {
        unsafe { QUEUE[head] = scancode; }
        HEAD.store(next, Release);
    }
}

/// Вызывается из обычного кода
pub fn queue_pop() -> Option<u8> {
    let tail = TAIL.load(Relaxed);

    // Проверка на пустоту
    if tail == HEAD.load(Acquire) {
        return None;
    }
    let value = unsafe { QUEUE[tail] };
    TAIL.store((tail + 1) % QUEUE_SIZE, Release);
    Some(value)
}


/// Проверка пустоты очереди
fn is_queue_empty() -> bool {
    TAIL.load(Relaxed) == HEAD.load(Acquire)
}


/// Чтение клавиши
pub fn read_key() -> Key {
    // Флаг: Предыдущий байт был префиксом 0xE0
    let mut extended: bool = false;

    loop {
        if let Some(scancode) = queue_pop() {
            // Префикс 0xE0: Клавиша прибежит след. байтом
            if scancode == PREFIX_EXTENDED_KEY {
                extended = true;
                continue;
            }

            // Игнорируем отпускание клавиш (бит 7 = 1)
            if scancode & SCANCODE_RELEASE != 0 {
                extended = false;
                continue;
            }

            // Распознаём функциональную клавишу
            let key: Option<Key> = if extended {
                extended = false;
                extended_to_key(scancode)
            } else {
                scancode_to_key(scancode)
            };

            // Возвращаем только распознанные клавиши
            if let Some(key) = key {
                return key;
            }
        } else {
            // Очередь пуста
            unsafe {
                asm!("cli");

                // Перепроверка очереди, пока нет прерываний
                if is_queue_empty() {
                    asm!("sti; hlt")
                } else {
                    asm!("sti");
                }
            }
        }
    }
}
