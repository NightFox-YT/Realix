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
    
    draw_logo();
    
    for _ in 0..12 {
        vga::put_char(b'\n', vga::Color::Black);
    }
    
    vga::print_str("  Welcome to Realix v0.07 - Rust Kernel\n", vga::Color::Cyan);
    vga::print_str("  Press any key to enter shell...", vga::Color::LightGray);
    
    keyboard::read_key_blocking();
    
    vga::clear_screen();
    vga::print_str("Welcome to Realix shell!\n", vga::Color::Cyan);
    shell::run();
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

fn draw_logo() {
    let block: u8 = 0xDB;
    
    let colors = [
        vga::Color::Red,
        vga::Color::Yellow,
        vga::Color::Green,
        vga::Color::Cyan,
        vga::Color::Blue,
        vga::Color::Magenta,
    ];
    
    let letters: [[[u8; 6]; 8]; 6] = [
        // R
        [[1,1,1,1,0,0],[1,0,0,1,0,0],[1,0,0,1,0,0],[1,1,1,1,0,0],
         [1,0,0,1,0,0],[1,0,0,1,0,0],[1,0,0,1,0,0],[1,0,0,1,0,0]],
        // E
        [[1,1,1,1,1,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,1,1,1,0,0],
         [1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,1,1,1,1,0]],
        // A
        [[0,1,1,1,0,0],[1,0,0,0,1,0],[1,0,0,0,1,0],[1,1,1,1,1,0],
         [1,0,0,0,1,0],[1,0,0,0,1,0],[1,0,0,0,1,0],[1,0,0,0,1,0]],
        // L
        [[1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],
         [1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,1,1,1,1,0]],
        // I
        [[0,1,1,1,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],
         [0,0,1,0,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],[0,1,1,1,0,0]],
        // X
        [[1,0,0,0,1,0],[0,1,0,1,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],
         [0,0,1,0,0,0],[0,1,0,1,0,0],[1,0,0,0,1,0],[1,0,0,0,1,0]],
    ];
    
    let start_x = 4;
    let start_y = 2;
    
    for (li, letter) in letters.iter().enumerate() {
        let color = colors[li % colors.len()];
        let offset_x = start_x + li * 7;
        
        for (row, line) in letter.iter().enumerate() {
            for (col, &pixel) in line.iter().enumerate() {
                if pixel == 1 {
                    vga::write_char_at(start_y + row, offset_x + col, block, color);
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
