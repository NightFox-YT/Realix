// © Realix > Command: Calculator
// (07.09.26) v0.12
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Простой калькулятор: calc <a> <+ - * /> <b>
/// Параметры:
///  - args: аргументы команды (после имени, может начинаться с пробела)
pub fn run(args: &str) {
    let mut parts = args.split_whitespace();
    let (Some(a_str), Some(op), Some(b_str), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        usage();
        return;
    };

    let (Ok(a), Ok(b)) = (a_str.parse::<u16>(), b_str.parse::<u16>()) else {
        usage();
        return;
    };

    let mut buf: [u8; 10] = [0u8; 10];

    match op {
        "+" => match a.checked_add(b) {
            Some(result) => print_result(result, &mut buf),
            None => print_error("Result too large (overflow)"),
        },
        "-" => {
            // Вычитание без переполнения: считаем как знаковое, выводим "-" при отрицательном
            let diff: i32 = a as i32 - b as i32;
            if diff < 0 {
                vga::print_line("Result: -", Color::LightGray);
                vga::print_line(utils::u32_to_dec_str((-diff) as u32, &mut buf), Color::White);
                vga::new_line();
            } else {
                print_result(diff as u16, &mut buf);
            }
        }
        "*" => match a.checked_mul(b) {
            Some(result) => print_result(result, &mut buf),
            None => print_error("Result too large (overflow)"),
        },
        "/" => {
            if b == 0 {
                print_error("Division by zero!");
            } else {
                print_result(a / b, &mut buf);
            }
        }
        _ => print_error("Unknown operator, use + - * /"),
    }
}

/// Вывод строки "Result: <число>"
fn print_result(value: u16, buf: &mut [u8; 10]) {
    vga::print_line("Result: ", Color::LightGray);
    vga::print_line(utils::u32_to_dec_str(value as u32, buf), Color::White);
    vga::new_line();
}

/// Вывод сообщения об ошибке
fn print_error(msg: &str) {
    vga::print_line("[!] ", Color::Red);
    vga::print_line(msg, Color::Red);
    vga::new_line();
}

/// Вывод подсказки по использованию команды
fn usage() {
    vga::print_line("[?] Usage: calc <num1> <+ - * /> <num2>\n", Color::LightGray);
}
