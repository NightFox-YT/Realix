use core::arch::naked_asm;
use crate::vga;

pub fn init() {
    unsafe {
        core::arch::asm!("out 0x20, al", in("al") 0x11u8);
        core::arch::asm!("out 0xA0, al", in("al") 0x11u8);
        core::arch::asm!("out 0x21, al", in("al") 0x20u8);
        core::arch::asm!("out 0xA1, al", in("al") 0x28u8);
        core::arch::asm!("out 0x21, al", in("al") 0x04u8);
        core::arch::asm!("out 0xA1, al", in("al") 0x02u8);
        core::arch::asm!("out 0x21, al", in("al") 0x01u8);
        core::arch::asm!("out 0xA1, al", in("al") 0x01u8);
        core::arch::asm!("out 0x21, al", in("al") 0xF9u8);
        core::arch::asm!("out 0xA1, al", in("al") 0xFFu8);
    }
}
#[unsafe(naked)]
pub unsafe extern "C" fn irq0_handler() {
    unsafe {
        naked_asm!(
            "push eax",
            "mov al, 0x20",
            "out 0x20, al",
            "pop eax",
            "iret",
        );
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn keyboard_handler() {
    unsafe {
        naked_asm!(
            "pushad",
            "mov al, 0x20",
            "out 0x20, al",
            "popad",
            "iret",
        );
    }
}

unsafe extern "C" fn keyboard_handler_impl() {
    
}

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
    loop { unsafe { core::arch::asm!("hlt"); } }
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
    loop { unsafe { core::arch::asm!("hlt"); } }
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
    loop { unsafe { core::arch::asm!("hlt"); } }
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
    loop { unsafe { core::arch::asm!("hlt"); } }
}

#[unsafe(naked)]
pub unsafe extern "C" fn default_handler() {
    unsafe {
        naked_asm!(
            "pushad",
            "mov al, 0x20",
            "out 0x20, al",
            "popad",
            "iret",
        );
    }
}
