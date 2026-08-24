// © Realix > Command: Echo
// (22.07.26) v0.1
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};

/// Печатает переданный текст (Без аргумента — пустая строка)
/// Параметры:
///  - line_ptr: указатель на строку после имени команды (echo)
pub fn run(line_ptr: &str) {
    // Вывод со срезанным пробелом-разделитель после имени
    vga::print_line(line_ptr.trim_start(), Color::LightGray);
    vga::print_new_line();
}