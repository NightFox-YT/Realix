// © Realix > Matrix command
// (04.07.26) v0.08
// ø Copyright by @liquifield
// ================

// Подключение функций
use crate::drivers::keyboard;
use crate::drivers::pit;
use crate::drivers::vga::{self, VGA_HEIGHT, VGA_WIDTH};

pub fn run() {
    let mut drops_heights: [usize; VGA_WIDTH] = [0; 80];
    
    // Инициализация (Капли на разной высоте)
    for col in 0..VGA_WIDTH {
        drops_heights[col] = (col * 7 + 3) % vga::VGA_HEIGHT;
    }

    pit::sleep(600);
    vga::clear_screen();
    
    loop {
        // Отрисовка одного кадра
        for col in 0..VGA_WIDTH {
            let y = drops_heights[col];
            
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
            
            // Стираем символы выше хвоста
            if y >= 3 {
                vga::write_char_at(y - 3, col, b' ', vga::Color::Black);
            }
            
            // Двигаем каплю вниз
            drops_heights[col] += 1;
            
            // Если достигла дна — сбрасываем наверх с стиранием столбца
            if drops_heights[col] >= VGA_HEIGHT + 1 {
                for row in 0..VGA_HEIGHT {
                    vga::write_char_at(row, col, b' ', vga::Color::Black);
                }
                drops_heights[col] = 0;
            }
        }
        
        // Задержка для плавности
        pit::sleep(40);
        
        // Выход по нажатой клавише
        if let Some(scancode) = keyboard::queue_pop() {
            // Игнорируем отпускание клавиш (бит 7 = 1)
            if scancode & 0x80 == 0 {
                if let Some(_ascii) = keyboard::scancode_to_ascii(scancode) {
                    break
                }
            }
        } else {
            // Очередь пуста
            unsafe { core::arch::asm!("sti; hlt"); }
        }
    }
    
    vga::clear_screen();
}

/// Псевдо-генерация случайного знака для позиции `row`, `col`
fn random_sym(col: usize, row: usize) -> u8 {
    let chars: &[u8; 42] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890@#$%&*";

    // Псевдо-случайное число (Коэффициенты случайны)
    let n: usize = col.wrapping_mul(17).wrapping_add(row.wrapping_mul(31));
    chars[n % chars.len()]
}