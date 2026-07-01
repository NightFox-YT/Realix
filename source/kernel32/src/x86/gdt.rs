// © Realix > GDT
// (25.06.26) v0.07
// ================

// Подключение функций
use core::mem::size_of;
use core::ptr::addr_of;

// Константы GDT
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
const GDT_SIZE: usize = 5;

// Коснтанты флагов Access Byte
#[allow(dead_code)]
pub mod access {
    pub const PRESENT:         u8 = 1 << 7; // Состояние использования дескриптора
    pub const RING0:           u8 = 0 << 5; // Кольцо 0 (Ядро)
    pub const RING3:           u8 = 3 << 5; // Кольцо 3 (ПО)
    pub const SYSTEM:          u8 = 1 << 4; // Обычный сегмент (не системный)
    pub const EXECUTABLE:      u8 = 1 << 3; // Исполняемый (code)
    pub const DIRECTION:       u8 = 1 << 2; // Направление (Обратное для стека)
    pub const READ_WRITE_ABLE: u8 = 1 << 1; // |-|
    pub const ACCESSED:        u8 = 1 << 0; // Состояние использования процессором
}

// Константы флагов Granularity
#[allow(dead_code)]
pub mod gran {
    pub const GRAN_4K:    u8 = 1 << 7; // Максимальный лимит в 4KB
    pub const BIT32_MODE: u8 = 1 << 6; // Дескриптор для 32-битного защищённого режима
    pub const LONG_MODE:  u8 = 1 << 5; // Дескриптор для 64-битного режима (0 для 32-бит)
}

// Дексриптор с заданным порядком полей (без выравнивания)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GdtDescriptor {
    limit_low:   u16,  // Нижние 16 бит
    base_low:    u16,  // Нижние 16 бит
    base_mid:    u8,   // Средние 8 бит
    access_byte: u8,   // |-|
    gran:        u8,   // Флаги (4 бита) + Лимит (Старшие 4 бита)
    base_high:   u8,   // Старшие 8 бат
}

impl GdtDescriptor {
    // Обязательный Null дексриптор (Создаётся во время компиляции)
    pub const fn null() -> Self {
        Self {
            limit_low: 0, base_low: 0,
            base_mid: 0, access_byte: 0,
            gran: 0, base_high: 0,
        }
    }

    // > Создание дескриптора (Плоская модель памяти)
    // Limit представляется 20 битами (нет такого типа данных, поэтому взято u32)
    pub const fn new(base: u32, limit: u32, access_byte: u8, flags: u8) -> Self {
        Self {
            limit_low: (limit & 0x0000FFFF) as u16,
            base_low:  (base  & 0x0000FFFF) as u16,

            // Сдвигаем вправо, берём только верхние 8 бит из нижних 16 бит
            base_mid:  ((base >> 16) & 0xFF) as u8,
            access_byte,

            // Берём флаги (верхние 4 бита) и лимит (верхние 4 бита из 20)
            gran: (flags & 0xF0) | (((limit >> 16) & 0x0F) as u8),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

// Структура-указатель для инструкции LGDT
// (Сохраняет заданный порядок полей без выравнивания)
#[repr(C, packed)]
struct GdtPointer {
    limit: u16,   // Размер GDT (байт)
    base:  u32,   // Линейный адрес GDT
}

// > "Перезагрузка" GDT
#[no_mangle]
unsafe fn gdt_flush(ptr: *const GdtPointer) {
    core::arch::asm!(
        "lgdt [{ptr}]",      // Загружаем указатель на GDT
        "mov {ax:x}, 0x10",  // Временно храним селектор Kernel Data

        // Перезаписываем сегментные регистры данных
        "mov ds, {ax:x}",
        "mov es, {ax:x}",
        "mov fs, {ax:x}",
        "mov gs, {ax:x}",
        "mov ss, {ax:x}",

        // Сброс сегментного регистра кода через дальний возврат + стек
        "push 0x08",
        "lea {tmp}, [2f]",
        "push {tmp}",
        "retf",
        "2:",
        ptr = in(reg) ptr,
        ax  = out(reg) _,
        tmp = out(reg) _,
        options(nostack, preserves_flags),
    );
}

// > Инициализация GDT
pub fn init() {
    use access::*;
    use gran::*;

    unsafe {
        // Null Descriptor (обязателен по спецификации x86)
        GDT[0] = GdtDescriptor::null();

        // (Ring 0) Kernel Code
        GDT[1] = GdtDescriptor::new(
            0x00000000,
            0x000FFFFF,
            PRESENT | RING0 | SYSTEM | EXECUTABLE | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        // (Ring 0) Kernel Data
        GDT[2] = GdtDescriptor::new(
            0x00000000,
            0x000FFFFF,
            PRESENT | RING0 | SYSTEM | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        // (Ring3) User Code
        GDT[3] = GdtDescriptor::new(
            0x00000000,
            0x000FFFFF,
            PRESENT | RING3 | SYSTEM | EXECUTABLE | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        // (Ring3) User Data
        GDT[4] = GdtDescriptor::new(
            0x00000000,
            0x000FFFFF,
            PRESENT | RING3 | SYSTEM | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        // Заполняем указатель GDT
        GDT_POINTER.limit = (size_of::<[GdtDescriptor; GDT_SIZE]>() - 1) as u16;
        GDT_POINTER.base = addr_of!(GDT) as u32;

        // Загружаем GDT
        gdt_flush(&raw const GDT_POINTER);
    }
}

// Занимаем место в памяти для GDT
#[used]
static mut GDT: [GdtDescriptor; GDT_SIZE] = [GdtDescriptor::null(); GDT_SIZE];
#[used]
static mut GDT_POINTER: GdtPointer = GdtPointer { limit: 0, base: 0 };