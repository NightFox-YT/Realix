// © Realix > Command: VGA demo
// (07.09.26) v0.12
// ================
// ❗️ В отличие от 16-bit версии (реальный графический режим 320x200/256 цветов через
//    BIOS int 0x10), здесь рисуем те же 3 вложенных прямоугольника блоками в текстовом
//    режиме: переключение VGA в режим 13h без BIOS требует ручной инициализации
//    аппаратных регистров (Sequencer/CRTC/Graphics/Attribute controller), что вынесено
//    за рамки этого прохода как отдельная, непроверенная в железе задача.

// Подключение функций
use crate::drivers::keyboard;
use crate::drivers::vga::{self, Color};

// Символ сплошного блока (используется и в логотипе загрузки)
const BLOCK_CHAR: u8 = 0xDB;

/// Команда демонстрации: 3 вложенных цветных прямоугольника
pub fn run() {
    vga::print_line("[+] Starting VGA demo...\n", Color::LightGray);
    vga::print_line(
        "Press any key to start... (After that you can press any key to exit)\n",
        Color::LightGray,
    );
    keyboard::read_key();

    fill_screen(Color::Blue);
    draw_rect(2, 4, 77, 20, Color::Red);
    draw_rect(6, 7, 73, 17, Color::Yellow);
    draw_rect(12, 10, 67, 14, Color::White);

    keyboard::read_key();
    vga::clear_screen();
}

/// Заливка всего экрана сплошным цветом
fn fill_screen(color: Color) {
    for row in 0..vga::VGA_HEIGHT {
        for col in 0..vga::VGA_WIDTH {
            vga::write_char_at(row, col, BLOCK_CHAR, color);
        }
    }
}

/// Рисование прямоугольника-рамки [x0, y0] - [x1, y1] (включительно)
fn draw_rect(x0: usize, y0: usize, x1: usize, y1: usize, color: Color) {
    for col in x0..=x1 {
        vga::write_char_at(y0, col, BLOCK_CHAR, color);
        vga::write_char_at(y1, col, BLOCK_CHAR, color);
    }
    for row in y0..=y1 {
        vga::write_char_at(row, x0, BLOCK_CHAR, color);
        vga::write_char_at(row, x1, BLOCK_CHAR, color);
    }
}
