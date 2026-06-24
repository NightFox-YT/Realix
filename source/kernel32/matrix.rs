use crate::vga;

pub fn run() {
    vga::clear_screen();
    
    let mut drops: [usize; 80] = [0; 80];
    
    // Инициализация: капли на разной высоте
    for col in 0..80 {
        drops[col] = (col * 7 + 3) % 25;
    }
    
    loop {
        // Отрисовка одного кадра
        for col in 0..80 {
            let y = drops[col];
            
            // Белая голова
            vga::write_char_at(y, col, random_sym(col, y), vga::Color::White);
            
            // Зелёный шлейф выше
            if y > 0 {
                vga::write_char_at(y - 1, col, random_sym(col, y - 1), vga::Color::Green);
            }
            
            // Тёмный хвост ещё выше
            if y >= 2 {
                vga::write_char_at(y - 2, col, random_sym(col, y - 2), vga::Color::DarkGray);
            }
            
            // Стираем всё что выше хвоста
            if y >= 3 {
                vga::write_char_at(y - 3, col, b' ', vga::Color::Black);
            }
            
            // Двигаем каплю вниз
            drops[col] += 1;
            
            // Если достигла дна — сбрасываем наверх
            if drops[col] >= 25 {
                for r in 0..25 {
                    vga::write_char_at(r, col, b' ', vga::Color::Black);
                }
                drops[col] = 0;
            }
        }
        
        // Задержка для плавности
        for _ in 0..20000000 {
            unsafe { core::arch::asm!("nop"); }
        }
        
        // Выход по любой клавише кроме Enter
        if let Some(_key) = check_key() {
            break;
        }
    }
    
    vga::clear_screen();
}

fn random_sym(col: usize, row: usize) -> u8 {
    let n = col.wrapping_mul(17).wrapping_add(row.wrapping_mul(31));
    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890@#$%&*";
    chars[n % chars.len()]
}

fn check_key() -> Option<u8> {
    unsafe {
        let s: u8;
        core::arch::asm!("in al, 0x64", out("al") s);
        if s & 1 != 0 && s & 0x20 == 0 {
            let sc: u8;
            core::arch::asm!("in al, 0x60", out("al") sc);
            if sc & 0x80 == 0 && sc != 0x1C {
                return Some(b'q');
            }
        }
        None
    }
}
