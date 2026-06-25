// © Realix > Kernel32: Main
// (25.06.26) v0.07
// ================

// Настройка компиляции (Без стандартной библиотеки и обёртки main())
#![no_std]
#![no_main]

// Импорт модулей
use core::panic::PanicInfo;
mod drivers;
mod gdt;

// > Настройка окружения ядра (Не изменять название функции при компиляции)
#[link_section = ".text.entry"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    gdt::init();

    drivers::vga::clear_screen();
    drivers::vga::print_str(0, 0, "Welcome, Realix v0.07 with Rust kernel...", drivers::vga::Color::Cyan);
    halt_loop();
}

// > Бесконечная остановка процессора
fn halt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// > Обработчик ошибок
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_loop()
}
