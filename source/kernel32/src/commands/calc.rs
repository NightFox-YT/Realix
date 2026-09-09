// © Realix > Command: Calculator
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Выполняет calc <num1> <+ - * /> <num2> (операнды - неотрицательные целые)
/// Параметры:
///  - args: аргументы после имени команды
pub fn run(args: &str) {
    let mut parts = args.split_whitespace();
    let (a, op, b, extra) = (parts.next(), parts.next(), parts.next(), parts.next());

    let (a, op, b) = match (a, op, b, extra) {
        (Some(a), Some(op), Some(b), None) => (a, op, b),
        _ => {
            vga::print_line("[?] Usage: calc <num1> <+ - * /> <num2> (or , . ' without Shift)\n", Color::LightGray);
            return;
        }
    };

    let a = match parse_u32(a) {
        Some(v) => v as i64,
        None => {
            vga::print_line("[!] Operands must be non-negative integers.\n", Color::Red);
            return;
        }
    };
    let b = match parse_u32(b) {
        Some(v) => v as i64,
        None => {
            vga::print_line("[!] Operands must be non-negative integers.\n", Color::Red);
            return;
        }
    };

    // Алиасы операторов без Shift: , -> + . -> - ' -> *
    let op = match op {
        "," => "+",
        "." => "-",
        "'" => "*",
        other => other,
    };

    let result: Option<i64> = match op {
        "+" => a.checked_add(b),
        "-" => a.checked_sub(b),
        "*" => a.checked_mul(b),
        "/" => {
            if b == 0 {
                vga::print_line("[!] Division by zero!\n", Color::Red);
                return;
            }
            Some(a / b)
        }
        _ => {
            vga::print_line("[!] Unknown operator, use + - * / (or , . ' without Shift)\n", Color::Red);
            return;
        }
    };

    let result = match result {
        Some(v) if (-(u32::MAX as i64)..=u32::MAX as i64).contains(&v) => v,
        _ => {
            vga::print_line("[!] Result too large (overflow)\n", Color::Red);
            return;
        }
    };

    vga::print_line("Result: ", Color::LightGray);

    let mut buf: [u8; 10] = [0u8; 10];
    if result < 0 {
        vga::print_line("-", Color::White);
        vga::print_line(utils::u32_to_dec_str((-result) as u32, &mut buf), Color::White);
    } else {
        vga::print_line(utils::u32_to_dec_str(result as u32, &mut buf), Color::White);
    }
    vga::print_new_line();
}

/// Разбор беззнакового десятичного числа
fn parse_u32(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }

    let mut value: u32 = 0;
    for byte in s.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add((byte - b'0') as u32)?;
    }
    Some(value)
}
