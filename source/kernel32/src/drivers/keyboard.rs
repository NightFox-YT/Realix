// © Realix > Driver: Keyboard
// (27.07.26) v0.1
// ================

// Подключение функций
use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering::{Acquire, Relaxed, Release}};

// Константы
pub const KEYBOARD_DATA_PORT: u16 = 0x60;
const QUEUE_SIZE: usize = 32;
const PREFIX_EXTENDED_KEY: u8 = 0xE0;
pub const SCANCODE_RELEASE: u8 = 0x80;

// Make-коды левого/правого Shift - сами по себе не производят символа, но
// переключают регистр букв и "верхнюю" раскладку остальных клавиш (см.
// scancode_to_ascii). Отслеживаются отдельно в read_key (см. ниже)
const SHIFT_LEFT_MAKE: u8 = 0x2A;
const SHIFT_RIGHT_MAKE: u8 = 0x36;

// Кольцевой буфер
static mut QUEUE: [u8; QUEUE_SIZE] = [0; QUEUE_SIZE];
static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);

// Состояние Shift (переживает между отдельными вызовами read_key - клавиша
// может быть зажата, пока читаются несколько символов подряд)
static SHIFT: AtomicBool = AtomicBool::new(false);

/// Клавиша
#[derive(Clone, Copy)]
pub enum Key { Char(u8), Up, Down, Left, Right, Escape, F7 }

/// Перевод scancode в ASCII (с учётом текущего состояния Shift)
pub fn scancode_to_ascii(scancode: u8, shift: bool) -> Option<u8> {
    match scancode {
        // Буквы (Shift - верхний регистр)
        0x10 => Some(if shift { b'Q' } else { b'q' }),
        0x11 => Some(if shift { b'W' } else { b'w' }),
        0x12 => Some(if shift { b'E' } else { b'e' }),
        0x13 => Some(if shift { b'R' } else { b'r' }),
        0x14 => Some(if shift { b'T' } else { b't' }),
        0x15 => Some(if shift { b'Y' } else { b'y' }),
        0x16 => Some(if shift { b'U' } else { b'u' }),
        0x17 => Some(if shift { b'I' } else { b'i' }),
        0x18 => Some(if shift { b'O' } else { b'o' }),
        0x19 => Some(if shift { b'P' } else { b'p' }),
        0x1E => Some(if shift { b'A' } else { b'a' }),
        0x1F => Some(if shift { b'S' } else { b's' }),
        0x20 => Some(if shift { b'D' } else { b'd' }),
        0x21 => Some(if shift { b'F' } else { b'f' }),
        0x22 => Some(if shift { b'G' } else { b'g' }),
        0x23 => Some(if shift { b'H' } else { b'h' }),
        0x24 => Some(if shift { b'J' } else { b'j' }),
        0x25 => Some(if shift { b'K' } else { b'k' }),
        0x26 => Some(if shift { b'L' } else { b'l' }),
        0x2C => Some(if shift { b'Z' } else { b'z' }),
        0x2D => Some(if shift { b'X' } else { b'x' }),
        0x2E => Some(if shift { b'C' } else { b'c' }),
        0x2F => Some(if shift { b'V' } else { b'v' }),
        0x30 => Some(if shift { b'B' } else { b'b' }),
        0x31 => Some(if shift { b'N' } else { b'n' }),
        0x32 => Some(if shift { b'M' } else { b'm' }),

        // Цифры (Shift - верхний ряд символов US-раскладки)
        0x02 => Some(if shift { b'!' } else { b'1' }),
        0x03 => Some(if shift { b'@' } else { b'2' }),
        0x04 => Some(if shift { b'#' } else { b'3' }),
        0x05 => Some(if shift { b'$' } else { b'4' }),
        0x06 => Some(if shift { b'%' } else { b'5' }),
        0x07 => Some(if shift { b'^' } else { b'6' }),
        0x08 => Some(if shift { b'&' } else { b'7' }),
        0x09 => Some(if shift { b'*' } else { b'8' }),
        0x0A => Some(if shift { b'(' } else { b'9' }),
        0x0B => Some(if shift { b')' } else { b'0' }),

        // Спецсимволы (Shift - соответствующий верхний символ US-раскладки)
        0x0C => Some(if shift { b'_' } else { b'-' }),
        0x0D => Some(if shift { b'+' } else { b'=' }),
        0x27 => Some(if shift { b':' } else { b';' }),
        0x0E => Some(b'\x08'),  // Backspace
        0x0F => Some(b'\t'),    // Tab
        0x1C => Some(b'\n'),    // Enter
        0x39 => Some(b' '),     // Пробел
        0x28 => Some(if shift { b'"' } else { b'\'' }),
        0x29 => Some(if shift { b'~' } else { b'`' }),
        0x2B => Some(if shift { b'|' } else { b'\\' }),
        0x35 => Some(if shift { b'?' } else { b'/' }),
        0x33 => Some(if shift { b'<' } else { b',' }),
        0x34 => Some(if shift { b'>' } else { b'.' }),
        0x1A => Some(if shift { b'{' } else { b'[' }),
        0x1B => Some(if shift { b'}' } else { b']' }),
        0x01 => Some(0x1B),   // ESC
        _ => None,
    }
}

/// Перевод scancode в функциональную клавишу
fn scancode_to_key(scancode: u8, shift: bool) -> Option<Key> {
    match scancode {
        0x01 => Some(Key::Escape),
        0x41 => Some(Key::F7),

        // Остальное: Символ из существующей таблицы (`scancode_to_ascii`)
        _ => scancode_to_ascii(scancode, shift).map(Key::Char),
    }
}

/// Перевод расширенного scancode (с префиксом 0xE0)
fn extended_to_key(scancode: u8) -> Option<Key> {
    match scancode {
        0x48 => Some(Key::Up),
        0x50 => Some(Key::Down),
        0x4B => Some(Key::Left),
        0x4D => Some(Key::Right),
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

            // Shift (левый/правый) сам не производит символа - только
            // переключает регистр для остальных клавиш. Проверяем ДО общего
            // игнорирования отпусканий ниже, иначе отпускание Shift'а само
            // потерялось бы неотличимым от отпускания любой другой клавиши
            let bare_scancode = scancode & !SCANCODE_RELEASE;
            if bare_scancode == SHIFT_LEFT_MAKE || bare_scancode == SHIFT_RIGHT_MAKE {
                SHIFT.store(scancode & SCANCODE_RELEASE == 0, Relaxed);
                extended = false;
                continue;
            }

            // Игнорируем отпускание остальных клавиш (бит 7 = 1)
            if scancode & SCANCODE_RELEASE != 0 {
                extended = false;
                continue;
            }

            // Распознаём функциональную клавишу
            let shift: bool = SHIFT.load(Relaxed);
            let key: Option<Key> = if extended {
                extended = false;
                extended_to_key(scancode)
            } else {
                scancode_to_key(scancode, shift)
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
