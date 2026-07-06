// © Realix > Kernel32: Main
// (03.07.26) v0.08
// ================
#![no_std]
#![no_main]

// Объявление модулей
mod drivers;
mod x86;
mod shell;
mod commands;
mod utils;

// Подключение функций
use core::arch::{asm, naked_asm}; 
use core::panic::PanicInfo;
use drivers::{vga, keyboard, pit};
use x86::{gdt, idt, memory};
// use core::ptr::read_unaligned;

/// Структура PCINFO (см. Загрузчик)
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct PCINFO {
    low_memory_amount: u16,
    disk_num: u8,
    memory_map: memory::E820Map,
}

/// Настройка окружения ядра
/// (Получение адреса блока информации о ПК)
#[link_section = ".text.entry"]
#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    // ! Полагаемся на настроенный стек из загрузчика
    naked_asm!("push ebx", "call kmain");
}

/// Основный цикл работы ядра
#[no_mangle]
extern "C" fn kmain(_pcinfo_addr: *mut PCINFO) -> ! {
    // Инициализация модулей
    idt::interrupts_disable();
    gdt::init();
    idt::init();
    pit::init(100);
    idt::interrupts_enable();

    // Вывод лого системы c ожиданием нажатия
    vga::clear_screen();
    draw_logo(4, 2);
    for _ in 0..11 { vga::new_line(); }

    vga::print_line("   Press any key to continue...", vga::Color::LightGray);
    
    keyboard::read_key();
    
    // Вывод заголовка Shell с его бесконечной работой
    vga::clear_screen();
    vga::print_line(
        "Welcome to Realix (Protected Mode with Rust kernel)...\n",
        vga::Color::Cyan
    );

    shell::run();
    halt_loop();
}

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
    
    for (lt_i, letter) in letters.iter().enumerate() {
        let color = colors[lt_i % colors.len()];
        let offset_x = start_x + lt_i * letter[0].len();
        
        for (row, line) in letter.iter().enumerate() {
            for (col, &pixel) in line.iter().enumerate() {
                if pixel == 1 {
                    vga::write_char_at(start_y + row, offset_x + col, 0xDB, color);
                }
            }
        }
    }
}

/// Бесконечная остановка процессора с выкл. прерываниями
fn halt_loop() -> ! {
    idt::interrupts_disable();
    loop { unsafe { asm!("hlt"); } }
}

/// Обработчик ошибок
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_loop()
}

/// Функция записи байта в порт (Встраивается в бинарник)
#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al", in("dx") port,
        in("al") value, options(nostack, nomem, preserves_flags)
    );
}

/// Функция чтения байт из порта (Встраивается в бинарник)
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!(
        "in al, dx", out("al") value,
        in("dx") port, options(nostack, nomem, preserves_flags)
    );
    value
}
