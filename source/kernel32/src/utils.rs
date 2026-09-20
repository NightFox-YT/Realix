// © Realix > Utils
// (27.07.26) v0.1
// ================

// Импорт функций
use core::arch::asm;


/// Перевод u32 числа в hex-строку (вида 0x000FCABC)
/// Параметры:
///  - value: значение для перевода
///  - str_buffer: буфер рассчитан на "0x" + 8 цифр
pub fn u32_to_hex_str(value: u32, str_buffer: &mut [u8; 10]) -> &str {
    // Добавляем в буфер шестнадцатеричный префикс
    str_buffer[0] = b'0';
    str_buffer[1] = b'x';

    // Перебираем цифры в числе
    for i in 0..8 {
        // Сдвигаем след. цифру на младшие 4 бита (Одна цифра в hex)
        let digit: u8 = ((value >> ((7 - i) * 4)) & 0xF) as u8;

        str_buffer[2 + i] = match digit {
            0..=9 => b'0' + digit,     // Цифры
            _ => b'A' + (digit - 10),  // Буквы
        };
    }

    // Буфер из ASCII-символов превращаем в строку
    core::str::from_utf8(str_buffer).unwrap()
}


/// Перевод u32 числа в dec-строку
/// Параметры:
///  - value: значение для перевода
///  - str_buffer: буфер рассчитан на 10 цифр
pub fn u32_to_dec_str(value: u32, str_buffer: &mut [u8; 10]) -> &str {
    let mut value_copy: u32 = value;
    let mut i: usize = str_buffer.len();

    // Особый случай
    if value == 0 {
        i -= 1;
        str_buffer[i] = b'0';
    }

    // Заполняем буфер в обратном порядке
    while value_copy > 0 {
        i -= 1;
        str_buffer[i] = b'0' + (value_copy % 10) as u8;
        value_copy /= 10;
    }

    // Буфер из ASCII-символов превращаем в строку
    core::str::from_utf8(&str_buffer[i..]).unwrap()
}


/// Обработка беззнакового десятичного 32-битного числа из строки
pub fn parse_u32(s: &str) -> Option<u32> {
    // Пустая строка не подходит
    if s.is_empty() {
        return None;
    }

    let mut value: u32 = 0;
    for byte in s.bytes() {
        // Если символ не является цифрой, то выходим
        if !byte.is_ascii_digit() {
            return None;
        }

        // Добавляем цифру к значению с защитой от переполнения
        value = value.checked_mul(10)?.checked_add((byte - b'0') as u32)?;
    }
    Some(value)
}


/// Запись байта (u8) в I/O-порт
#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nostack, nomem, preserves_flags),
    );
}


/// Запись слова (u16) в I/O-порт
#[inline(always)]
pub unsafe fn outw(port: u16, value: u16) {
    asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") value,
        options(nostack, nomem, preserves_flags),
    );
}


/// Запись 32-битного слова (long) числа в I/O-порт
#[inline(always)]
pub unsafe fn outl(port: u16, value: u32) {
    asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") value,
        options(nostack, nomem, preserves_flags),
    );
}


/// Чтение байта (u8) из I/O-порта
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nostack, nomem, preserves_flags),
    );

    value
}


/// Чтение слова (u16) из I/O-порта
#[inline(always)]
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;

    asm!(
        "in ax, dx",
        in("dx") port,
        out("ax") value,
        options(nostack, nomem, preserves_flags),
    );

    value
}


/// Чтение 32-битного слова (long) числа из I/O-порта
#[inline(always)]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;

    asm!(
        "in eax, dx",
        in("dx") port,
        out("eax") value,
        options(nostack, nomem, preserves_flags),
    );

    value
}

/// Задержка IO-порта
#[inline(always)]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}
