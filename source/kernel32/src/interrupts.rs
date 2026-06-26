use core::arch::naked_asm;
use crate::vga;
use crate::keyboard;

// > Инициализация PIC (Programmable Interrupt Controller)
// Переназначаем векторы: Master IRQ0→0x20, Slave IRQ8→0x28
// Маска по умолчанию: 0xF9 (IRQ1 + каскад IRQ2)
pub fn init() {
    init_pic_mask(0xF9);
}

// Инициализация PIC с указанной маской master PIC.
// slave всегда маскируется полностью (0xFF).
pub fn init_pic_mask(master_mask: u8) {
    unsafe {
        // ICW1: инициализация, ждём ICW4
        core::arch::asm!("out 0x20, al", in("al") 0x11u8);
        core::arch::asm!("out 0xA0, al", in("al") 0x11u8);
        // ICW2: базовые векторы прерываний
        core::arch::asm!("out 0x21, al", in("al") 0x20u8); // Master: IRQ0 → INT 0x20
        core::arch::asm!("out 0xA1, al", in("al") 0x28u8); // Slave:  IRQ8 → INT 0x28
        // ICW3: каскадирование (IRQ2 на master)
        core::arch::asm!("out 0x21, al", in("al") 0x04u8);
        core::arch::asm!("out 0xA1, al", in("al") 0x02u8);
        // ICW4: режим x86
        core::arch::asm!("out 0x21, al", in("al") 0x01u8);
        core::arch::asm!("out 0xA1, al", in("al") 0x01u8);
        // Маски
        core::arch::asm!("out 0x21, al", in("al") master_mask);
        core::arch::asm!("out 0xA1, al", in("al") 0xFFu8);
    }
}

// > Разрешить аппаратные прерывания (STI)
pub fn enable() {
    unsafe {
        core::arch::asm!("sti");
    }
}

// > Запретить аппаратные прерывания (CLI)
#[allow(dead_code)]
pub fn disable() {
    unsafe {
        core::arch::asm!("cli");
    }
}

// ----- IRQ0: Таймер (PIT) -----
#[unsafe(naked)]
pub unsafe extern "C" fn irq0_handler() {
    unsafe {
        naked_asm!(
            "push eax",
            "mov al, 0x20",
            "out 0x20, al",   // EOI master
            "pop eax",
            "iret",
        );
    }
}

// ----- IRQ1: Клавиатура -----
#[unsafe(naked)]
pub unsafe extern "C" fn keyboard_handler() {
    unsafe {
        naked_asm!(
            "pushad",
            "call {}",
            "popad",
            "iret",
            sym keyboard_handler_impl
        );
    }
}

unsafe extern "C" fn keyboard_handler_impl() {
    let scancode: u8;
    core::arch::asm!("in al, 0x60", out("al") scancode);
    core::arch::asm!("out 0x20, al", in("al") 0x20u8);
    keyboard::handle_scancode(scancode);
}

// ----- Исключения CPU -----

#[unsafe(naked)]
pub unsafe extern "C" fn divide_by_zero_handler() {
    unsafe {
        naked_asm!(
            "pushad",
            "call {}",
            "popad",
            "iret",
            sym divide_by_zero_impl
        );
    }
}

unsafe extern "C" fn divide_by_zero_impl() {
    vga::print_str("\n[PANIC] Division by zero!\n", vga::Color::Red);
    loop { core::arch::asm!("hlt"); }
}

#[unsafe(naked)]
pub unsafe extern "C" fn double_fault_handler() {
    unsafe {
        naked_asm!(
            "pushad",
            "call {}",
            "popad",
            "iret",
            sym double_fault_impl
        );
    }
}

unsafe extern "C" fn double_fault_impl() {
    vga::print_str("\n[PANIC] Double fault!\n", vga::Color::Red);
    loop { core::arch::asm!("hlt"); }
}

#[unsafe(naked)]
pub unsafe extern "C" fn general_protection_fault_handler() {
    unsafe {
        naked_asm!(
            "pushad",
            "call {}",
            "popad",
            "iret",
            sym gpf_impl
        );
    }
}

unsafe extern "C" fn gpf_impl() {
    vga::print_str("\n[PANIC] General Protection Fault!\n", vga::Color::Red);
    loop { core::arch::asm!("hlt"); }
}

#[unsafe(naked)]
pub unsafe extern "C" fn invalid_opcode_handler() {
    unsafe {
        naked_asm!(
            "pushad",
            "call {}",
            "popad",
            "iret",
            sym invalid_opcode_impl
        );
    }
}

unsafe extern "C" fn invalid_opcode_impl() {
    vga::print_str("\n[PANIC] Invalid Opcode!\n", vga::Color::Red);
    loop { core::arch::asm!("hlt"); }
}

// ----- Заглушка для неиспользуемых векторов -----
// Минимальный обработчик: push eax / EOI master / pop eax / iret.
// НЕ шлём EOI в slave PIC — это вызывает проблемы при master-only IRQ.
// НЕ используем pushad/popad — достаточно сохранить только eax.
#[unsafe(naked)]
pub unsafe extern "C" fn default_handler() {
    unsafe {
        naked_asm!(
            "push eax",
            "mov al, 0x20",
            "out 0x20, al",  // EOI master PIC
            "pop eax",
            "iret",
        );
    }
}
