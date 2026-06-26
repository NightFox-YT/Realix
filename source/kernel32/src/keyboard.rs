use crate::vga::{self, Color};

static mut SHIFT_PRESSED: bool = false;

// Циклический буфер для нажатий клавиш (заполняется из прерывания)
static mut KEY_BUFFER: [u8; 32] = [0; 32];
static mut KEY_BUF_HEAD: usize = 0;
static mut KEY_BUF_TAIL: usize = 0;

fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    let shift = unsafe { SHIFT_PRESSED };
    match scancode {
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
        0x0C => Some(if shift { b'_' } else { b'-' }),
        0x0D => Some(if shift { b'+' } else { b'=' }),
        0x0E => Some(b'\x08'),     // Backspace
        0x0F => Some(b'\t'),       // Tab
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
        0x1A => Some(if shift { b'{' } else { b'[' }),
        0x1B => Some(if shift { b'}' } else { b']' }),
        0x1C => Some(b'\n'),       // Enter
        0x1E => Some(if shift { b'A' } else { b'a' }),
        0x1F => Some(if shift { b'S' } else { b's' }),
        0x20 => Some(if shift { b'D' } else { b'd' }),
        0x21 => Some(if shift { b'F' } else { b'f' }),
        0x22 => Some(if shift { b'G' } else { b'g' }),
        0x23 => Some(if shift { b'H' } else { b'h' }),
        0x24 => Some(if shift { b'J' } else { b'j' }),
        0x25 => Some(if shift { b'K' } else { b'k' }),
        0x26 => Some(if shift { b'L' } else { b'l' }),
        0x27 => Some(if shift { b':' } else { b';' }),
        0x28 => Some(if shift { b'"' } else { b'\'' }),
        0x2B => Some(if shift { b'|' } else { b'\\' }),
        0x2C => Some(if shift { b'Z' } else { b'z' }),
        0x2D => Some(if shift { b'X' } else { b'x' }),
        0x2E => Some(if shift { b'C' } else { b'c' }),
        0x2F => Some(if shift { b'V' } else { b'v' }),
        0x30 => Some(if shift { b'B' } else { b'b' }),
        0x31 => Some(if shift { b'N' } else { b'n' }),
        0x32 => Some(if shift { b'M' } else { b'm' }),
        0x33 => Some(if shift { b'<' } else { b',' }),
        0x34 => Some(if shift { b'>' } else { b'.' }),
        0x35 => Some(if shift { b'?' } else { b'/' }),
        0x48 => Some(0x80), // Стрелка вверх
        0x50 => Some(0x81), // Стрелка вниз
        0x39 => Some(b' '), // Пробел
        _ => None,
    }
}

// Вызывается из обработчика прерывания клавиатуры (interrupts.rs)
// Работает ТОЛЬКО с данными из порта 0x60, не трогает порты 0x64/0x60
pub unsafe fn handle_scancode(scancode: u8) {
    // Отпускание клавиши (старший бит = 1)
    if scancode & 0x80 != 0 {
        if scancode == 0xAA || scancode == 0xB6 {
            SHIFT_PRESSED = false;
        }
        return;
    }

    // Нажатие Shift (левый или правый)
    if scancode == 0x2A || scancode == 0x36 {
        SHIFT_PRESSED = true;
        return;
    }

    // Преобразуем сканкод в ASCII и кладём в циклический буфер
    if let Some(ascii) = scancode_to_ascii(scancode) {
        let next = (KEY_BUF_TAIL + 1) % 32;
        if next != KEY_BUF_HEAD {
            KEY_BUFFER[KEY_BUF_TAIL] = ascii;
            KEY_BUF_TAIL = next;
        }
    }
}

// Пытается прочитать символ из буфера (неблокирующий вызов)
fn try_read_key() -> Option<u8> {
    unsafe {
        if KEY_BUF_HEAD != KEY_BUF_TAIL {
            let c = KEY_BUFFER[KEY_BUF_HEAD];
            KEY_BUF_HEAD = (KEY_BUF_HEAD + 1) % 32;
            Some(c)
        } else {
            None
        }
    }
}

// Настройка контроллера клавиатуры на генерацию IRQ1
pub unsafe fn enable_interrupts() {
    // Ждём, пока буфер клавиатуры освободится
    while (inb(0x64) & 2) != 0 {}
    // Команда чтения байта конфигурации
    outb(0x64, 0x20);
    // Ждём данные
    while (inb(0x64) & 1) == 0 {}
    let mut config = inb(0x60);
    config |= 0x01;  // Устанавливаем бит 0 (Enable Interrupt)
    // Записываем обратно
    while (inb(0x64) & 2) != 0 {}
    outb(0x64, 0x60);
    while (inb(0x64) & 2) != 0 {}
    outb(0x60, config);
}

// Очистка буфера клавиатуры (на случай pending scancodes)
pub unsafe fn flush_buffer() {
    while (inb(0x64) & 1) == 1 {
        inb(0x60);
    }
}

pub unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    core::arch::asm!("in al, dx", out("al") result, in("dx") port);
    result
}

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}

// Блокирующее чтение — ждём прерывание через HLT
// Прерывание от клавиатуры разбудит CPU и заполнит буфер
pub fn read_key_blocking() -> u8 {
    loop {
        // Проверяем буфер (заполняется прерываниями клавиатуры)
        if let Some(c) = try_read_key() {
            return c;
        }
        // Спим до следующего прерывания (любого, но клавиатура разбудит)
        unsafe {
            core::arch::asm!("sti; hlt; cli", options(nomem));
        }
    }
}

// Читает строку с помощью прерываний клавиатуры
pub fn read_line() -> [u8; 64] {
    let mut buf = [0u8; 64];
    let mut pos = 0;

    loop {
        let key = read_key_blocking();
        match key {
            b'\n' => {
                buf[pos] = 0;
                vga::put_char(b'\n', Color::LightGray);
                return buf;
            }
            b'\x08' => {
                if pos > 0 {
                    pos -= 1;
                    vga::backspace();
                }
            }
            c if pos < 63 && c >= 0x20 => {
                buf[pos] = c;
                pos += 1;
                vga::put_char(c, Color::LightGray);
            }
            _ => {}
        }
    }
}