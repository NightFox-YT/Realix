// © Realix > Kernel32: Main
// (11.07.26) v0.09
// ================

#![no_std]
#![no_main]

// Объявление модулей
mod commands;
mod drivers;
mod shell;
mod utils;
mod x86;

// Подключение функций
use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;

use drivers::{keyboard, pit, vga};
use x86::{gdt, idt, memory};
// use core::ptr::read_unaligned;

/// Структура PCINFO, формируемая загрузчиком.
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct PcInfo {
    low_memory_amount: u16,
    disk_num: u8,
    memory_map: memory::E820Map,
}


// Границы секции BSS (определены в linker.ld) для обнуления вручную
unsafe extern "C" {
    unsafe static __bss_start: u8;
    unsafe static __bss_end: u8;
}

/// Низкоуровневая точка входа Kernel32 (Настройка окружения)
/// При входе: EBX = адрес PCINFO, ESP = стек от загрузчика.
#[link_section = ".text.entry"]
#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    naked_asm!(
        // Устанавливаем DF & Сохраняем переданный адрес PCINFO
        "cld",
        "push ebx",

        // Установка: EDI - начало BSS, ECX - её размер
        "lea edi, [__bss_start]",
        "lea ecx, [__bss_end]",
        "sub ecx, edi",

        // Обнуляем BSS через eax
        "xor eax, eax",
        "rep stosb",

        // Восстанавливаем адрес PCINFO и передаём первым аргументом по cdecl
        "pop ebx",
        "push ebx",
        "call kmain",

        // Защита на случай незапланированного возвращения из функции
        "2:",
        "cli",
        "hlt",
        "jmp 2b"
    );
}


/// Основный цикл работы ядра
#[no_mangle]
extern "C" fn kmain(pcinfo_addr: *const PcInfo) -> ! {
    // Инициализация модулей
    idt::interrupts_disable();
    gdt::init();
    idt::init();
    pit::init(100);
    idt::interrupts_enable();

    // Проверка указателя PCINFO
    if pcinfo_addr.is_null() {
        vga::clear_screen();
        vga::print_line(
            "[KERNEL PANIC] Invalid PCINFO address.\n",
            vga::Color::Red,
        );
        halt_loop();
    }

    unsafe { memory::init_from_pcinfo(pcinfo_addr as usize); }

    vga::clear_screen();
    draw_logo(4, 2);
    for _ in 0..11 { vga::new_line(); }

    vga::print_line(
        "   Press any key to continue...",
        vga::Color::LightGray,
    );
    keyboard::read_key();
    
    // Вывод заголовка Shell с его бесконечной работой
    vga::clear_screen();
    vga::print_line(
        "Welcome to Realix (Protected Mode with Rust kernel)...\n",
        vga::Color::Cyan,
    );

    shell::run();
    halt_loop();
}


/// Отрисовка логотипа Realix
fn draw_logo(start_x: usize, start_y: usize) {
    // Массив из 2 уровней:
    // 1) Массивы для каждлой буквы
    // 2) Массив для каждой строки буквы (0 - пробел, 1 - блок)
    let letters: [[[u8; 6]; 8]; 6] = [
        [[1,1,1,1,0,0],[1,0,0,1,0,0],[1,0,0,1,0,0],[1,1,1,1,0,0],
         [1,1,0,0,0,0],[1,0,1,0,0,0],[1,0,0,1,0,0],[1,0,0,1,0,0]],
        [[1,1,1,1,1,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,1,1,1,0,0],
         [1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,1,1,1,1,0]],
        [[0,1,1,1,0,0],[1,0,0,0,1,0],[1,0,0,0,1,0],[1,1,1,1,1,0],
         [1,0,0,0,1,0],[1,0,0,0,1,0],[1,0,0,0,1,0],[1,0,0,0,1,0]],
        [[1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],
         [1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,1,1,1,1,0]],
        [[0,1,1,1,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],
         [0,0,1,0,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],[0,1,1,1,0,0]],
        [[1,0,0,0,1,0],[0,1,0,1,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],
         [0,0,1,0,0,0],[0,1,0,1,0,0],[1,0,0,0,1,0],[1,0,0,0,1,0]],
    ];

    // Список цветов для букв (1 буква - 1 цвет)
    let colors = [
        vga::Color::Red,
        vga::Color::Yellow,
        vga::Color::Green,
        vga::Color::Cyan,
        vga::Color::Blue,
        vga::Color::Magenta,
    ];

    for (letter_idx, letter) in letters.iter().enumerate() {
        let color = colors[letter_idx % colors.len()];
        let offset_x = start_x + letter_idx * letter[0].len();

        for (row, line) in letter.iter().enumerate() {
            for (col, &pixel) in line.iter().enumerate() {
                if pixel == 1 {
                    vga::write_char_at(
                        start_y + row, offset_x + col,
                        0xDB, color,
                    );
                }
            }
        }
    }
}


/// Бесконечная остановка процессора
pub fn halt_loop() -> ! {
    idt::interrupts_disable();
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)); }
    }
}


/// Обработчик паники
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_loop()
}


/// Запись байта в I/O-порт
#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nostack, nomem, preserves_flags),
    );
}


/// Чтение байта из I/O-порта
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
