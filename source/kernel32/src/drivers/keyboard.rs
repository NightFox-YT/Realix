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

// Scancode'ы левого/правого Shift (без бита отпускания)
const SCANCODE_LSHIFT: u8 = 0x2A;
const SCANCODE_RSHIFT: u8 = 0x36;

// Кольцевой буфер
static mut QUEUE: [u8; QUEUE_SIZE] = [0; QUEUE_SIZE];
static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);

// Состояние Shift (Left или Right, обновляется в read_key по make/break кодам)
static SHIFT_HELD: AtomicBool = AtomicBool::new(false);

/// Клавиша
#[derive(Clone, Copy)]
pub enum Key { Char(u8), Up, Down, Escape, F7 }

/// Перевод scancode в ASCII (с учётом текущего состояния Shift)
pub fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    let base: u8 = match scancode {
        // Буквы
        0x10 => b'q', 0x11 => b'w', 0x12 => b'e',
        0x13 => b'r', 0x14 => b't', 0x15 => b'y',
        0x16 => b'u', 0x17 => b'i', 0x18 => b'o',
        0x19 => b'p', 0x1E => b'a', 0x1F => b's',
        0x20 => b'd', 0x21 => b'f', 0x22 => b'g',
        0x23 => b'h', 0x24 => b'j', 0x25 => b'k',
        0x26 => b'l', 0x2C => b'z', 0x2D => b'x',
        0x2E => b'c', 0x2F => b'v', 0x30 => b'b',
        0x31 => b'n', 0x32 => b'm',

        // Цифры
        0x02 => b'1', 0x03 => b'2', 0x04 => b'3',
        0x05 => b'4', 0x06 => b'5', 0x07 => b'6',
        0x08 => b'7', 0x09 => b'8', 0x0A => b'9',
        0x0B => b'0',

        // Спецсимволы
        0x0C => b'-', 0x0D => b'=', 0x27 => b';',
        0x0E => b'\x08',  // Backspace
        0x0F => b'\t',    // Tab
        0x1C => b'\n',    // Enter
        0x39 => b' ',     // Пробел
        0x28 => b'\'', 0x29 => b'`',
        0x2B => b'\\', 0x35 => b'/',
        0x33 => b',',  0x34 => b'.',
        0x1A => b'[',  0x1B => b']',
        0x01 => 0x1B,   // ESC
        _ => return None,
    };

    if !SHIFT_HELD.load(Relaxed) {
        return Some(base);
    }

    Some(shift_char(base))
}

/// Верхний регистр букв и "верхние" символы цифр/пунктуации (US-раскладка)
fn shift_char(base: u8) -> u8 {
    match base {
        b'a'..=b'z' => base - (b'a' - b'A'),
        b'1' => b'!', b'2' => b'@', b'3' => b'#', b'4' => b'$', b'5' => b'%',
        b'6' => b'^', b'7' => b'&', b'8' => b'*', b'9' => b'(', b'0' => b')',
        b'-' => b'_', b'=' => b'+', b';' => b':', b'\'' => b'"', b'`' => b'~',
        b',' => b'<', b'.' => b'>', b'/' => b'?', b'[' => b'{', b']' => b'}',
        b'\\' => b'|',
        other => other,  // управляющие символы, пробел и т.п. - без изменений
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

            // Отслеживание состояния Shift (make/break, бит 7 - отпускание)
            let bare_code: u8 = scancode & !SCANCODE_RELEASE;
            if bare_code == SCANCODE_LSHIFT || bare_code == SCANCODE_RSHIFT {
                SHIFT_HELD.store(scancode & SCANCODE_RELEASE == 0, Relaxed);
                extended = false;
                continue;
            }

            // Игнорируем отпускание прочих клавиш (бит 7 = 1)
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
