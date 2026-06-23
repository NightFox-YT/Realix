#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga;
mod keyboard;
mod shell;

#[link_section = ".text.entry"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga::clear_screen();
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
