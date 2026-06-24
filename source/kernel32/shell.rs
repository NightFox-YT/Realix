use crate::vga::{self, Color};
use crate::keyboard;

const PROMPT: &str = "Realix> ";

fn print_hex_byte(n: u8) {
    let hex = b"0123456789ABCDEF";
    vga::put_char(hex[(n >> 4) as usize], vga::Color::White);
    vga::put_char(hex[(n & 0x0F) as usize], vga::Color::White);
}

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
	    vga::print_str("  snake    - Play Snake game\n", vga::Color::LightGray);
	    vga::print_str("  fetch    - Show OS information\n", vga::Color::LightGray);
	    vga::print_str("  matrix   - Matrix rain animation\n", vga::Color::LightGray);
            vga::print_str("  calc     - Simple calculator\n", vga::Color::LightGray);
            vga::print_str("  netinfo  - Show network card MAC\n", vga::Color::LightGray);
        }
        "matrix" => {
            vga::print_str("Entering Matrix... (press any key to exit)\n", vga::Color::Green);
            crate::matrix::run();
        }
        "netinfo" => {
            if crate::rtl8139::is_ready() {
                let mut mac: [u8; 6] = [0; 6];
                crate::rtl8139::get_mac(&mut mac);
                vga::print_str("MAC: ", vga::Color::Green);
                for i in 0..6 {
                    print_hex_byte(mac[i]);
                    if i < 5 {
                        vga::put_char(b':', vga::Color::White);
                    }
                }
                vga::put_char(b'\n', vga::Color::LightGray);
            } else {
                vga::print_str("RTL8139 not found\n", vga::Color::Red);
            }
        }
        "clear" => {
            vga::clear_screen();
        }
	"fetch" | "neofetch" | "fastfetch" => {
	    crate::fetch::run();
        }
	
	"snake" => {
	    vga::print_str("Starting Snake...(WASD to move, Q to quit)\n", vga::Color::Green);
	    crate::snake::run();
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
        _ if input.starts_with("calc ") => {
            crate::calc::run(&input[5..]);
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
