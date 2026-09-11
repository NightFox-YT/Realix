// © Realix > Commands: Numbers
// (07.09.26) v0.12
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};
use crate::utils;

// Предел индекса Фибоначчи (Результат должен влезть в u32 без переполнения)
// fib(46) = 1836311903, fib(47) = 2971215073 (переполнение u32)
const FIB_MAX_INDEX: u32 = 46;

/// Команда вывода числа в шестнадцатеричном виде: hex <num>
/// Параметры:
///  - args: аргументы команды (после имени)
pub fn hex(args: &str) {
    let Ok(value) = args.trim().parse::<u32>() else {
        vga::print_line("[?] Usage: hex <num>\n", Color::LightGray);
        return;
    };

    let mut buf: [u8; 10] = [0u8; 10];
    vga::print_line("Hex: ", Color::LightGray);
    vga::print_line(utils::u32_to_hex_str(value, &mut buf), Color::White);
    vga::new_line();
}

/// Команда вычисления числа Фибоначчи: fib <0-46>
/// Параметры:
///  - args: аргументы команды (после имени)
pub fn fib(args: &str) {
    let Ok(index) = args.trim().parse::<u32>() else {
        vga::print_line("[?] Usage: fib <0-46>\n", Color::LightGray);
        return;
    };

    if index > FIB_MAX_INDEX {
        vga::print_line("[!] Fib argument must be 0..46.\n", Color::Red);
        return;
    }

    let (mut a, mut b): (u32, u32) = (0, 1);
    for _ in 0..index {
        let next: u32 = a + b;
        a = b;
        b = next;
    }

    let mut buf: [u8; 10] = [0u8; 10];
    vga::print_line("Fib: ", Color::LightGray);
    vga::print_line(utils::u32_to_dec_str(a, &mut buf), Color::White);
    vga::new_line();
}
