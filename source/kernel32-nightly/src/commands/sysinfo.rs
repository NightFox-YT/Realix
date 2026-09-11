// © Realix > Command: Sysinfo
// (07.09.26) v0.12
// ================
// ❗️ Зависимости: crate::pcinfo() (структура PCINFO от загрузчика)

// Подключение функций
use crate::drivers::pit;
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Выводит системную информацию: видеорежим, тики с загрузки, загрузочный диск
pub fn show() {
    let mut buf: [u8; 10] = [0u8; 10];

    // Kernel32 всегда работает в текстовом режиме 80x25 (кроме временной VGA-демо)
    vga::print_line("Video mode: Text 80x25\n", Color::LightGray);

    vga::print_line("Ticks since boot: ", Color::LightGray);
    vga::print_line(utils::u32_to_dec_str(pit::get_ticks(), &mut buf), Color::White);
    vga::new_line();

    let boot_drive: u8 = unsafe { crate::pcinfo().boot_drive_num };
    vga::print_line("Boot drive (num): ", Color::LightGray);
    vga::print_line(utils::u32_to_hex_str(boot_drive as u32, &mut buf), Color::White);
    vga::new_line();
}
