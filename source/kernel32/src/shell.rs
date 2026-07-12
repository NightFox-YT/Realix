// © Realix > Shell
// (03.07.26) v0.08
// ø Вдохновлено @liquifield
// ================

// Импорт функций
use core::arch::asm;
use crate::drivers::vga::{self, Color};
use crate::drivers::{pit, keyboard};
use crate::utils;
use crate::commands::matrix;

// Константы
const PROMPT: &str = "Realix >> ";

/// Основной цикл CLI
pub fn run() {
    vga::print_line("Type 'help' for list of commands.\n\n", Color::LightGray);

    loop {
        vga::print_line(PROMPT, Color::Green);
        let input_array = keyboard::read_line();
        let input_str: &str = core::str::from_utf8(&input_array).unwrap_or("");

        // Обрезаем по нуль-терминатору
        let end_idx: usize = input_str.find('\0').unwrap_or(input_str.len());
        execute(&input_str[..end_idx]);
    }
}


/// Функция выполнения команды
fn execute(input: &str) {
    // Форматируем введённую строку
    let input: &str = input.trim();

    match input {
        "help" => {
            vga::print_line("Commands:\n", Color::Cyan);
            vga::print_line("  [Base]\n", Color::Cyan);
            vga::print_line("> help     - Show this manual\n", Color::LightGray);
            vga::print_line("> clear    - Clear screen\n", Color::LightGray);
            vga::print_line("> echo [t] - Print text to console\n", Color::LightGray);
            vga::print_line("> uptime   - Show uptime (seconds)\n", Color::LightGray);
            vga::print_line("  [Fun]\n", Color::Cyan);
            vga::print_line("> matrix   - Show matrix rain\n", Color::LightGray);
            vga::print_line("  [Power]\n", Color::Cyan);
            vga::print_line("> reboot   - Reboot PC\n", Color::LightGray);
            vga::print_line("> shutdown - Power off PC\n", Color::LightGray);
        }
        "clear" => { vga::clear_screen(); }
        "reboot" => {
            vga::print_line("Rebooting...\n", Color::Red);
            unsafe {
                let mut timeout: u32 = 0;

                // Опрашиваем контроллер PS/2
                loop {
                    let status: u8;
                    asm!("in al, 0x64", out("al") status);
                    
                    // Если порта нет (0xFF) или превышен таймаут, запасной план
                    if status == 0xFF || timeout > 100_000 {
                        break;
                    }
                    
                    // Если первый бит равен 0 (Входной буфер PS/2 пуст), сбрасываем
                    if status & 0x02 == 0 {
                        asm!("out 0x64, al", in("al") 0xFEu8);
                        break;
                    }
                    timeout += 1;
                }
                
                vga::print_line("PS/2 reboot failed...", Color::Red);
                crate::halt_loop();
            }
        }
        "shutdown" => {
            vga::print_line("Shutting down...\n", Color::Red);
            unsafe {
                asm!("out dx, ax", in("dx") 0x604u16, in("ax") 0x2000u16);
                
                vga::print_line("i440FX shutdown failed...", Color::Red);
                crate::halt_loop();
            }

        }
        _ if input.starts_with("echo ") => {
            vga::print_line(&input[5..], Color::LightGray);
            vga::new_line();
        }
        "echo" => { vga::new_line(); }
        "uptime" => {
            let mut str_buffer: [u8; 10] = [0u8; 10];

            vga::print_line("Uptime (seconds): ", Color::LightGray);
            vga::print_line(
                utils::u32_to_dec_str(pit::get_uptime(), &mut str_buffer),
                Color::LightGray);
            vga::new_line();
        }
        "matrix" => {
            vga::print_line("Entering Matrix... (Press any key to exit)\n", Color::Green);
            matrix::run();
        }
        "" => {}
        _ => {
            vga::print_line("[!] Unknown command. Type 'help' for list of commands.\n", Color::Red);
        }
    }
}
