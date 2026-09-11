// © Realix > Command: load
// (07.09.26) v0.12
// ================
// ❗️ Зависимости: commands::disk

// Подключение функций
use crate::commands::disk;
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Команда загрузки файла с диска в общий буфер: load <filename>
pub fn run(args: &str) {
    let (bpb, entry) = match disk::resolve(args, "[?] Usage: load <filename>\n") {
        Ok(v) => v,
        Err(msg) => { vga::print_line(msg, Color::LightGray); return; }
    };

    match disk::load_into_buffer(&bpb, &entry) {
        Ok(size) => {
            let mut buf: [u8; 10] = [0u8; 10];
            vga::print_line("[+] File loaded (", Color::LightGray);
            vga::print_line(utils::u32_to_dec_str(size as u32, &mut buf), Color::White);
            vga::print_line(" bytes)\n", Color::LightGray);
        }
        Err(msg) => vga::print_line(msg, Color::Red),
    }
}
