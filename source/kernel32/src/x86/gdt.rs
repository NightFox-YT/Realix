// © Realix > GDT
// (01.07.26) v0.08
// ================

// Подключение функций
use core::mem::size_of;

// Константы GDT
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
const GDT_SIZE: usize = 6;

// Коснтанты флагов Access Byte
#[allow(dead_code)]
pub mod access {
    pub const PRESENT:           u8 = 1 << 7; // Флаг использования дескриптора
    pub const RING0:             u8 = 0 << 5; // Кольцо 0 (Ядро)
    pub const RING3:             u8 = 3 << 5; // Кольцо 3 (ПО)
    pub const SYSTEM:            u8 = 1 << 4; // Обычный сегмент (не системный)
    pub const EXECUTABLE:        u8 = 1 << 3; // Исполняемый (code)
    pub const RESERVE_DIRECTION: u8 = 1 << 2; // Направление (Обратное для стека)
    pub const READ_WRITE_ABLE:   u8 = 1 << 1; // |-|
    pub const ACCESSED:          u8 = 1 << 0; // Флаг использования процессором
    pub const TSS_AVAILABLE_32:  u8 = 0b1001; // Тип дескриптора для TSS
}

// Константы флагов Granularity
#[allow(dead_code)]
pub mod granularity {
    pub const GRAN_4K:    u8 = 1 << 7; // Увеличенный лимит (*4KB)
    pub const BIT32_MODE: u8 = 1 << 6; // Дескриптор для 32-битного защищённого режима
    pub const LONG_MODE:  u8 = 1 << 5; // Дескриптор для 64-битного режима (0 для 32-бит)
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GdtDescriptor {
    limit_low:   u16,
    base_low:    u16,
    base_mid:    u8,
    access_byte: u8,
    granularity: u8,  // Флаги (4 бита) + Лимит (Старшие 4 бита)
    base_high:   u8, 
}

impl GdtDescriptor {
    /// Обязательный Null дексриптор
    pub const fn null() -> Self {
        Self {
            limit_low: 0, base_low: 0,
            base_mid: 0, access_byte: 0,
            granularity: 0, base_high: 0,
        }
    }

    /// Создание дескриптора по плоской модели памяти
    /// (Limit представляется 20 битами, т.к. нет такого типа данных, взято u32)
    pub const fn new(base: u32, limit: u32, access_byte: u8, flags: u8) -> Self {
        Self {
            limit_low: (limit & 0x0000FFFF) as u16,
            base_low:  (base  & 0x0000FFFF) as u16,

            // Сдвигаем вправо до 16 верхних бит, берём только нижние 8 бит
            base_mid:  ((base >> 16) & 0xFF) as u8,
            access_byte,

            // Берём флаги (верхние 4 бита) + Лимит (верхние 4 бита из 20)
            granularity: (flags & 0xF0) | (((limit >> 16) & 0x0F) as u8),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

// Структура-указатель для инструкции LGDT
#[repr(C, packed)]
struct GdtPointer {
    limit: u16,  // Размер GDT (байт)
    base:  u32,  // Линейный адрес GDT
}

#[repr(C, packed)]
pub struct TaskStateSegment {
    prev_tss: u32,

    // Стек и селектор стека Ring0, Ring1, Ring2
    // (Используется при прерывании из User Code)
    esp0: u32, ss0: u32,
    esp1: u32, ss1: u32,
    esp2: u32, ss2: u32,

    cr3: u32, eip: u32, eflags: u32,
    eax: u32, ecx: u32, edx: u32, ebx: u32,
    esp: u32, ebp: u32, esi: u32, edi: u32,
    es: u32,  cs: u32,  ss: u32,  ds: u32,
    fs: u32, gs: u32,
    ldt: u32,
    trap: u16,
    iomap_base: u16,
}

impl TaskStateSegment {
    /// Создание пустого TSS
    pub const fn new() -> Self {
        TaskStateSegment {
            prev_tss: 0, esp0: 0, ss0: 0, esp1: 0, ss1: 0,
            esp2: 0, ss2: 0, cr3: 0, eip: 0, eflags: 0,
            eax: 0, ecx: 0, edx: 0, ebx: 0, esp: 0, ebp: 0,
            esi: 0, edi: 0, es: 0, cs: 0, ss: 0, ds: 0,
            fs: 0, gs: 0, ldt: 0, trap: 0,
            iomap_base: size_of::<TaskStateSegment>() as u16,
        }
    }

    /// Обновление стека ядра (Вызывать при переключении процессов)
    pub fn set_kernel_stack(&mut self, stack_top: u32) {
        self.esp0 = stack_top;
    }
}

/// "Перезагрузка" GDT
#[no_mangle]
unsafe fn gdt_flush(gdt_pointer: *const GdtPointer) {
    core::arch::asm!(
        // Загружаем указатель на GDT
        "lgdt [{pointer}]",

        // Перезаписываем сегментные регистры данных на селектор Kernel Data
        "mov {ax:x}, 0x10",
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
        pointer = in(reg) gdt_pointer,
        ax  = out(reg) _,
        tmp = out(reg) _,
        options(nostack, preserves_flags),
    );
}

// > "Перезагрузка" TSS
unsafe fn tss_flush(tss_selector: u16) {
    core::arch::asm!(
        "ltr {sel:x}",
        sel = in(reg) tss_selector,
        options(nostack, preserves_flags),
    );
}

// > Инициализация GDT
pub fn init() {
    use access::*;
    use granularity::*;

    unsafe {
        // Настройка TSS с "заглушкой" до нормального аллокатора
        TSS.ss0 = KERNEL_DATA_SELECTOR as u32;
        TSS.set_kernel_stack(0x90000);

        // Null Descriptor (обязателен по спецификации x86)
        GDT[0] = GdtDescriptor::null();

        // (Ring 0) Kernel Code
        GDT[1] = GdtDescriptor::new(
            0x0,
            0x000FFFFF,
            PRESENT | RING0 | SYSTEM | EXECUTABLE | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        // (Ring 0) Kernel Data
        GDT[2] = GdtDescriptor::new(
            0x0,
            0x000FFFFF,
            PRESENT | RING0 | SYSTEM | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        // (Ring3) User Code
        GDT[3] = GdtDescriptor::new(
            0x0,
            0x000FFFFF,
            PRESENT | RING3 | SYSTEM | EXECUTABLE | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        // (Ring3) User Data
        GDT[4] = GdtDescriptor::new(
            0x0,
            0x000FFFFF,
            PRESENT | RING3 | SYSTEM | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        // (TSS)
        GDT[5] = GdtDescriptor::new(
            &raw const TSS as u32,
            (size_of::<TaskStateSegment>() - 1) as u32,
            PRESENT | RING0 | TSS_AVAILABLE_32,
            0,
        );

        // Заполняем указатель GDT
        GDT_POINTER.limit = (size_of::<[GdtDescriptor; GDT_SIZE]>() - 1) as u16;
        GDT_POINTER.base = &raw const GDT as u32;

        // Загружаем GDT и TSS
        gdt_flush(&raw const GDT_POINTER);
        tss_flush(0x28);
    }
}

// Занимаем место в памяти для GDT и TSS
#[used]
static mut GDT: [GdtDescriptor; GDT_SIZE] = [GdtDescriptor::null(); GDT_SIZE];
static mut GDT_POINTER: GdtPointer = GdtPointer { limit: 0, base: 0 };
static mut TSS: TaskStateSegment = TaskStateSegment::new();
