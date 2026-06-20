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