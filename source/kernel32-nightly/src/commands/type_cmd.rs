// © Realix > Command: type
// (07.09.26) v0.12
// ================
// ❗️ Зависимости: commands::disk
// ❗️ Имя файла - `type_cmd.rs`, т.к. `type` - зарезервированное слово Rust

// Подключение функций
use crate::commands::disk;
use crate::drivers::vga::{self, Color};

/// Команда печати файла как текста: type <filename>
pub fn run(args: &str) {
    let (bpb, entry) = match disk::resolve(args, "[?] Usage: type <filename>\n") {
        Ok(v) => v,
        Err(msg) => { vga::print_line(msg, Color::LightGray); return; }
    };

    match disk::load_into_buffer(&bpb, &entry) {
        Ok(size) => {
            let full: &[u8; disk::FILE_BUFFER_SIZE] = unsafe { &*&raw const disk::FILE_BUFFER };
            let bytes: &[u8] = &full[..size];
            for &byte in bytes {
                // Печатаем как есть - непечатные байты BIOS/VGA просто отрисует "как есть"
                vga::print_char(byte, Color::LightGray);
            }
            vga::new_line_if_needed();
        }
        Err(msg) => vga::print_line(msg, Color::Red),
    }
}
