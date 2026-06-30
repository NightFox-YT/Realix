// © Realix > VGA
// (26.06.26) v0.07
// ================

// Подключение модулей
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

// Константы
const VGA_BUFFER: *mut u8 = 0xB8000 as *mut u8;
pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;

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


pub fn clear_screen() {
    // "Стираем" экран пробелами с чёрным фоном
    for byte in 0..(VGA_WIDTH * VGA_HEIGHT) {
        unsafe {
            VGA_BUFFER.add(byte * 2).write_volatile(b' ');
            VGA_BUFFER.add(byte * 2 + 1).write_volatile(0x0F);
        }
    }

    // Сбрасываем позицию курсора
    CURSOR_ROW.store(0, Relaxed);
    CURSOR_COL.store(0, Relaxed);
    update_cursor();
}


// > Поднять все строки на экране на `n` позиций
fn scroll_up(lines_count: usize) {
    // Если кол-во строк для прокрутки больше чем высота VGA
    if lines_count >= VGA_HEIGHT {
        clear_screen();
        return;
    }

    // Копируем все строки кроме первой на `n` позиций выше
    for row in lines_count..VGA_HEIGHT {
        let src_offset: usize = (row * VGA_WIDTH) * 2;
        let dst_offset: usize = ((row - lines_count) * VGA_WIDTH) * 2;

        for byte in 0..(VGA_WIDTH * 2) {
            unsafe {
                let value: u8 = VGA_BUFFER.add(src_offset + byte).read_volatile();
                VGA_BUFFER.add(dst_offset + byte).write_volatile(value);
            }
        }
    }

    // "Стираем" последние `n` строк пробелами с чёрным фоном
    for row in (VGA_HEIGHT - lines_count)..VGA_HEIGHT {
        let row_offset: usize = row * VGA_WIDTH * 2;

        for col in 0..VGA_WIDTH {
            unsafe {
                VGA_BUFFER.add(row_offset + col * 2).write_volatile(b' ');
                VGA_BUFFER.add(row_offset + col * 2 + 1).write_volatile(0x0F);
            }
        }
    }

    // Обновляем позицию курсора на `n` строк вверх
    let _row: usize = CURSOR_ROW.load(Relaxed);
    CURSOR_ROW.store(_row.saturating_sub(lines_count), Relaxed);
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


// > Вывод символа на экран (с принципами TTY)
pub fn print_char(char_byte: u8, color: Color) {
    match char_byte {
        b'\n' => {
            CURSOR_COL.store(0, Relaxed);
            CURSOR_ROW.fetch_add(1, Relaxed); 
        }
        b'\r' => { CURSOR_COL.store(0, Relaxed); }
        _ => {
            let row = CURSOR_ROW.load(Relaxed);
            let col = CURSOR_COL.load(Relaxed);
            let offset = (row * VGA_WIDTH + col) * 2;
            unsafe {
                VGA_BUFFER.add(offset).write_volatile(char_byte);
                VGA_BUFFER.add(offset + 1).write_volatile(color as u8);
            }
            CURSOR_COL.fetch_add(1, Relaxed);
        }
    }

    if CURSOR_COL.load(Relaxed) >= VGA_WIDTH {
        CURSOR_COL.store(0, Relaxed);
        CURSOR_ROW.fetch_add(1, Relaxed); 
    }

    while CURSOR_ROW.load(Relaxed) >= VGA_HEIGHT {
        scroll_up(1);
    }

    update_cursor();
}


pub fn print_str(string: &str, color: Color) {
    for byte in string.bytes() {
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
