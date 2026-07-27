// © Realix > IDT
// (03.07.26) v0.08
// ================

// Подключение функций
use core::ptr::addr_of_mut;
use core::mem::size_of;
use crate::x86::{gdt, isr, pic};

// Константы
const IDT_SIZE: usize = 256;
const IDT_GATE_32BIT: u8 = 0x8E;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct InterruptDescriptor {
    base_low:  u16,
    selector:  u16,
    reserved:  u8,
    flags:     u8,
    base_high: u16,
}

impl InterruptDescriptor {
    /// Обработчик прерывания отсутствует
    pub const fn missing() -> Self {
        InterruptDescriptor {
            base_low: 0, selector: 0,
            reserved: 0, flags: 0,
            base_high: 0,
        }
    }

    /// Установка обработчика для дескриптора прерывания
    pub fn set_handler(&mut self, handler_addr: u32, selector: u16, flags: u8) {
        self.base_low = (handler_addr & 0xFFFF) as u16;
        self.selector = selector;
        self.reserved = 0;
        self.flags = flags;
        self.base_high = ((handler_addr >> 16) & 0xFFFF) as u16;
    }
}

// Структура-указатель для инструкции LIDT
#[repr(C, packed)]
pub struct IdtPointer {
    limit: u16,
    base: u32,
}

// Занимаем место в памяти для IDT (! Инициализировать 1 раз)
static mut IDT: [InterruptDescriptor; IDT_SIZE] = [InterruptDescriptor::missing(); IDT_SIZE];

/// Функция инициализации IDT
pub fn init() {
    unsafe { set_handlers(&mut *addr_of_mut!(IDT)); }
    pic::remap();

    // Формируем указатель на IDT
    let idt_pointer: IdtPointer = IdtPointer {
        limit: (size_of::<[InterruptDescriptor; IDT_SIZE]>() - 1) as u16,
        base: &raw const IDT as u32,
    };

    // Загружаем таблицу в процессор
    unsafe {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &raw const idt_pointer,
            options(readonly, nostack, preserves_flags),
        );
    }
}

/// Регистрирует обработчиков прерываний CPU
fn set_handlers(idt_addr: &mut [InterruptDescriptor; IDT_SIZE]) {
    // Макрос для установки прерывания
    macro_rules! set {
        ($vec:expr, $handler:expr) => {
            idt_addr[$vec].set_handler(
                $handler as u32, gdt::KERNEL_CODE_SELECTOR, IDT_GATE_32BIT
            );
        };
    }

    // Установка обработчиков исключений
    set!(0, isr::exc_divide_by_zero as *const ());
    set!(1, isr::exc_debug as *const ());
    set!(2, isr::exc_non_maskable_interrupt as *const ());
    set!(3, isr::exc_breakpoint as *const ());
    set!(4, isr::exc_overflow as *const ());
    set!(5, isr::exc_bound_range_exceeded as *const ());
    set!(6, isr::exc_invalid_opcode as *const ());
    set!(7, isr::exc_device_not_available as *const ());
    set!(8, isr::exc_double_fault as *const ());
    set!(9, isr::exc_coprocessor_segment_overrun as *const ());
    set!(10, isr::exc_invalid_tss as *const ());
    set!(11, isr::exc_segment_not_present as *const ());
    set!(12, isr::exc_stack_segment_fault as *const ());
    set!(13, isr::exc_general_protection_fault as *const ());
    set!(14, isr::exc_page_fault as *const ());
    // ... (вектор 15 зарезервирован под Intel, обработчик не генерируется)
    set!(16, isr::exc_x86_floating_point_exception as *const ());
    set!(17, isr::exc_alignment_check as *const ());
    set!(18, isr::exc_machine_check as *const ());
    set!(19, isr::exc_simd_floating_point_exception as *const ());

    // Установка обработчиков IRQ (Не все пока обрабатываются)
    set!(32, isr::irq_stub_32 as *const ());  // PIT (Programmable Interval Timer)
    set!(33, isr::irq_stub_33 as *const ());
    set!(34, isr::irq_stub_34 as *const ());
    set!(35, isr::irq_stub_35 as *const ());
    set!(36, isr::irq_stub_36 as *const ());
    set!(37, isr::irq_stub_37 as *const ());
    set!(38, isr::irq_stub_38 as *const ());
    set!(39, isr::irq_stub_39 as *const ());
    set!(40, isr::irq_stub_40 as *const ());
    set!(41, isr::irq_stub_41 as *const ());
    set!(42, isr::irq_stub_42 as *const ());
    set!(43, isr::irq_stub_43 as *const ());
    set!(44, isr::irq_stub_44 as *const ());
    set!(45, isr::irq_stub_45 as *const ());
    set!(46, isr::irq_stub_46 as *const ());
    set!(47, isr::irq_stub_47 as *const ());
    // ... (Остальные обработчики)
}

/// Разрешение аппаратных прерываний (sti)
pub fn interrupts_enable() {
    unsafe { core::arch::asm!("sti", options(nostack, preserves_flags)); }
}

/// Запрет аппаратных прерываний (cli)
pub fn interrupts_disable() {
    unsafe { core::arch::asm!("cli", options(nostack, preserves_flags)); }
}