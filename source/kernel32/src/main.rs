// © Realix > Kernel32: Main
// (03.07.26) v0.08
// ================
#![no_std]
#![no_main]

// Объявление модулей
mod drivers;
mod x86;
mod shell;
mod utils;

// Подключение функций
use core::panic::PanicInfo;
use drivers::{vga, pit};
use x86::{gdt, idt};

/// Настройка окружения ядра
#[link_section = ".text.entry"]
#[no_mangle]
pub fn _start() -> ! {
    // > Инициализация модулей
    idt::interrupts_disable();

    gdt::init();
    idt::init();
    pit::init(100);

    idt::interrupts_enable();

    // Вывод приветственного сообщения
    vga::clear_screen();
    vga::print_line(
        "Welcome, Realix (Protected Mode with Rust kernel)...\n",
        vga::Color::Cyan
    );

    shell::run();
    halt_loop();
}

/// Бесконечная остановка процессора с выкл. прерываниями
fn halt_loop() -> ! {
    idt::interrupts_disable();
    loop { unsafe { core::arch::asm!("hlt"); } }
}

/// Обработчик ошибок
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_loop()
}

/// Функция записи байта в порт (Встраивается в бинарник)
#[inline(always)]
pub fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al", in("dx") port,
            in("al") value, options(nostack, nomem, preserves_flags)
        );
    }
}

/// Функция чтения байт из порта (Встраивается в бинарник)
#[inline(always)]
pub fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx", out("al") value,
            in("dx") port, options(nostack, nomem, preserves_flags)
        );
    }
    value
}
