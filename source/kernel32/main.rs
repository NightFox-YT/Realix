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
mod idt;
mod interrupts;
mod arp;

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
    // Инициализация IDT и PIC
    idt::init();
    idt::set_handler(0x00, interrupts::divide_by_zero_handler, 0x8E);
    idt::set_handler(0x06, interrupts::invalid_opcode_handler, 0x8E);
    idt::set_handler(0x08, interrupts::double_fault_handler, 0x8E);
    idt::set_handler(0x0D, interrupts::general_protection_fault_handler, 0x8E);
    for i in 0..255 {
        if i != 0x00 && i != 0x06 && i != 0x08 && i != 0x0D {
            idt::set_handler(i, interrupts::default_handler, 0x8E);
        }
    }
    interrupts::init();
    vga::print_str("[ok] IDT+PIC initialized\n", vga::Color::Green);
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
