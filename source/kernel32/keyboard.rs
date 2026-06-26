use core::arch::asm;
use crate::vga::{self, Color};

const INPUT_MAX: usize = 64;

fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    match scancode {
        // Цифры
        0x02 => Some(b'1'), 0x03 => Some(b'2'), 0x04 => Some(b'3'),
        0x05 => Some(b'4'), 0x06 => Some(b'5'), 0x07 => Some(b'6'),
        0x08 => Some(b'7'), 0x09 => Some(b'8'), 0x0A => Some(b'9'),
        0x0B => Some(b'0'),
        // Спецсимволы на цифрах (Shift пока не обрабатываем — будут строчные)
        0x0C => Some(b'-'), 0x0D => Some(b'='),
        // Backspace
        0x0E => Some(b'\x08'),
        // Tab
        0x0F => Some(b'\t'),
        // Буквы
        0x10 => Some(b'q'), 0x11 => Some(b'w'), 0x12 => Some(b'e'),
        0x13 => Some(b'r'), 0x14 => Some(b't'), 0x15 => Some(b'y'),
        0x16 => Some(b'u'), 0x17 => Some(b'i'), 0x18 => Some(b'o'),
        0x19 => Some(b'p'),
        0x1A => Some(b'['), 0x1B => Some(b']'),
        0x1C => Some(b'\n'), // Enter
        0x1E => Some(b'a'), 0x1F => Some(b's'), 0x20 => Some(b'd'),
        0x21 => Some(b'f'), 0x22 => Some(b'g'), 0x23 => Some(b'h'),
        0x24 => Some(b'j'), 0x25 => Some(b'k'), 0x26 => Some(b'l'),
        0x27 => Some(b';'), 0x28 => Some(b'\''),
        0x29 => Some(b'`'),  // backtick
        0x2B => Some(b'\\'),
        0x2C => Some(b'z'), 0x2D => Some(b'x'), 0x2E => Some(b'c'),
        0x2F => Some(b'v'), 0x30 => Some(b'b'), 0x31 => Some(b'n'),
        0x32 => Some(b'm'),
        0x33 => Some(b','), 0x34 => Some(b'.'),
        0x35 => Some(b'/'),
        // Пробел
        0x39 => Some(b' '),
        _ => None,
    }
}

pub fn read_key_blocking() -> u8 {
    loop {
        unsafe {
            let status: u8;
            asm!("in al, 0x64", out("al") status);
            if status & 1 != 0 {
                // Игнорируем данные мыши (бит 5)
                if status & 0x20 != 0 {
                    let _trash: u8;
                    asm!("in al, 0x60", out("al") _trash);
                    continue;
                }
                let scancode: u8;
                asm!("in al, 0x60", out("al") scancode);
                
                // Только нажатия, не отпускания
                if scancode & 0x80 == 0 {
                    if let Some(ascii) = scancode_to_ascii(scancode) {
                        return ascii;
                    }
                }
            }
        }
    }
}

pub fn read_line() -> [u8; INPUT_MAX] {
    let mut buf = [0u8; INPUT_MAX];
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
            c if pos < INPUT_MAX - 1 && c >= 0x20 && c <= 0x7E => {
                buf[pos] = c;
                pos += 1;
                vga::put_char(c, Color::LightGray);
            }
            _ => {}
        }
    }
}
