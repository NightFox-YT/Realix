// © Realix > Command: Matrix
// ø @liquifield
// (27.07.26) v0.1
// ================

// Подключение функций
use crate::drivers::keyboard;
use crate::drivers::pit;
use crate::drivers::vga::{self, VGA_TEXT_HEIGHT, VGA_TEXT_WIDTH};

/// Запуск анимации Matrix
pub fn run() {
    let mut drops_heights: [usize; VGA_TEXT_WIDTH] = [0; VGA_TEXT_WIDTH];

    // Инициализация (Капли на разной высоте)
    for (col, height) in drops_heights.iter_mut().enumerate() {
        *height = (col.wrapping_mul(7).wrapping_add(3)) % VGA_TEXT_HEIGHT;
    }

    pit::sleep(400);
    vga::clear_screen();

    loop {
        // Отрисовка одного кадра
        for (col, height) in drops_heights.iter_mut().enumerate() {
            let row: usize = *height;

            // Белая голова
            if row < VGA_TEXT_HEIGHT {
                vga::write_char_at(
                    row, col,
                    random_symbol(col, row),
                    vga::Color::White,
                );
            }

            // Зелёный шлейф выше
            if row > 0 && row - 1 < VGA_TEXT_HEIGHT {
                vga::write_char_at(
                    row - 1, col,
                    random_symbol(col, row - 1),
                    vga::Color::Green,
                );
            }

            // Тёмный хвост ещё выше
            if row >= 2 && row - 2 < VGA_TEXT_HEIGHT {
                vga::write_char_at(
                    row - 2, col,
                    random_symbol(col, row - 2),
                    vga::Color::DarkGray,
                );
            }

            // Стираем символы выше хвоста
            if row >= 3 && row - 3 < VGA_TEXT_HEIGHT {
                vga::write_char_at(
                    row - 3, col,
                    b' ',
                    vga::Color::Black,
                );
            }

            // Двигаем каплю вниз
            *height += 1;

            // Если достигла дна - сбрасываем наверх со стиранием столбца
            if *height > VGA_TEXT_HEIGHT {
                for clear_row in 0..VGA_TEXT_HEIGHT {
                    vga::write_char_at(
                        clear_row, col,
                        b' ',
                        vga::Color::Black,
                    );
                }
                *height = 0;
            }
        }

        // Задержка для плавности
        pit::sleep(40);

        // Выход по любой нажатой клавише (Отпускания игнорируем)
        if let Some(scancode) = keyboard::queue_pop() {
            if scancode & keyboard::SCANCODE_RELEASE == 0 {
                break;
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
