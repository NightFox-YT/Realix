// © Realix > Command: Banner
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::vga::{self, Color, VGA_WIDTH};

// Отступ текста от рамки с каждой стороны и символ рамки
const PADDING: usize = 2;
const BORDER_CHAR: u8 = b'*';

/// Печатает переданный текст в рамке по центру строки
/// Параметры:
///  - text: текст для показа (без аргумента — печатается подсказка по использованию)
pub fn run(text: &str) {
    let text: &str = text.trim();

    if text.is_empty() {
        vga::print_line("[!] Usage: banner <text>\n", Color::Red);
        return;
    }

    // Ограничиваем длину текста, чтобы рамка влезла в ширину экрана
    let max_text_len: usize = VGA_WIDTH - (PADDING + 1) * 2;
    let text: &str = &text[..text.len().min(max_text_len)];

    let inner_width: usize = text.len() + PADDING * 2;
    let border_width: usize = inner_width + 2;

    print_border(border_width);
    print_text_row(text, inner_width);
    print_border(border_width);
}

/// Печатает верхнюю/нижнюю границу рамки
fn print_border(width: usize) {
    for _ in 0..width {
        vga::print_char(BORDER_CHAR, Color::Yellow);
    }
    vga::new_line();
}

/// Печатает среднюю строку рамки с текстом по центру
fn print_text_row(text: &str, inner_width: usize) {
    let left_pad: usize = (inner_width - text.len()) / 2;
    let right_pad: usize = inner_width - text.len() - left_pad;

    vga::print_char(BORDER_CHAR, Color::Yellow);
    for _ in 0..left_pad {
        vga::print_char(b' ', Color::Black);
    }
    vga::print_line(text, Color::White);
    for _ in 0..right_pad {
        vga::print_char(b' ', Color::Black);
    }
    vga::print_char(BORDER_CHAR, Color::Yellow);
    vga::new_line();
}
