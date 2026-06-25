#![no_std]
#![no_main]

use core::panic::PanicInfo;
#[path = "drivers/vga.rs"]
mod vga;
mod keyboard;
mod shell;
mod snake;
mod fetch;
mod matrix;
mod calc;
mod pci;
mod rtl8139;
mod arp;
mod rtc;

#[link_section = ".text.entry"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga::clear_screen();
    
    vga::print_str("Realix v0.07\n", vga::Color::Cyan);
    
    if rtl8139::init() {
        vga::print_str("[net] RTL8139 OK\n", vga::Color::Green);
    }
    
    shell::run();
    
    loop { unsafe { core::arch::asm!("hlt"); } }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { unsafe { core::arch::asm!("hlt"); } }
}

