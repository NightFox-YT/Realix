#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga;
mod keyboard;
mod shell;
mod snake;
mod fetch;
mod matrix;
mod calc;
mod pci;
mod rtl8139;

#[link_section = ".text.entry"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga::clear_screen();
    // Инициализация сети
    if rtl8139::init() {
        vga::print_str("[net] RTL8139 initialized\n", vga::Color::Green);
    } else {
        vga::print_str("[net] RTL8139 not found\n", vga::Color::DarkGray);
    }
    vga::print_str("Welcome, Realix v0.07 with Rust kernel!\n", vga::Color::Cyan);
    shell::run();
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
