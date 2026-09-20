// © Realix > Защита стека ядра
// ================
// Загрузчик устанавливает ESP в KERNEL_STACK_TOP. Страничная адресация пока
// не включена, поэтому аппаратная защитная страница недоступна. Вместо неё
// используются зарезервированный диапазон, canary у его нижней границы
// и периодическая проверка ESP.

use core::arch::asm;

use crate::{drivers::vga, halt_loop};

pub const KERNEL_STACK_BOTTOM: usize = 0x80000;
pub const KERNEL_STACK_TOP: usize = 0x90000;

const STACK_CANARY: u32 = 0x534C_584B; // "KXLS"
const MINIMUM_HEADROOM: usize = 4096;

/// Устанавливает canary до включения аппаратных прерываний.
pub fn init() {
    unsafe {
        (KERNEL_STACK_BOTTOM as *mut u32).write_volatile(STACK_CANARY);
    }
}

/// Возвращает false, если стек достиг защитной области или в нём осталось
/// недостаточно места для фрейма прерывания и аварийной диагностики.
pub fn is_intact() -> bool {
    let esp: usize;
    unsafe {
        asm!("mov {}, esp", out(reg) esp, options(nomem, nostack, preserves_flags));
    }

    let canary = unsafe { (KERNEL_STACK_BOTTOM as *const u32).read_volatile() };
    esp >= KERNEL_STACK_BOTTOM + MINIMUM_HEADROOM
        && esp <= KERNEL_STACK_TOP
        && canary == STACK_CANARY
}

/// Немедленно останавливает систему, не продолжая работу с повреждённым ядром.
pub fn halt_on_overflow() -> ! {
    vga::clear_screen();
    vga::print_line("[KERNEL PANIC] Kernel stack overflow detected.\n", vga::Color::Red);
    halt_loop();
}
