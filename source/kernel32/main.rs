// Настройка компиляции (Без стандартной библиотеки и обёртки main())
#![no_std]
#![no_main]

// Импорт модулей
use core::panic::PanicInfo;
mod vga;

#[link_section = ".text.entry"]
#[no_mangle] // Не изменять название функции при компиляции
pub extern "C" fn _start() -> ! {
    vga::clear_screen();
    vga::print_str(0, 0, "Welcome, Realix v0.07 with Rust kernel...", vga::Color::Cyan);
    halt_loop();
}

// Бесконечная остановка процессора
fn halt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// Обработчик ошибок
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_loop()
}
