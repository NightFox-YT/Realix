// © Realix > Command: Help
// (22.07.26) v0.1
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};

/// Выводит список доступных команд, сгруппированный по категориям
pub fn show() {
    vga::print_line("Commands:\n", Color::LightCyan);
    vga::print_line("  [Base]\n", Color::Cyan);
    vga::print_line("> help      - Show this manual\n", Color::LightGray);
    vga::print_line("> clear/cls - Clear screen\n", Color::LightGray);
    vga::print_line("> echo [t]  - Print text to console\n", Color::LightGray);
    vga::print_line("> uptime    - Show uptime (seconds)\n", Color::LightGray);
    vga::print_line("> meminfo   - Show memory information\n", Color::LightGray);
    vga::print_line("  [Fat12]\n", Color::Cyan);
    vga::print_line("> dir             - List root directory\n", Color::LightGray);
    vga::print_line("> type <f>        - Print file as text\n", Color::LightGray);
    vga::print_line("> write <f> <t>   - Create/overwrite file with text\n", Color::LightGray);
    vga::print_line("> rename <f> <f2> - Rename file\n", Color::LightGray);
    vga::print_line("> delete <f>      - Delete file\n", Color::LightGray);
    vga::print_line("  [Fun]\n", Color::Cyan);
    vga::print_line("> matrix    - Show matrix rain\n", Color::LightGray);
    vga::print_line("  [Power]\n", Color::Cyan);
    vga::print_line("> reboot    - Reboot PC\n", Color::LightGray);
    vga::print_line("> shutdown  - Power off PC\n", Color::LightGray);
}