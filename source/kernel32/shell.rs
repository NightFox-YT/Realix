use crate::vga::{self, Color};
use crate::keyboard;

const PROMPT: &str = "Realix> ";

pub fn run() {
    vga::print_str("Type 'help' for commands.\n", Color::LightGray);
    loop {
        vga::print_str(PROMPT, Color::Green);
        let input = keyboard::read_line();
        let input_str = core::str::from_utf8(&input).unwrap_or("");
        // Обрезаем по нуль-терминатору
        let end = input_str.find('\0').unwrap_or(input_str.len());
        execute(&input_str[..end]);
    }
}

fn execute(input: &str) {
    match input {
        "help" => {
            vga::print_str("Commands:\n", Color::Cyan);
            vga::print_str("  help     - Show this manual\n", Color::LightGray);
            vga::print_str("  clear    - Clear screen\n", Color::LightGray);
            vga::print_str("  echo [t] - Print text\n", Color::LightGray);
            vga::print_str("  meminfo  - Show memory info\n", Color::LightGray);
            vga::print_str("  reboot   - Reboot PC\n", Color::LightGray);
            vga::print_str("  shutdown - Power off PC\n", Color::LightGray);
        }
        "clear" => {
            vga::clear_screen();
        }
        "reboot" => {
            vga::print_str("Rebooting...\n", Color::Red);
            unsafe {
                let mut good = 0x02u8;
                while good & 0x02 != 0 {
                    let status: u8;
                    core::arch::asm!("in al, 0x64", out("al") status);
                    good = status;
                }
                core::arch::asm!("out 0x64, al", in("al") 0xFEu8);
            }
            loop {
                unsafe { core::arch::asm!("hlt"); }
            }
        }
        "shutdown" => {
            vga::print_str("Shutting down...\n", Color::Red);
            unsafe {
                core::arch::asm!(
                    "out dx, ax",
                    in("dx") 0x604u16,
                    in("ax") 0x2000u16,
                );
            }
            loop {
                unsafe { core::arch::asm!("hlt"); }
            }
        }
        _ if input.starts_with("echo ") => {
            vga::print_str(&input[5..], Color::LightGray);
            vga::print_str("\n", Color::LightGray);
        }
        "meminfo" => {
            vga::print_str("Memory: 640 KB lower memory\n", Color::Yellow);
        }
        "" => {}
        _ => {
            vga::print_str("Unknown command. Type 'help' for list.\n", Color::Red);
        }
    }
}
