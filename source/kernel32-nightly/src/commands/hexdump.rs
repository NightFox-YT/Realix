// © Realix > Command: hexdump
// (07.09.26) v0.12
// ================
// ❗️ Зависимости: commands::disk

// Подключение функций
use crate::commands::disk;
use crate::drivers::vga::{self, Color};
use crate::utils;

// Кол-во байт в одной строке дампа
const BYTES_PER_LINE: usize = 16;

/// Команда шестнадцатеричного дампа файла: hexdump <filename>
pub fn run(args: &str) {
    let (bpb, entry) = match disk::resolve(args, "[?] Usage: hexdump <filename>\n") {
        Ok(v) => v,
        Err(msg) => { vga::print_line(msg, Color::LightGray); return; }
    };

    let size: usize = match disk::load_into_buffer(&bpb, &entry) {
        Ok(size) => size,
        Err(msg) => { vga::print_line(msg, Color::Red); return; }
    };

    let full: &[u8; disk::FILE_BUFFER_SIZE] = unsafe { &*&raw const disk::FILE_BUFFER };
    let bytes: &[u8] = &full[..size];
    let mut addr_buf: [u8; 10] = [0u8; 10];
    let mut byte_buf: [u8; 10] = [0u8; 10];

    for (line_start, chunk) in bytes.chunks(BYTES_PER_LINE).enumerate() {
        vga::print_line(
            utils::u32_to_hex_str((line_start * BYTES_PER_LINE) as u32, &mut addr_buf),
            Color::Cyan,
        );
        vga::print_line("  ", Color::LightGray);

        for &byte in chunk {
            let hex: &str = utils::u32_to_hex_str(byte as u32, &mut byte_buf);
            vga::print_line(&hex[6..], Color::White); // Только последние 2 hex-цифры
            vga::print_char(b' ', Color::LightGray);
        }

        vga::print_line(" | ", Color::LightGray);
        for &byte in chunk {
            let printable: u8 = if (0x20..=0x7E).contains(&byte) { byte } else { b'.' };
            vga::print_char(printable, Color::LightGray);
        }
        vga::new_line();
    }
}
