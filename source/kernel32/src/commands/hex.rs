// © Realix > Command: Hex
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Выполняет hex <num>: печатает десятичное число в шестнадцатеричном виде
/// Параметры:
///  - args: аргументы после имени команды
pub fn run(args: &str) {
    let mut parts = args.split_whitespace();
    let arg = parts.next();
    let extra = parts.next();

    let value = match (arg, extra) {
        (Some(arg), None) => utils::parse_u32(arg),
        _ => None,
    };

    let value = match value {
        Some(v) => v,
        None => {
            vga::print_line("[?] Usage: hex <num>\n", Color::LightGray);
            return;
        }
    };

    let mut buf: [u8; 10] = [0u8; 10];
    vga::print_line("Hex: ", Color::LightGray);
    vga::print_line(utils::u32_to_hex_str(value, &mut buf), Color::White);
    vga::new_line();
}
