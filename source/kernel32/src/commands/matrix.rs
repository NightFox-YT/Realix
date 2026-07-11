// © Realix > Matrix command
// Исправленная версия
// ===================

use crate::drivers::keyboard;
use crate::drivers::pit;
use crate::drivers::vga::{self, VGA_HEIGHT, VGA_WIDTH};


/// Запуск анимации Matrix.
pub fn run() {
    let mut drops_heights: [usize; VGA_WIDTH] = [0; VGA_WIDTH];

    for column in 0..VGA_WIDTH {
        drops_heights[column] =
            (column.wrapping_mul(7).wrapping_add(3)) % VGA_HEIGHT;
    }

    pit::sleep(600);
    vga::clear_screen();

    loop {
        for column in 0..VGA_WIDTH {
            let row = drops_heights[column];

            if row < VGA_HEIGHT {
                vga::write_char_at(
                    row,
                    column,
                    random_symbol(column, row),
                    vga::Color::White,
                );
            }

            if row > 0 && row - 1 < VGA_HEIGHT {
                vga::write_char_at(
                    row - 1,
                    column,
                    random_symbol(column, row - 1),
                    vga::Color::Green,
                );
            }

            if row >= 2 && row - 2 < VGA_HEIGHT {
                vga::write_char_at(
                    row - 2,
                    column,
                    random_symbol(column, row - 2),
                    vga::Color::DarkGray,
                );
            }

            if row >= 3 && row - 3 < VGA_HEIGHT {
                vga::write_char_at(
                    row - 3,
                    column,
                    b' ',
                    vga::Color::Black,
                );
            }

            drops_heights[column] += 1;

            if drops_heights[column] >= VGA_HEIGHT + 3 {
                for clear_row in 0..VGA_HEIGHT {
                    vga::write_char_at(
                        clear_row,
                        column,
                        b' ',
                        vga::Color::Black,
                    );
                }

                drops_heights[column] = 0;
            }
        }

        pit::sleep(40);

        /*
         * Выход по любой нажатой клавише, а не только по клавише,
         * имеющей ASCII-представление.
         */
        if let Some(scancode) = keyboard::queue_pop() {
            if scancode & 0x80 == 0 {
                break;
            }
        } else {
            unsafe {
                core::arch::asm!(
                    "sti",
                    "hlt",
                    options(nostack, nomem),
                );
            }
        }
    }

    vga::clear_screen();
}


/// Детерминированный псевдослучайный символ.
fn random_symbol(column: usize, row: usize) -> u8 {
    const CHARACTERS: &[u8; 42] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890@#$%&*";

    let index = column
        .wrapping_mul(17)
        .wrapping_add(row.wrapping_mul(31));

    CHARACTERS[index % CHARACTERS.len()]
}
