// © Realix > Kernel32: Main
// (25.06.26) v0.07
// ================
#![no_std]
#![no_main]

// Импорт модулей
use core::panic::PanicInfo;

mod drivers;
mod x86;
mod shell;

// > Настройка окружения ядра
#[link_section = ".text.entry"]
#[no_mangle]
pub fn _start() -> ! {
    x86::gdt::init();
    x86::idt::init();

    drivers::vga::clear_screen();
    drivers::vga::print_str("Welcome, Realix (Protected Mode with Rust kernel)...\n", drivers::vga::Color::Cyan);

    shell::run();
    halt_loop();
}

// > Бесконечная остановка процессора
fn halt_loop() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

// > Обработчик ошибок
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_loop()
}
