// © Realix > Shell
// (26.06.26) v0.07
// ================

// Подключение модулей
use core::str;
use core::arch::asm;
use crate::drivers::vga::{self, Color};
use crate::drivers::keyboard;

// Константы
const PROMPT: &str = "Realix >> ";

// > Основной цикл CLI
pub fn run() {
    vga::print_str("Type 'help' for list of commands.\n\n", Color::LightGray);

    loop {
        vga::print_str(PROMPT, Color::Green);
        let input_array= keyboard::read_line();
        let input_str: &str = str::from_utf8(&input_array).unwrap_or("");

        // Обрезаем по нуль-терминатору
        let end_idx: usize = input_str.find('\0').unwrap_or(input_str.len());
        execute(&input_str[..end_idx]);
    }
}


// > Функция выполнения команды
fn execute(input: &str) {
    match input {
        "help" => {
            vga::print_str("Commands:\n", Color::Cyan);
            vga::print_str("  [Base]\n", Color::Cyan);
            vga::print_str("> help     - Show this manual\n", Color::LightGray);
            vga::print_str("> clear    - Clear screen\n", Color::LightGray);
            vga::print_str("> echo [t] - Print text to console\n", Color::LightGray);
            vga::print_str("  [Power]\n", Color::Cyan);
            vga::print_str("> reboot   - Reboot PC\n", Color::LightGray);
            vga::print_str("> shutdown - Power off PC\n", Color::LightGray);
        }
        "clear" => { vga::clear_screen(); }
        "reboot" => {
            vga::print_str("Rebooting...\n", Color::Red);
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
                
                vga::print_str("PS/2 reboot failed...", Color::Red);
                loop { asm!("hlt"); }
            }
        }
        "shutdown" => {
            vga::print_str("Shutting down...\n", Color::Red);
            unsafe {
                asm!("out dx, ax", in("dx") 0x604u16, in("ax") 0x2000u16);
                
                vga::print_str("i440FX shutdown failed...", Color::Red);
                loop { asm!("hlt"); }
            }

        }
        _ if input.starts_with("echo ") => {
            vga::print_str(&input[5..], Color::LightGray);
            vga::print_char(b'\n', Color::LightGray);
        }
        "" => {}
        _ => {
            vga::print_str("[!] Unknown command. Type 'help' for list of commands.\n", Color::Red);
        }
    }
}
