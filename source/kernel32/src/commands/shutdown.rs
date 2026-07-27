// © Realix > Command: Shutdown
// (22.07.26) v0.1
// ================

// Подключение функций
use crate::drivers::{pit, vga::{self, Color}};
use crate::utils::outw;

// Порты выключения по платформам (ACPI PM1a_CNT и его аналоги)
const PORT_I440FX:     u16 = 0x604;  // i440FX (QEMU)
const PORT_BOCHS:      u16 = 0xB004; // Bochs / старый QEMU
const PORT_VIRTUALBOX: u16 = 0x4004; // VirtualBox
const PORT_ACPI:       u16 = 0x1000; // ACPI PM1a_CNT (запасной)

// Слово команды сна (SLP_TYP | SLP_EN) для разных платформ
const SLEEP_CMD_QEMU: u16 = 0x2000;
const SLEEP_CMD_VBOX: u16 = 0x3400;

// Пауза между попытками, мс
const SHUTDOWN_STEP_DELAY: u32 = 100;

/// Выключение ПК: i440FX -> Bochs/QEMU alternative/VirtualBox -> ACPI
pub fn run() {
    vga::print_line("Shutting down...\n", Color::Red);
    unsafe {
        // Метод 1: i440FX (QEMU)
        outw(PORT_I440FX, SLEEP_CMD_QEMU);
        pit::sleep(SHUTDOWN_STEP_DELAY);

        // Метод 2: Bochs/QEMU alternative
        outw(PORT_BOCHS, SLEEP_CMD_QEMU);
        pit::sleep(SHUTDOWN_STEP_DELAY);

        // Метод 3: VirtualBox
        outw(PORT_VIRTUALBOX, SLEEP_CMD_VBOX);
        pit::sleep(SHUTDOWN_STEP_DELAY);

        vga::print_line("[!] i440FX shutdown failed...", Color::Red);

        // Метод 4: ACPI (Если доступен)
        // Пробуем отправить команду через PM1a_CNT
        outw(PORT_ACPI, SLEEP_CMD_QEMU);

        vga::print_line("[!] All shutdown methods failed. Halting.\n", Color::Red);
        crate::halt_loop();
    }
}
