// © Realix > Shell
// (12.07.26) v0.09
// ø Вдохновлено @liquifield
// ================

// Импорт функций
use core::arch::asm;
use crate::drivers::vga::{self, Color};
use crate::drivers::{pit, keyboard};
use crate::utils;
use crate::commands::matrix;
use crate::x86::frame_allocator;

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
            vga::print_line("  [Memory]\n", Color::Cyan);
            vga::print_line("> meminfo  - Show memory information\n", Color::LightGray);
            vga::print_line("  [Fun]\n", Color::Cyan);
            vga::print_line("> matrix   - Show matrix rain\n", Color::LightGray);
            vga::print_line("  [Power]\n", Color::Cyan);
            vga::print_line("> reboot   - Reboot PC\n", Color::LightGray);
            vga::print_line("> shutdown - Power off PC\n", Color::LightGray);
        }
        "clear" => { vga::clear_screen(); }
        "meminfo" => {
            let (total, free) = frame_allocator::get_stats();
            let mut buf: [u8; 10] = [0; 10];
            
            vga::print_line("Memory Information:\n", Color::Cyan);
            vga::print_line("  Total frames: ", Color::LightGray);
            vga::print_line(utils::u32_to_dec_str(total as u32, &mut buf), Color::White);
            vga::new_line();
            
            vga::print_line("  Free frames: ", Color::LightGray);
            vga::print_line(utils::u32_to_dec_str(free as u32, &mut buf), Color::White);
            vga::new_line();
            
            let total_mb: u32 = (total as u32 * 4096) / (1024 * 1024);
            let free_mb: u32 = (free as u32 * 4096) / (1024 * 1024);
            
            vga::print_line("  Total memory: ", Color::LightGray);
            vga::print_line(utils::u32_to_dec_str(total_mb, &mut buf), Color::White);
            vga::print_line(" MB\n", Color::LightGray);
            
            vga::print_line("  Free memory: ", Color::LightGray);
            vga::print_line(utils::u32_to_dec_str(free_mb, &mut buf), Color::White);
            vga::print_line(" MB\n", Color::LightGray);
        }
        "reboot" => {
            vga::print_line("Rebooting...\n", Color::Red);
            unsafe {
                let mut timeout: u32 = 0;

                // Метод 1: Опрашиваем контроллер PS/2
                loop {
                    let status: u8;
                    asm!("in al, 0x64", out("al") status);
                    
                    if status == 0xFF || timeout > 100_000 {
                        break;
                    }
                    
                    if status & 0x02 == 0 {
                        asm!("out 0x64, al", in("al") 0xFEu8);
                        vga::print_line("PS/2 reboot command sent.\n", Color::Green);
                        // Ждём немного для перезагрузки
                        for _ in 0..1000000 { asm!("nop"); }
                        break;
                    }
                    timeout += 1;
                }
                
                // Метод 2: Сброс через контроллер клавиатуры
                vga::print_line("Trying keyboard controller reset...\n", Color::Red);
                timeout = 0;
                loop {
                    let status: u8;
                    asm!("in al, 0x64", out("al") status);
                    
                    if status == 0xFF || timeout > 100_000 {
                        break;
                    }
                    
                    if status & 0x02 == 0 {
                        asm!("out 0x64, al", in("al") 0xD1u8);
                        for _ in 0..1000 { asm!("nop"); }
                        asm!("out 0x60, al", in("al") 0xFEu8);
                        vga::print_line("Keyboard controller reset sent.\n", Color::Green);
                        for _ in 0..1000000 { asm!("nop"); }
                        break;
                    }
                    timeout += 1;
                }
                
                // Метод 3: Тройной отказ (Triple fault)
                vga::print_line("All methods failed. Attempting triple fault...\n", Color::Red);
                // Загружаем пустой IDT, чтобы вызвать тройной отказ
                let null_idt: [u8; 6] = [0; 6];
                asm!("lidt [{}]", in(reg) &null_idt, options(nostack));
                // Генерируем исключение (деление на ноль)
                asm!("div {}", in(reg) 0u32, options(nostack));
                
                vga::print_line("Triple fault failed. Halting.\n", Color::Red);
                crate::halt_loop();
            }
        }
        "shutdown" => {
            vga::print_line("Shutting down...\n", Color::Red);
            unsafe {
                // Метод 1: i440FX (QEMU)
                asm!("out dx, ax", in("dx") 0x604u16, in("ax") 0x2000u16);
                for _ in 0..100000 { asm!("nop"); }
                
                // Метод 2: Bochs/QEMU alternative
                asm!("out dx, ax", in("dx") 0xB004u16, in("ax") 0x2000u16);
                for _ in 0..100000 { asm!("nop"); }
                
                // Метод 3: VirtualBox
                asm!("out dx, ax", in("dx") 0x4004u16, in("ax") 0x3400u16);
                for _ in 0..100000 { asm!("nop"); }
                
                // Метод 4: ACPI (если доступен)
                // Пробуем отправить команду через PM1a_CNT
                asm!("out dx, ax", in("dx") 0x1000u16, in("ax") 0x2000u16);
                
                vga::print_line("All shutdown methods failed. Halting.\n", Color::Red);
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
