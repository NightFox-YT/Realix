// © Realix > Utils
// (03.07.26) v0.08
// ================

/// > Перевод u32 числа в hex-строку (вида 0xFCABC)
pub fn u32_to_hex_str(value: u32, str_buffer: &mut [u8; 10]) -> &str {
    // Добавляем в буффер шестнацатеричный префикс
    str_buffer[0] = b'0';
    str_buffer[1] = b'x';

    for i in 0..8 {
        // Сдвигаем цифру на младшиие 4 бита (Одна цифра в hex)
        let digit: u8 = ((value >> ((7 - i) * 4)) & 0xF) as u8;

        str_buffer[2 + i] = match digit {
            0..=9 => b'0' + digit,     // Цифры
            _ => b'A' + (digit - 10),  // Буквы
        };
    }

    // Буфер из ASCII-символов превращаем в строку
    core::str::from_utf8(str_buffer).unwrap()
}

/// > Перевод u32 числа в dec-строку
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