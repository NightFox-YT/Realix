// © Realix > Command: Write
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::fat12;
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Выполняет write <name> <text...>: создаёт/перезаписывает файл текстом
pub fn run(args: &str) {
    let args = args.trim_start();
    let (name, text) = match args.find(' ') {
        Some(i) => (&args[..i], args[i + 1..].trim_start()),
        None => (args, ""),
    };

    if name.is_empty() {
        vga::print_line("[?] Usage: write <name> <text>\n", Color::LightGray);
        return;
    }

    let name_83 = match fat12::parse_name_83(name) {
        Some(n) => n,
        None => {
            vga::print_line("[!] Invalid filename.\n", Color::Red);
            return;
        }
    };

    if !fat12::write_file(&name_83, text.as_bytes()) {
        vga::print_line("[!] Write failed (disk full or directory full).\n", Color::Red);
        return;
    }

    let mut buf: [u8; 10] = [0u8; 10];
    vga::print_line("Wrote ", Color::LightGray);
    vga::print_line(utils::u32_to_dec_str(text.len() as u32, &mut buf), Color::White);
    vga::print_line(" bytes.\n", Color::LightGray);
}
