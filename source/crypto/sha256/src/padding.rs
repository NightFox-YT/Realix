// Copyright Gleb Obitotsky <https://github.com/oxxx1mif> 2026.
//
// License: GNU General Public License v3
// You can find the license file in the project root.
//
// Implementation crypto version 0.1
// The code was written for Realix.
// 19 june 2026

// Thanks to the authors:
// https://datatracker.ietf.org/doc/html/rfc6234

const BLOCK_SIZE: usize = 64;

const LENGTH_SIZE: usize = 8;

const LENGTH_OFFSET: usize = BLOCK_SIZE - LENGTH_SIZE;

/// Проверяет, нужен ли дополнительный блок.
/// SHA-256 резервирует последние 64 бита
/// блока под длину сообщения.
/// Если данных >= 56 байт,
/// поле длины уже не помещается после 0x80.
#[inline(always)]
pub(crate) const fn needs_extra_block(
    buffer_len: usize,
) -> bool {
    buffer_len >= LENGTH_OFFSET
}

/// Добавляет SHA-256 padding в текущий блок.
/// message
/// || 1 bit
/// || zero padding
/// || 64-bit message length
/// Возвращает:
/// false:
///     padding полностью помещается
/// true:
///     требуется второй блок
/// buffer_len должен быть < 64
#[inline(always)]
pub(crate) fn pad(
    buffer: &mut [u8; BLOCK_SIZE],
    buffer_len: usize,
    bit_len: u64,
) -> bool {

    debug_assert!(
        buffer_len < BLOCK_SIZE
    );

    // Добавляем обязательный бит '1'
    buffer[buffer_len] = 0x80;

    // Если исходных данных >= 56 байт,
    // длина не помещается.
    if needs_extra_block(buffer_len) {
        // Заполняем остаток текущего блока нулями.
        clear(
            &mut buffer[buffer_len + 1 .. BLOCK_SIZE]
        );
        return true;
    }

    // Заполняем нулями до поля длины.
    clear(
        &mut buffer[buffer_len + 1 .. LENGTH_OFFSET]
    );

    // Записываем длину сообщения.
    write_length(
        buffer,
        bit_len
    );

    false
}

/// Создает второй padding-блок.
/// Используется если:
/// Первый блок:
/// message + 0x80 + zeros
/// Второй блок:
/// zeros + message length
#[inline(always)]
pub(crate) fn pad_second_block(
    buffer: &mut [u8; BLOCK_SIZE],
    bit_len: u64,
) {
    // Полностью очищаем блок.
    *buffer = [0u8; BLOCK_SIZE];

    write_length(
        buffer,
        bit_len
    );
}

/// Записывает длину сообщения.
/// SHA-256 использует.
/// 64-bit unsigned integer
#[inline(always)]
pub(crate) fn write_length(
    buffer: &mut [u8; BLOCK_SIZE],
    bit_len: u64,
) {
    buffer[LENGTH_OFFSET..BLOCK_SIZE]
        .copy_from_slice(
            &bit_len.to_be_bytes()
        );
}

/// Очистка диапазона памяти.
#[inline(always)]
fn clear(
    data: &mut [u8],
) {
    for byte in data {

        *byte = 0;
    }
}

#[cfg(test)]
mod padding_tests {
    use crate::padding::{self, *};

    // ------------------------------------------------------------------
    // needs_extra_block
    // ------------------------------------------------------------------
    #[test]
    fn needs_extra_block_boundaries() {
        // 55 байт — поле длины ещё влезает
        assert!(!needs_extra_block(55));
        // 56 байт — уже не влезает
        assert!(needs_extra_block(56));
        assert!(needs_extra_block(63));
    }

    // ------------------------------------------------------------------
    // pad — пустое сообщение (0 байт)
    // ------------------------------------------------------------------
    #[test]
    fn pad_empty_message() {
        let mut buffer = [0xFFu8; BLOCK_SIZE]; // мусор для проверки очистки
        let need_second = pad(&mut buffer, 0, 0);
        assert!(!need_second);

        // Первый байт — 0x80
        assert_eq!(buffer[0], 0x80);

        // Баты 1..LENGTH_OFFSET должны быть нулями
        for i in 1..LENGTH_OFFSET {
            assert_eq!(buffer[i], 0, "byte {} non-zero", i);
        }

        // Длина 0 в последних 8 байтах
        assert_eq!(&buffer[LENGTH_OFFSET..], &[0u8; 8]);
    }

    // ------------------------------------------------------------------
    // pad — 5 байт (общий случай с нулями между 0x80 и длиной)
    // ------------------------------------------------------------------
    #[test]
    fn pad_5_bytes() {
        let mut buffer = [0xFFu8; BLOCK_SIZE];
        for i in 0..5 {
            buffer[i] = 0xAA; // произвольные данные
        }
        let bit_len = 5 * 8;
        let need_second = pad(&mut buffer, 5, bit_len);
        assert!(!need_second);

        assert_eq!(buffer[5], 0x80);
        // байты 6..LENGTH_OFFSET (т.е. 6..56) — нули
        for i in 6..LENGTH_OFFSET {
            assert_eq!(buffer[i], 0);
        }
        // длина
        let len_bytes = &buffer[LENGTH_OFFSET..];
        assert_eq!(u64::from_be_bytes(len_bytes.try_into().unwrap()), bit_len);
    }

    // ------------------------------------------------------------------
    // pad — ровно 55 байт (0x80 и длина прилегают вплотную)
    // ------------------------------------------------------------------
    #[test]
    fn pad_55_bytes_exact_fit() {
        let mut buffer = [0xFFu8; BLOCK_SIZE];
        for i in 0..55 {
            buffer[i] = 0x61; // 'a'
        }
        let bit_len = 55 * 8;
        let need_second = pad(&mut buffer, 55, bit_len);
        assert!(!need_second);

        // 0x80 сразу за данными
        assert_eq!(buffer[55], 0x80);
        // нулей между 0x80 и длиной быть не должно (56 == LENGTH_OFFSET)
        let len_bytes = &buffer[LENGTH_OFFSET..];
        assert_eq!(u64::from_be_bytes(len_bytes.try_into().unwrap()), bit_len);
    }

    // ------------------------------------------------------------------
    // pad — 56 байт (требуется второй блок)
    // ------------------------------------------------------------------
    #[test]
    fn pad_56_bytes_extra_block() {
        let mut buffer = [0xFFu8; BLOCK_SIZE];
        for i in 0..56 {
            buffer[i] = 0x62; // 'b'
        }
        let bit_len = 56 * 8;
        let need_second = pad(&mut buffer, 56, bit_len);
        assert!(need_second);

        // 0x80 на позиции 56
        assert_eq!(buffer[56], 0x80);
        // остаток буфера (57..64) обнулён
        for i in 57..BLOCK_SIZE {
            assert_eq!(buffer[i], 0);
        }

        // Проверяем второй блок через pad_second_block
        let mut second_buffer = [0xCCu8; BLOCK_SIZE];
        pad_second_block(&mut second_buffer, bit_len);
        // Все байты, кроме последних 8, нули
        for i in 0..LENGTH_OFFSET {
            assert_eq!(second_buffer[i], 0);
        }
        let len_bytes = &second_buffer[LENGTH_OFFSET..];
        assert_eq!(u64::from_be_bytes(len_bytes.try_into().unwrap()), bit_len);
    }

    // ------------------------------------------------------------------
    // pad — 63 байта (максимально возможный buffer_len < 64)
    // ------------------------------------------------------------------
    #[test]
    fn pad_63_bytes_max() {
        let mut buffer = [0xFFu8; BLOCK_SIZE];
        for i in 0..63 {
            buffer[i] = 0x63;
        }
        let bit_len = 63 * 8;
        let need_second = pad(&mut buffer, 63, bit_len);
        assert!(need_second);

        // 0x80 попадает на последний байт
        assert_eq!(buffer[63], 0x80);
    }

    // ------------------------------------------------------------------
    // write_length — проверка записи длины
    // ------------------------------------------------------------------
    #[test]
    fn write_length_correct() {
        let mut buffer = [0xFFu8; BLOCK_SIZE];
        write_length(&mut buffer, 0x1234567890ABCDEF);
        let len_bytes = &buffer[LENGTH_OFFSET..];
        assert_eq!(
            u64::from_be_bytes(len_bytes.try_into().unwrap()),
            0x1234567890ABCDEF
        );
    }

    // ------------------------------------------------------------------
    // clear — очистка диапазона
    // ------------------------------------------------------------------
    #[test]
    fn clear_range() {
        let mut data = [0xFFu8; 10];
        crate::padding::clear(&mut data[2..8]);
        assert_eq!(data, [0xFF, 0xFF, 0,0,0,0,0,0, 0xFF, 0xFF]);
    }
}