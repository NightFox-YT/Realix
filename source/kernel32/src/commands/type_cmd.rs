// © Realix > Command: Type
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::fat12;
use crate::drivers::vga::{self, Color};

// Максимальный размер файла, отображаемого через type (без динамической памяти)
const MAX_TEXT_SIZE: usize = 4096;

/// Выполняет type <name>: печатает содержимое файла как текст
pub fn run(args: &str) {
    let name = args.trim();
    if name.is_empty() {
        vga::print_line("[?] Usage: type <name>\n", Color::LightGray);
        return;
    }

    let name_83 = match fat12::parse_name_83(name) {
        Some(n) => n,
        None => {
            vga::print_line("[!] Invalid filename.\n", Color::Red);
            return;
        }
    };

    let mut buf: [u8; MAX_TEXT_SIZE] = [0; MAX_TEXT_SIZE];
    match fat12::read_file(&name_83, &mut buf) {
        Some(size) => {
            let text = core::str::from_utf8(&buf[..size as usize]).unwrap_or("<binary data>");
            vga::print_line(text, Color::White);
            vga::new_line_if_needed();
        }
        None => {
            vga::print_line("[!] File not found (or too large to display).\n", Color::Red);
        }
    }
}
