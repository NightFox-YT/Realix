// © Realix > Command: Delete
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::fat12;
use crate::drivers::vga::{self, Color};

/// Выполняет delete <name>: удаляет файл
pub fn run(args: &str) {
    let name = args.trim();
    if name.is_empty() {
        vga::print_line("[?] Usage: delete <name>\n", Color::LightGray);
        return;
    }

    let name_83 = match fat12::parse_name_83(name) {
        Some(n) => n,
        None => {
            vga::print_line("[!] Invalid filename.\n", Color::Red);
            return;
        }
    };

    if fat12::delete_file(&name_83) {
        vga::print_line("Deleted.\n", Color::LightGray);
    } else {
        vga::print_line("[!] File not found.\n", Color::Red);
    }
}
