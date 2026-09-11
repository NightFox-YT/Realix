// © Realix > Command: ls
// (07.09.26) v0.12
// ================
// ❗️ Зависимости: fs::fat12

// Подключение функций
use crate::drivers::vga::{self, Color};
use crate::fs::fat12::{self, DirEntry};
use crate::utils;

/// Форматирование "сырого" имени 8.3 в привычный вид "NAME.EXT" (Без паддинга)
fn format_name(raw: &[u8; 11], out: &mut [u8; 13]) -> usize {
    let name_end: usize = raw[..8].iter().rposition(|&b| b != b' ').map_or(0, |i| i + 1);
    let ext_end: usize = raw[8..11].iter().rposition(|&b| b != b' ').map_or(0, |i| i + 1);

    let mut len: usize = 0;
    out[..name_end].copy_from_slice(&raw[..name_end]);
    len += name_end;

    if ext_end > 0 {
        out[len] = b'.';
        len += 1;
        out[len..len + ext_end].copy_from_slice(&raw[8..8 + ext_end]);
        len += ext_end;
    }

    len
}

/// Команда вывода списка файлов корневого каталога: ls
pub fn run() {
    let Some(bpb) = fat12::read_bpb() else {
        vga::print_line("[!] Failed to read disk (boot sector)\n", Color::Red);
        return;
    };

    let mut count: u32 = 0;
    let mut name_buf: [u8; 13] = [0; 13];
    let mut num_buf: [u8; 10] = [0u8; 10];

    let ok: bool = fat12::list_files(&bpb, |entry: &DirEntry| {
        count += 1;
        let len: usize = format_name(&entry.name, &mut name_buf);
        let name: &str = core::str::from_utf8(&name_buf[..len]).unwrap_or("?");

        vga::print_line("  ", Color::LightGray);
        vga::print_line(name, Color::White);
        vga::print_line(" - ", Color::LightGray);
        vga::print_line(utils::u32_to_dec_str(entry.size, &mut num_buf), Color::LightGray);
        vga::print_line(" bytes\n", Color::LightGray);
    });

    if !ok {
        vga::print_line("[!] Disk read error while listing files\n", Color::Red);
        return;
    }

    vga::print_line("Total: ", Color::LightGray);
    vga::print_line(utils::u32_to_dec_str(count, &mut num_buf), Color::White);
    vga::print_line(" file(s)\n", Color::LightGray);
}
