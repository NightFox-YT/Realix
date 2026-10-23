// © Realix > Command: Gfx
// (09.09.26) v0.1
// ================
// ❗️ Зависимости: x86::realmode_video (переход в Real Mode для смены
//    видеорежима), drivers::vga (пиксельная отрисовка, уже существовала)

// Подключение функций
use crate::config::{TEXT_MODE_80x25, VIDEO_MODE_320x200};
use crate::drivers::keyboard;
use crate::drivers::vga::{self, Color};
use crate::x86::realmode_video;

/// Выполняет gfx: переключает VGA в 320x200x256, рисует тестовый узор
/// (проверка заливки, отдельных пикселей и залитого прямоугольника),
/// ждёт нажатия клавиши и возвращает текстовый режим 80x25
pub fn run() {
    vga::print_line(
        "Switching to 320x200 graphics mode... (press any key to return)\n",
        Color::LightGray,
    );

    realmode_video::set_mode(VIDEO_MODE_320x200 as u8);
    draw_test_pattern();

    keyboard::read_key();

    realmode_video::set_mode(TEXT_MODE_80x25 as u8);
    vga::init(0);
    vga::clear_screen();
    vga::print_line("Back in text mode.\n", Color::LightGray);
}

/// Рисует тестовый узор: 16 цветных полос (заливка), диагональ (проверка
/// точной адресации отдельных пикселей) и залитый прямоугольник в центре
fn draw_test_pattern() {
    vga::fill_screen(Color::Black);

    let colors = [
        Color::Black, Color::Blue, Color::Green, Color::Cyan,
        Color::Red, Color::Magenta, Color::Brown, Color::LightGray,
        Color::DarkGray, Color::LightBlue, Color::LightGreen, Color::LightCyan,
        Color::LightRed, Color::Pink, Color::Yellow, Color::White,
    ];

    let stripe_height = vga::VGA_VIDEO_HEIGHT / colors.len();
    for (i, &color) in colors.iter().enumerate() {
        let y1 = i * stripe_height;
        let y2 = if i == colors.len() - 1 {
            vga::VGA_VIDEO_HEIGHT - 1
        } else {
            y1 + stripe_height - 1
        };
        vga::draw_rect(0, vga::VGA_VIDEO_WIDTH - 1, y1, y2, color);
    }

    // Диагональ поверх полос - проверка адресации отдельных пикселей
    // (округление/сдвиг в set_pixel/fill_screen проявился бы как перекос)
    for x in 0..vga::VGA_VIDEO_WIDTH {
        let y = x * vga::VGA_VIDEO_HEIGHT / vga::VGA_VIDEO_WIDTH;
        vga::set_pixel(x, y, Color::White);
    }

    // Залитый прямоугольник в центре - проверка draw_rect на не-крайних координатах
    let box_w = 60;
    let box_h = 40;
    let box_x = (vga::VGA_VIDEO_WIDTH - box_w) / 2;
    let box_y = (vga::VGA_VIDEO_HEIGHT - box_h) / 2;
    vga::draw_rect(box_x, box_x + box_w, box_y, box_y + box_h, Color::White);
}
