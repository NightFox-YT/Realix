// © Realix > Command: About
// (07.09.26) v0.12
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};
use crate::OS_VERSION;

/// Выводит краткую информацию о Realix
pub fn run() {
    vga::print_line("> Realix version: ", Color::LightGray);
    vga::print_line(OS_VERSION, Color::White);
    vga::new_line();
    vga::print_line("Realix is a lightweight hybrid x86 OS.\n", Color::LightGray);
    vga::print_line("It supports a built-in boot switcher that lets users choose:\n", Color::LightGray);
    vga::print_line("1. 16-bit Real Mode kernel for legacy compatibility\n", Color::LightGray);
    vga::print_line("2. 32-bit Protected Mode kernel for high performance.\n", Color::LightGray);
}
