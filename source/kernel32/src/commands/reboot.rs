// © Realix > Command: Reboot
// (22.07.26) v0.1
// ================

// Подключение функций
use core::arch::asm;
use crate::drivers::{pit, vga::{self, Color}};
use crate::utils::{inb, outb};

// Порты и команды контроллера PS/2
const PS2_STATUS_PORT:      u16 = 0x64;  // Порт статуса/команд
const PS2_DATA_PORT:        u16 = 0x60;  // Порт данных
const PS2_CMD_PULSE_RESET:   u8 = 0xFE;  // Импульс на линии reset (перезагрузка)
const PS2_CMD_WRITE_OUTPUT:  u8 = 0xD1;  // Команда записи в выходной порт контроллера
const PS2_STATUS_INPUT_FULL: u8 = 0x02;  // Бит "входной буфер занят"
const PS2_PORT_ABSENT:       u8 = 0xFF;  // Контроллер отсутствует (порт читается как 0xFF)

// Лимит опроса контроллера перед переходом к следующему методу
const REBOOT_POLL_LIMIT: u32 = 100_000;

/// Перезагрузка ПК: PS/2 -> контроллер клавиатуры -> triple fault
pub fn run() {
    vga::print_line("[.] Trying sent PS/2 controller reboot command...\n", Color::Red);
    unsafe {
        let mut timeout: u32 = 0;

        // Метод 1: Опрашиваем контроллер PS/2
        loop {
            let status: u8 = inb(PS2_STATUS_PORT);

            // Если порта нет (0xFF) или превышен таймаут, запасной план
            if status == PS2_PORT_ABSENT || timeout > REBOOT_POLL_LIMIT {
                break;
            }

            // Если входной буфер пуст (бит сброшен), отправляем сброс
            if status & PS2_STATUS_INPUT_FULL == 0 {
                outb(PS2_STATUS_PORT, PS2_CMD_PULSE_RESET);
                vga::print_line("[+] PS/2 reboot command sent.\n", Color::Green);
                break;
            }
            timeout += 1;
        }

        // Метод 2: Сброс через контроллер клавиатуры
        vga::print_line("[.] Trying keyboard controller reset...\n", Color::Red);
        timeout = 0;

        loop {
            let status: u8 = inb(PS2_STATUS_PORT);

            // Если порта нет (0xFF) или превышен таймаут, запасной план
            if status == PS2_PORT_ABSENT || timeout > REBOOT_POLL_LIMIT {
                break;
            }

            // Если входной буфер пуст (бит сброшен), отправляем сброс
            if status & PS2_STATUS_INPUT_FULL == 0 {
                outb(PS2_STATUS_PORT, PS2_CMD_WRITE_OUTPUT);
                pit::sleep(10);
                outb(PS2_DATA_PORT, PS2_CMD_PULSE_RESET);
                vga::print_line("[+] Keyboard controller reset sent.\n", Color::Green);
                break;
            }
            timeout += 1;
        }

        // Метод 3: Тройной отказ (Triple fault)
        vga::print_line("[.] All methods failed. Attempting triple fault...\n", Color::Red);

        // Загружаем пустой IDT, чтобы вызвать тройной отказ
        let null_idt: [u8; 6] = [0; 6];
        asm!("lidt [{}]", in(reg) &null_idt, options(nostack));

        // Генерируем исключение (деление на ноль)
        asm!("div {}", in(reg) 0u32, options(nostack));

        vga::print_line("[!] Triple fault failed. Halting.\n", Color::Red);
        crate::halt_loop();
    }
}
