// © Realix > IDT
// (25.06.26) v0.07
// ================

// Подключение функций
use core::ptr::addr_of;

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
#[repr(C, packed)]  // Структура сохраняет заданный порядок полей без выравнивания
pub struct IdtPointer {
    limit: u16,
    base: u32,
}

// Занимаем место в памяти для IDT
static mut IDT: [InterruptDescriptor; IDT_SIZE] = [InterruptDescriptor::missing(); IDT_SIZE];

// > Функция инициализации IDT
pub fn init() {
    // IDT[0].set_handler(divide_by_zero_handler as u32, 0x08, 0x8E);
    
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
