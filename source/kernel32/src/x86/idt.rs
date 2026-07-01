// © Realix > IDT
// (25.06.26) v0.07
// ================

// Подключение функций
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
use crate::x86::gdt;
use crate::x86::isr;

// Константы
const IDT_SIZE: usize = 256;
const IDT_GATE_32BIT_INT: u8 = 0x8E;

// Дескриптор с заданным порядком полей без выравнивания
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct InterruptDescriptor {
    base_low: u16,
    selector: u16,
    reserved: u8,
    flags: u8,
    base_high: u16,
}

impl InterruptDescriptor {
    // > Обработчик прерывания отсутствует
    pub const fn missing() -> Self {
        InterruptDescriptor {
            base_low: 0,
            selector: 0,
            reserved: 0,
            flags: 0,
            base_high: 0,
        }
    }

    // > Установить обработчик для дескриптора прерывания
    pub fn set_handler(&mut self, handler_addr: u32, selector: u16, flags: u8) {
        self.base_low = (handler_addr & 0xFFFF) as u16;
        self.selector = selector;
        self.reserved = 0 as u8;
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

// Занимаем место в памяти для IDT
static mut IDT: [InterruptDescriptor; IDT_SIZE] = [InterruptDescriptor::missing(); IDT_SIZE];

// > Функция инициализации IDT
pub fn init() {
    unsafe { set_exception_handlers(&mut *addr_of_mut!(IDT)); }
    
    // Формируем указатель на IDT
    let idt_pointer: IdtPointer = IdtPointer {
        limit: (core::mem::size_of::<[InterruptDescriptor; 256]>() - 1) as u16,
        base: addr_of!(IDT) as u32,
    };

    unsafe {
        // Загружаем таблицу в процессор
        core::arch::asm!(
            "lidt [{}]", 
            in(reg) &idt_pointer, 
            options(readonly, nostack, preserves_flags),
        );
    }
}

// > Регистрирует обработчики исключений CPU (вектора 0-19)
fn set_exception_handlers(idt_addr: &mut [InterruptDescriptor; IDT_SIZE]) {
    // > Макрос для установки прерывания
    macro_rules! set {
        ($vec:expr, $handler:expr) => {
            idt_addr[$vec].set_handler($handler as u32, gdt::KERNEL_CODE_SELECTOR, IDT_GATE_32BIT_INT);
        };
    }

    // Установка обработчиков прерываний
    set!(0, isr::isr_divide_by_zero as *const ());
    set!(1, isr::isr_debug as *const ());
    set!(2, isr::isr_non_maskable_interrupt as *const ());
    set!(3, isr::isr_breakpoint as *const ());
    set!(4, isr::isr_overflow as *const ());
    set!(5, isr::isr_bound_range_exceeded as *const ());
    set!(6, isr::isr_invalid_opcode as *const ());
    set!(7, isr::isr_device_not_available as *const ());
    set!(8, isr::isr_double_fault as *const ());
    set!(9, isr::isr_coprocessor_segment_overrun as *const ());
    set!(10, isr::isr_invalid_tss as *const ());
    set!(11, isr::isr_segment_not_present as *const ());
    set!(12, isr::isr_stack_segment_fault as *const ());
    set!(13, isr::isr_general_protection_fault as *const ());
    set!(14, isr::isr_page_fault as *const ());
    // ... (вектор 15 зарезервирован под Intel, обработчик не генерируется)
    set!(16, isr::isr_x86_floating_point_exception as *const ());
    set!(17, isr::isr_alignment_check as *const ());
    set!(18, isr::isr_machine_check as *const ());
    set!(19, isr::isr_simd_floating_point_exception as *const ());
    // ... (Остальные обработчики)
}