// © Realix > Commands: Text
// (07.09.26) v0.12
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};
use crate::utils;

// Предел кода символа для команды `ascii`
const ASCII_MAX_CODE: u32 = 255;

// Предел числа повторов для команды `repeat`
const REPEAT_MAX_COUNT: u32 = 20;

/// Команда печати длины аргумента: len <t>
pub fn len(args: &str) {
    let text: &str = args.trim_start();
    let mut buf: [u8; 10] = [0u8; 10];

    vga::print_line("Length: ", Color::LightGray);
    vga::print_line(utils::u32_to_dec_str(text.len() as u32, &mut buf), Color::White);
    vga::new_line();
}

/// Команда печати аргумента в верхнем регистре: upper <t>
pub fn upper(args: &str) {
    for byte in args.trim_start().bytes() {
        vga::print_char(byte.to_ascii_uppercase(), Color::LightGray);
    }
    vga::new_line();
}

/// Команда печати аргумента в нижнем регистре: lower <t>
pub fn lower(args: &str) {
    for byte in args.trim_start().bytes() {
        vga::print_char(byte.to_ascii_lowercase(), Color::LightGray);
    }
    vga::new_line();
}

/// Команда печати аргумента задом наперёд: reverse <t>
pub fn reverse(args: &str) {
    for byte in args.trim_start().bytes().rev() {
        vga::print_char(byte, Color::LightGray);
    }
    vga::new_line();
}

/// Команда вывода символа по десятичному коду: ascii <0-255>
pub fn ascii(args: &str) {
    let Ok(code) = args.trim().parse::<u32>() else {
        vga::print_line("[?] Usage: ascii <0-255>\n", Color::LightGray);
        return;
    };

    if code > ASCII_MAX_CODE {
        vga::print_line("[?] Usage: ascii <0-255>\n", Color::LightGray);
        return;
    }

    vga::print_line("Char: ", Color::LightGray);
    vga::print_char(code as u8, Color::LightGray);
    vga::new_line();
}

/// Команда повтора текста `N` раз: repeat <1-20> <text>
/// Параметры:
///  - args: аргументы команды (после имени) - "<count> <text>"
pub fn repeat(args: &str) {
    let args: &str = args.trim_start();
    let (count_str, text) = match args.find(' ') {
        Some(idx) => (&args[..idx], args[idx..].trim_start()),
        None => (args, ""),
    };

    let Ok(count) = count_str.parse::<u32>() else {
        vga::print_line("[?] Usage: repeat <1-20> <text>\n", Color::LightGray);
        return;
    };

    // Ноль повторов - тихо выходим
    if count == 0 {
        return;
    }

    if count > REPEAT_MAX_COUNT {
        vga::print_line("[!] Repeat count must be 1..20.\n", Color::Red);
        return;
    }

    if text.is_empty() {
        vga::print_line("[?] Usage: repeat <1-20> <text>\n", Color::LightGray);
        return;
    }

    for _ in 0..count {
        vga::print_line(text, Color::LightGray);
    }
}
