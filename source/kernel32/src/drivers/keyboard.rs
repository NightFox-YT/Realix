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

// Make-код Ctrl - левый идёт обычным (не-расширенным) байтом 0x1D, правый -
// расширенным (0xE0 0x1D), но сам байт совпадает - для простого "нажат ли
// Ctrl" разницу можно не делать (см. read_key: проверяется независимо от
// текущего значения `extended`)
const CTRL_MAKE: u8 = 0x1D;

// Скан-коды W/A/S/D - используются как управление окнами Cliff (Ctrl+WASD
// двигает активное окно, Ctrl+Shift+WASD - меняет его размер), см.
// Key::WindowMove/WindowResize ниже. Обычная печать 'w'/'a'/'s'/'d' (без
// Ctrl) не затронута - перехват происходит только при зажатом Ctrl
const SCANCODE_W: u8 = 0x11;
const SCANCODE_A: u8 = 0x1E;
const SCANCODE_S: u8 = 0x1F;
const SCANCODE_D: u8 = 0x20;

// Кольцевой буфер
static mut QUEUE: [u8; QUEUE_SIZE] = [0; QUEUE_SIZE];
static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);

// Состояние Shift/Ctrl (переживает между отдельными вызовами read_key -
// клавиша может быть зажата, пока читаются несколько символов подряд)
static SHIFT: AtomicBool = AtomicBool::new(false);
static CTRL: AtomicBool = AtomicBool::new(false);

/// Направление - для Key::WindowMove/WindowResize (см. ниже)
#[derive(Clone, Copy, PartialEq)]
pub enum Direction { Up, Down, Left, Right }

/// Клавиша
#[derive(Clone, Copy)]
pub enum Key {
    Char(u8), Up, Down, Left, Right, Escape, F7,
    // Ctrl+WASD / Ctrl+Shift+WASD - управление окнами (см. SCANCODE_W и
    // остальные выше); не пересекается с обычной печатью 'w'/'a'/'s'/'d'
    WindowMove(Direction),
    WindowResize(Direction),
}

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
    read_key_impl(None).expect("read_key_impl(None) всегда возвращает Some")
}

/// Как read_key(), но возвращает None, если клавиша не появилась за
/// `timeout_ticks` тиков PIT - используется Cliff для периодической
/// перерисовки (напр. часы на панели) без участия пользователя. hlt внутри
/// и так просыпается на каждый тик PIT (нужен для аптайма) - здесь просто
/// добавлена проверка дедлайна между пробуждениями, ничего больше не меняя
pub fn read_key_timeout(timeout_ticks: u32) -> Option<Key> {
    let deadline = crate::drivers::pit::get_ticks().wrapping_add(timeout_ticks);
    read_key_impl(Some(deadline))
}

fn read_key_impl(deadline: Option<u32>) -> Option<Key> {
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

            // Ctrl - как и Shift, сам не производит символа. Левый/правый
            // делят один и тот же байт (0x1D) - различать их не нужно,
            // достаточно знать "зажат ли Ctrl вообще" (см. константу выше)
            if bare_scancode == CTRL_MAKE {
                CTRL.store(scancode & SCANCODE_RELEASE == 0, Relaxed);
                extended = false;
                continue;
            }

            // Игнорируем отпускание остальных клавиш (бит 7 = 1)
            if scancode & SCANCODE_RELEASE != 0 {
                extended = false;
                continue;
            }

            // Ctrl+WASD - управление окнами Cliff, перехватывается раньше
            // обычной раскладки (иначе W/A/S/D печатались бы как буквы)
            if !extended && CTRL.load(Relaxed) {
                let direction = match scancode {
                    SCANCODE_W => Some(Direction::Up),
                    SCANCODE_A => Some(Direction::Left),
                    SCANCODE_S => Some(Direction::Down),
                    SCANCODE_D => Some(Direction::Right),
                    _ => None,
                };
                if let Some(direction) = direction {
                    return Some(if SHIFT.load(Relaxed) {
                        Key::WindowResize(direction)
                    } else {
                        Key::WindowMove(direction)
                    });
                }
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
                return Some(key);
            }
        } else {
            // Дедлайн (см. read_key_timeout) - выходим по таймауту, ничего
            // не дожидаясь. wrapping_sub + `as i32 <= 0` корректно работает
            // и в редком случае переполнения счётчика тиков (~497 дней)
            if let Some(deadline) = deadline {
                if (crate::drivers::pit::get_ticks().wrapping_sub(deadline) as i32) >= 0 {
                    return None;
                }
            }

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
