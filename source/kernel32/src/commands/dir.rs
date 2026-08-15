// © Realix > Command: Dir
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::fat12;
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Выполняет dir: список файлов корневого каталога (имя + размер)
pub fn run() {
    let capacity: u16 = fat12::dir_entries_capacity();

    if capacity == 0 {
        vga::print_line("[!] Filesystem not available.\n", Color::Red);
        return;
    }

    let mut shown: u32 = 0;
    let mut buf: [u8; 10] = [0u8; 10];

    for i in 0..capacity {
        let entry = match fat12::dir_entry(i as usize) {
            Some(e) => e,
            None => break, // Конец каталога
        };

        if !fat12::is_real_entry(&entry) {
            continue; // Удалённая/LFN/volume/каталог - пропускаем
        }

        print_name(&entry.name);
        vga::print_line("  ", Color::LightGray);
        vga::print_line(utils::u32_to_dec_str(entry.size, &mut buf), Color::White);
        vga::print_line(" bytes\n", Color::LightGray);

        shown += 1;
    }

    if shown == 0 {
        vga::print_line("No files.\n", Color::LightGray);
    }
}

/// Печатает 11-байтное имя 8.3 в привычном виде "NAME.EXT" (без хвостовых пробелов)
fn print_name(name_83: &[u8; 11]) {
    let base_end: usize = name_83[..8].iter().rposition(|&b| b != b' ').map_or(0, |i| i + 1);
    let ext_end: usize = name_83[8..11].iter().rposition(|&b| b != b' ').map_or(0, |i| i + 1);

    vga::print_line(
        core::str::from_utf8(&name_83[..base_end]).unwrap_or("?"),
        Color::White,
    );

    if ext_end > 0 {
        vga::print_line(".", Color::White);
        vga::print_line(
            core::str::from_utf8(&name_83[8..8 + ext_end]).unwrap_or("?"),
            Color::White,
        );
    }
}
