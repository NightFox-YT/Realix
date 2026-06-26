use core::arch::asm;

// Флаги type_attr для IDT
pub const INTERRUPT_GATE: u8 = 0x8E; // P=1, DPL=0, Type=0xE (32-bit Interrupt Gate)
pub const TRAP_GATE: u8      = 0x8F; // P=1, DPL=0, Type=0xF (32-bit Trap Gate)

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_high: u16,
}

impl IdtEntry {
    const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            zero: 0,
            type_attr: 0,
            offset_high: 0,
        }
    }
}

// 256 записей для всех векторов прерываний (0x00–0xFF)
// Выравнивание по 8 байт обязательно: LIDT требует выровненную таблицу
#[repr(align(8))]
struct IdtTable([IdtEntry; 256]);

static mut IDT: IdtTable = IdtTable([IdtEntry::empty(); 256]);

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u32,
}

// Загружаем IDT через LIDT
pub fn init() {
    unsafe {
        // Инициализируем ВСЕ 256 векторов обработчиком по умолчанию,
        // чтобы spurious-прерывания (IRQ7/IRQ15) не вызывали #GP.
        fill_default();
        let ptr = IdtPtr {
            limit: (core::mem::size_of::<IdtTable>() - 1) as u16,
            base: IDT.0.as_ptr() as u32,
        };
        asm!("lidt [{}]", in(reg) &ptr, options(nostack));
    }
}

// Заполняет все 256 векторов IDT указанным обработчиком
pub fn fill_default() {
    unsafe {
        // Прямой каст через указатель, без transmute
        let addr = crate::interrupts::default_handler as *const () as u32;
        for i in 0..256 {
            IDT.0[i] = IdtEntry {
                offset_low: (addr & 0xFFFF) as u16,
                selector: 0x08,
                zero: 0,
                type_attr: INTERRUPT_GATE,
                offset_high: ((addr >> 16) & 0xFFFF) as u16,
            };
        }
    }
}

// Устанавливает обработчик для указанного вектора прерывания
pub fn set_handler(interrupt: u8, handler: unsafe extern "C" fn(), flags: u8) {
    unsafe {
        let addr = handler as u32;
        IDT.0[interrupt as usize] = IdtEntry {
            offset_low: (addr & 0xFFFF) as u16,
            selector: 0x08, // Kernel Code Segment
            zero: 0,
            type_attr: flags,
            offset_high: ((addr >> 16) & 0xFFFF) as u16,
        };
    }
}