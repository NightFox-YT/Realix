// © Realix > Command: Meminfo
// (22.07.26) v0.1
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};
use crate::memory::frame_allocator;
use crate::utils;

// Байт в мегабайте (для перевода объёма памяти)
const BYTES_PER_MB: usize = 1024 * 1024;

/// Выводит статистику памяти: число фреймов и объём (всего/свободно)
pub fn show() {
    let (total_frames, free_frames) = frame_allocator::get_stats();
    let mut str_buffer: [u8; 10] = [0u8; 10];

    vga::print_line("Memory information:\n", Color::Cyan);

    vga::print_line("> Total frames: ", Color::LightGray);
    vga::print_line(
        utils::u32_to_dec_str(total_frames as u32, &mut str_buffer),
        Color::White);
    vga::new_line();

    vga::print_line("> Free frames:  ", Color::LightGray);
    vga::print_line(
        utils::u32_to_dec_str(free_frames as u32, &mut str_buffer),
        Color::White);
    vga::new_line();

    let total_mb: u32 = (frame_allocator::get_total_memory() / BYTES_PER_MB) as u32;
    let free_mb: u32 = (frame_allocator::get_free_memory() / BYTES_PER_MB) as u32;

    vga::print_line("> Total memory: ", Color::LightGray);
    vga::print_line(utils::u32_to_dec_str(total_mb, &mut str_buffer), Color::White);
    vga::print_line(" MB\n", Color::LightGray);

    vga::print_line("> Free memory:  ", Color::LightGray);
    vga::print_line(utils::u32_to_dec_str(free_mb, &mut str_buffer), Color::White);
    vga::print_line(" MB\n", Color::LightGray);
}