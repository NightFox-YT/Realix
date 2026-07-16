// © Realix > VGA
// (16.07.26) v0.1
// ø Вдохновлено @liquifield
// ================
// ! Не вызывать из IRQ прерываний (Гонка данных на константах)

// Подключение функций
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use crate::utils;

// Константы
const VGA_BUFFER: *mut u8 = 0xB8000 as *mut u8;
pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
const EMPTY_CELL: u16 = (0x0F << 8) | b' ' as u16;  // Пробел + 0x0F (белый на чёрном)

static CURSOR_ROW: AtomicUsize = AtomicUsize::new(0);
static CURSOR_COL: AtomicUsize = AtomicUsize::new(0);

// Таблица цветов
#[allow(dead_code)]
#[derive(Clone, Copy)]
#[repr(u8)]
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

/// > Заполнение `count` ячеек экрана пустой ячейкой, начиная с `start_cell`
fn clear_cells(start_cell: usize, count: usize) {
    let cells: *mut u16 = VGA_BUFFER as *mut u16;

    for i in start_cell..start_cell + count {
        unsafe { cells.add(i).write_volatile(EMPTY_CELL); }
    }
}

pub fn clear_screen() {
    // "Стираем" экран пробелами с чёрным фоном
    clear_cells(0, VGA_WIDTH * VGA_HEIGHT);

    // Сбрасываем позицию курсора
    CURSOR_ROW.store(0, Relaxed);
    CURSOR_COL.store(0, Relaxed);
    update_cursor();
}


/// Поднять все строки на экране на `n` позиций
fn scroll_up(lines_count: usize) {
    // Если кол-во строк для прокрутки больше чем высота VGA
    if lines_count >= VGA_HEIGHT {
        clear_screen();
        return;
    }

    // Копируем строки `n`..HEIGHT в начало экрана единым блоком (memmove).
    unsafe {
        core::ptr::copy(
            VGA_BUFFER.add(lines_count * VGA_WIDTH * 2),
            VGA_BUFFER,
            (VGA_HEIGHT - lines_count) * VGA_WIDTH * 2
        );
    }

    // "Стираем" последние `n` строк пробелами с чёрным фоном
    clear_cells((VGA_HEIGHT - lines_count) * VGA_WIDTH, lines_count * VGA_WIDTH);

    // Обновляем позицию курсора на `n` строк вверх
    let _row: usize = CURSOR_ROW.load(Relaxed);
    CURSOR_ROW.store(if _row > lines_count { _row - lines_count } else { 0 }, Relaxed);
    update_cursor();
}


fn update_cursor() {
    // Вычисляем новую позицию
    let _row: usize = CURSOR_ROW.load(Relaxed);
    let _col: usize = CURSOR_COL.load(Relaxed);
    let pos: u16 = (_row * VGA_WIDTH + _col) as u16;

    // Включаем порт VGA и передаём младший, затем старший байт позиции курсора
    unsafe {
        asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Fu8);
        asm!("out dx, al", in("dx") 0x3D5u16, in("al") (pos & 0xFF) as u8);

        asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Eu8);
        asm!("out dx, al", in("dx") 0x3D5u16, in("al") ((pos >> 8) & 0xFF) as u8);
    }
}


/// Вывод символа на экран (по принципам TTY)
pub fn print_char(char_byte: u8, color: Color) {
    match char_byte {
        b'\n' => {
            CURSOR_COL.store(0, Relaxed);
            CURSOR_ROW.fetch_add(1, Relaxed); 
        }
        b'\r' => { CURSOR_COL.store(0, Relaxed); }
        _ => {
            let row: usize = CURSOR_ROW.load(Relaxed);
            let col: usize = CURSOR_COL.load(Relaxed);
            let offset: usize = (row * VGA_WIDTH + col) * 2;
            unsafe {
                VGA_BUFFER.add(offset).write_volatile(char_byte);
                VGA_BUFFER.add(offset + 1).write_volatile(color as u8);
            }
            CURSOR_COL.fetch_add(1, Relaxed);
        }
    }

    // Перенос курсора на след. строку
    if CURSOR_COL.load(Relaxed) >= VGA_WIDTH {
        CURSOR_COL.store(0, Relaxed);
        CURSOR_ROW.fetch_add(1, Relaxed); 
    }

    // Если курсор выходит за нижнюю границу экрана
    while CURSOR_ROW.load(Relaxed) >= VGA_HEIGHT {
        scroll_up(1);
    }

    update_cursor();
}

/// Вывод символа в определённой позиции
pub fn write_char_at(row: usize, col: usize, char_byte: u8, color: Color) {
    // За границу экрана мы не пишем
    if row >= VGA_HEIGHT || col >= VGA_WIDTH {
        return;
    }

    let offset: usize = (row * VGA_WIDTH + col) * 2;
    unsafe {
        VGA_BUFFER.add(offset).write_volatile(char_byte);
        VGA_BUFFER.add(offset + 1).write_volatile(color as u8);
    }
}

/// Вывод строки на экран (по принципам TTY)
pub fn print_line(line: &str, color: Color) {
    for byte in line.bytes() {
        print_char(byte, color);
    }
}

pub fn print_backspace() {
    // > Обновление позиции курсора
    if CURSOR_COL.load(Relaxed) > 0 {
        CURSOR_COL.fetch_sub(1, Relaxed);
    } else if CURSOR_ROW.load(Relaxed) > 0 {
        CURSOR_ROW.fetch_sub(1, Relaxed); 
        CURSOR_COL.store(VGA_WIDTH - 1, Relaxed);
    }

    // Замена последнего символа на пробел
    let _row: usize = CURSOR_ROW.load(Relaxed);
    let _col: usize = CURSOR_COL.load(Relaxed);
    let offset: usize = (_row * VGA_WIDTH + _col) * 2;
    unsafe {
        VGA_BUFFER.add(offset).write_volatile(b' ');
        VGA_BUFFER.add(offset + 1).write_volatile(0x0F);
    }

    update_cursor();
}

/// Перевод строки
pub fn new_line() {
    print_char(b'\n', Color::White);
}

/// Вывод строки дампа регистра
pub fn print_reg_line(reg_label: &str, value: u32) {
    let mut buffer: [u8; 10] = [0u8; 10];

    print_line("> ", Color::LightGray);
    print_line(reg_label, Color::LightGray);
    print_line(" = ", Color::LightGray);
    print_line(utils::u32_to_hex_str(value, &mut buffer), Color::White);
    new_line();
}