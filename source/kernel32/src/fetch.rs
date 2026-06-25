use crate::vga;

pub fn run() {
    vga::clear_screen();
    
    let start_y = 3;
    let start_x = 3;
    
    // Буква R: узнаваемая форма с ножкой
    let r_logo: [[(u8, vga::Color); 8]; 7] = [
        // 1: ██████  
        [(0xDB, vga::Color::LightCyan), (0xDB, vga::Color::LightCyan), (0xDB, vga::Color::LightCyan), (0xDB, vga::Color::LightCyan), (0xDB, vga::Color::LightCyan), (0xDB, vga::Color::LightCyan), (0x20, vga::Color::Black), (0x20, vga::Color::Black)],
        // 2: █    █ 
        [(0xDB, vga::Color::Blue), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0xDB, vga::Color::LightCyan), (0x20, vga::Color::Black), (0x20, vga::Color::Black)],
        // 3: █    █ 
        [(0xDB, vga::Color::Blue), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0xDB, vga::Color::LightCyan), (0x20, vga::Color::Black), (0x20, vga::Color::Black)],
        // 4: █████  
        [(0xDB, vga::Color::Blue), (0xDB, vga::Color::Blue), (0xDB, vga::Color::Blue), (0xDB, vga::Color::Blue), (0xDB, vga::Color::Blue), (0xDB, vga::Color::Cyan), (0x20, vga::Color::Black), (0x20, vga::Color::Black)],
        // 5: █   █  
        [(0xDB, vga::Color::Blue), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0xDB, vga::Color::Pink), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black)],
        // 6: █    █ 
        [(0xDB, vga::Color::Blue), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0xDB, vga::Color::Pink), (0x20, vga::Color::Black), (0x20, vga::Color::Black)],
        // 7: █     █
        [(0xDB, vga::Color::Blue), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0x20, vga::Color::Black), (0xDB, vga::Color::Pink), (0x20, vga::Color::Black)],
    ];
    
    for (i, row) in r_logo.iter().enumerate() {
        for (j, &(ch, color)) in row.iter().enumerate() {
            vga::write_char_at(start_y + i, start_x + j, ch, color);
        }
    }
    
    let info: [(&str, &str); 7] = [
        ("OS", "Realix v0.07"),
        ("Kernel", "Rust 32-bit"),
        ("Shell", "Realix Shell"),
        ("Memory", "640 KB"),
        ("CPU", "x86 i686+"),
        ("Display", "VGA 80x25"),
        ("Apps", "snake, matrix, calc"),
    ];
    
    for (i, (label, value)) in info.iter().enumerate() {
        let y = start_y + i;
        let x = 22;
        
        for (j, c) in label.bytes().enumerate() {
            vga::write_char_at(y, x + j, c, vga::Color::Green);
        }
        vga::write_char_at(y, x + label.len(), b':', vga::Color::DarkGray);
        for (j, c) in value.bytes().enumerate() {
            vga::write_char_at(y, x + label.len() + 2 + j, c, vga::Color::White);
        }
    }
    
    for x in 0..80 {
        vga::write_char_at(start_y + 9, x, b'=', vga::Color::DarkGray);
    }
    
    let hint = "Press any key to return";
    for (i, c) in hint.bytes().enumerate() {
        vga::write_char_at(23, (80 - hint.len()) / 2 + i, c, vga::Color::DarkGray);
    }
    
    wait_key();
    vga::clear_screen();
}

fn wait_key() {
    loop {
        unsafe {
            let s: u8;
            core::arch::asm!("in al, 0x64", out("al") s);
            if s & 1 != 0 && s & 0x20 == 0 {
                let sc: u8;
                core::arch::asm!("in al, 0x60", out("al") sc);
                if sc & 0x80 == 0 {
                    return;
                }
            }
        }
    }
}
