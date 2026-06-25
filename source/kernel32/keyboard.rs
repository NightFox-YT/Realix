use core::arch::asm;
use crate::vga::{self, Color};

const INPUT_MAX: usize = 64;
static mut SHIFT_PRESSED: bool = false;

fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    let shift = unsafe { SHIFT_PRESSED };
    match scancode {
        // Цифры
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
        // Символы
        0x0C => Some(if shift { b'_' } else { b'-' }),
        0x0D => Some(if shift { b'+' } else { b'=' }),
        // Backspace
        0x0E => Some(b'\x08'),
        // Tab
        0x0F => Some(b'\t'),
        // Буквы
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
        0x1C => Some(b'\n'), // Enter
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
        0x29 => Some(if shift { b'~' } else { b'`' }),
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
        // Пробел
        0x39 => Some(b' '),
        // Shift
        0x2A | 0x36 => {
            unsafe { SHIFT_PRESSED = true; }
            None
        }
        0xAA | 0xB6 => {
            unsafe { SHIFT_PRESSED = false; }
            None
        }
        _ => None,
    }
}

pub fn read_key_blocking() -> u8 {
    loop {
        unsafe {
            let status: u8;
            asm!("in al, 0x64", out("al") status);
            if status & 1 != 0 {
                if status & 0x20 != 0 {
                    let _trash: u8;
                    asm!("in al, 0x60", out("al") _trash);
                    continue;
                }
                let scancode: u8;
                asm!("in al, 0x60", out("al") scancode);
                
                // Обрабатываем нажатие и отпускание
                let pressed = scancode & 0x80 == 0;
                let key_code = scancode & 0x7F;
                
                // Обновляем Shift
                if key_code == 0x2A || key_code == 0x36 {
                    if pressed {
                        unsafe { SHIFT_PRESSED = true; }
                    } else {
                        unsafe { SHIFT_PRESSED = false; }
                    }
                    continue;
                }
                
                if pressed {
                    if let Some(ascii) = scancode_to_ascii(key_code) {
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
            c if pos < INPUT_MAX - 1 && c >= 0x20 => {
                buf[pos] = c;
                pos += 1;
                vga::put_char(c, Color::LightGray);
            }
            _ => {}
        }
    }
}
