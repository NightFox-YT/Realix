// © Realix > Command: Fibonacci
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Выполняет fib <n>: n-ное число Фибоначчи (fib(0) = 0, fib(1) = 1)
/// Параметры:
///  - args: аргументы после имени команды
pub fn run(args: &str) {
    let mut parts = args.split_whitespace();
    let arg = parts.next();
    let extra = parts.next();

    let n = match (arg, extra) {
        (Some(arg), None) => utils::parse_u32(arg),
        _ => None,
    };

    let n = match n {
        Some(v) => v,
        None => {
            vga::print_line("[?] Usage: fib <n>\n", Color::LightGray);
            return;
        }
    };

    let mut a: u32 = 0;
    let mut b: u32 = 1;

    for _ in 0..n {
        let next = match a.checked_add(b) {
            Some(v) => v,
            None => {
                vga::print_line("[!] Fib argument too large (overflow)\n", Color::Red);
                return;
            }
        };
        a = b;
        b = next;
    }

    let mut buf: [u8; 10] = [0u8; 10];
    vga::print_line("Fib: ", Color::LightGray);
    vga::print_line(utils::u32_to_dec_str(a, &mut buf), Color::White);
    vga::new_line();
}
