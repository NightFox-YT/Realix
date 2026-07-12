// © Realix > Matrix command
// (04.07.26) v0.08
// ø Copyright by @liquifield
// ================

// Подключение функций
use crate::drivers::keyboard;
use crate::drivers::pit;
use crate::drivers::vga::{self, VGA_HEIGHT, VGA_WIDTH};


/// Запуск анимации Matrix
pub fn run() {
    let mut drops_heights: [usize; VGA_WIDTH] = [0; VGA_WIDTH];

    // Инициализация (Капли на разной высоте)
    for col in 0..VGA_WIDTH {
        drops_heights[col] =
            (col.wrapping_mul(7).wrapping_add(3)) % VGA_HEIGHT;
    }

    pit::sleep(500);
    vga::clear_screen();

    loop {
        // Отрисовка одного кадра
        for col in 0..VGA_WIDTH {
            let row = drops_heights[col];

            // Белая голова
            if row < VGA_HEIGHT {
                vga::write_char_at(
                    row, col,
                    random_symbol(col, row),
                    vga::Color::White,
                );
            }

            // Зелёный шлейф выше
            if row > 0 && row - 1 < VGA_HEIGHT {
                vga::write_char_at(
                    row - 1, col,
                    random_symbol(col, row - 1),
                    vga::Color::Green,
                );
            }

            // Тёмный хвост ещё выше
            if row >= 2 && row - 2 < VGA_HEIGHT {
                vga::write_char_at(
                    row - 2, col,
                    random_symbol(col, row - 2),
                    vga::Color::DarkGray,
                );
            }

            // Стираем символы выше хвоста
            if row >= 3 && row - 3 < VGA_HEIGHT {
                vga::write_char_at(
                    row - 3, col,
                    b' ',
                    vga::Color::Black,
                );
            }

            // Двигаем каплю вниз
            drops_heights[col] += 1;

            // Если достигла дна — сбрасываем наверх с стиранием столбца
            if drops_heights[col] >= VGA_HEIGHT + 1 {
                for clear_row in 0..VGA_HEIGHT {
                    vga::write_char_at(
                        clear_row, col,
                        b' ',
                        vga::Color::Black,
                    );
                }
                drops_heights[col] = 0;
            }
        }

        // Задержка для плавности
        pit::sleep(40);

        // Выход по любой нажатой клавише
        if let Some(scancode) = keyboard::queue_pop() {
            // Игнорируем отпускание клавиш (бит 7 = 1)
            if scancode & 0x80 == 0 {
                break;
            }
        } else {
            // Очередь пуста
            unsafe {
                core::arch::asm!(
                    "sti", "hlt",
                    options(nostack, nomem),
                );
            }
        }
    }

    vga::clear_screen();
}


/// Псевдо-генерация случайного символа
fn random_symbol(col: usize, row: usize) -> u8 {
    const CHARACTERS: &[u8; 42] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890@#$%&*";

    // Псевдо-случайное число (Коэффициенты случайны)
    let index = col.wrapping_mul(17).wrapping_add(row.wrapping_mul(31));
    CHARACTERS[index % CHARACTERS.len()]
}
