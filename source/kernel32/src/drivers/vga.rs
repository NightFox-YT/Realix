// © Realix > VGA
// (25.06.26) v0.07
// ================

// Константы
const VGA_BUFFER: *mut u8 = 0xB8000 as *mut u8; // Указатель
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

// Таблица цветов (1 байт - u8)
#[repr(u8)]
#[allow(dead_code)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

// Очистка экрана: Закрашивает весь экран пробелами
pub fn clear_screen() {
    for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
        unsafe {
            // Записываем в vga буфер пробелы (белый на чёрном)
            VGA_BUFFER.add(i * 2).write_volatile(b' ');
            VGA_BUFFER.add(i * 2 + 1).write_volatile(0x0F);
        }
    }
}

/// Вывод строки на экран
pub fn print_str(row: usize, col: usize, s: &str, color: Color) {
    let color_byte = color as u8;
    let start_offset = (row * VGA_WIDTH + col) * 2;

    // Цикл вывода на экран по одному символу
    for (i, byte) in s.bytes().enumerate() {
        unsafe {
            VGA_BUFFER.add(start_offset + i * 2).write_volatile(byte);
            VGA_BUFFER.add(start_offset + i * 2 + 1).write_volatile(color_byte);
        }
    }
}
