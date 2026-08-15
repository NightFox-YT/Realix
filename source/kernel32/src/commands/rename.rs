// © Realix > Command: Rename
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::fat12;
use crate::drivers::vga::{self, Color};

/// Выполняет rename <old> <new>: переименовывает файл
pub fn run(args: &str) {
    let mut parts = args.split_whitespace();
    let old_name = parts.next();
    let new_name = parts.next();
    let extra = parts.next();

    let (old_name, new_name) = match (old_name, new_name, extra) {
        (Some(o), Some(n), None) => (o, n),
        _ => {
            vga::print_line("[?] Usage: rename <old> <new>\n", Color::LightGray);
            return;
        }
    };

    let old_83 = match fat12::parse_name_83(old_name) {
        Some(n) => n,
        None => {
            vga::print_line("[!] Invalid filename.\n", Color::Red);
            return;
        }
    };
    let new_83 = match fat12::parse_name_83(new_name) {
        Some(n) => n,
        None => {
            vga::print_line("[!] Invalid filename.\n", Color::Red);
            return;
        }
    };

    if fat12::rename_file(&old_83, &new_83) {
        vga::print_line("Renamed.\n", Color::LightGray);
    } else {
        vga::print_line("[!] Rename failed (not found, or new name already exists).\n", Color::Red);
    }
}
