// © Realix > GDT
// (25.06.26) v0.08
// ================

use core::arch::naked_asm;
use core::mem::size_of;
use core::ptr::addr_of;

const GDT_SIZE: usize = 6;

#[allow(dead_code)]
pub mod access {
    pub const PRESENT:         u8 = 1 << 7;
    pub const RING0:           u8 = 0 << 5;
    pub const RING3:           u8 = 3 << 5;
    pub const SYSTEM:          u8 = 1 << 4;
    pub const EXECUTABLE:      u8 = 1 << 3;
    pub const DIRECTION:       u8 = 1 << 2;
    pub const READ_WRITE_ABLE: u8 = 1 << 1;
    pub const ACCESSED:        u8 = 1 << 0;
    pub const TSS_AVAILABLE:   u8 = 0x89; // P=1 (0x80) | DPL=0 | System | Type=0x09
    pub const TSS_BUSY:        u8 = 0x8B; // P=1 (0x80) | DPL=0 | System | Type=0x0B
}

#[allow(dead_code)]
pub mod gran {
    pub const GRAN_4K:    u8 = 1 << 7;
    pub const BIT32_MODE: u8 = 1 << 6;
    pub const LONG_MODE:  u8 = 1 << 5;
}

#[allow(dead_code)]
pub mod selector {
    pub const KERNEL_CODE: u16 = 0x08;
    pub const KERNEL_DATA: u16 = 0x10;
    pub const USER_CODE:   u16 = 0x18;
    pub const USER_DATA:   u16 = 0x20;
    pub const TSS_SEL:     u16 = 0x28;
}

#[repr(C, packed)]
pub struct Tss {
    pub prev_tss: u32,
    pub esp0:     u32,
    pub ss0:      u32,
    pub esp1:     u32,
    pub ss1:      u32,
    pub esp2:     u32,
    pub ss2:      u32,
    pub cr3:      u32,
    pub eip:      u32,
    pub eflags:   u32,
    pub eax:      u32,
    pub ecx:      u32,
    pub edx:      u32,
    pub ebx:      u32,
    pub esp:      u32,
    pub ebp:      u32,
    pub esi:      u32,
    pub edi:      u32,
    pub es:       u32,
    pub cs:       u32,
    pub ss:       u32,
    pub ds:       u32,
    pub fs:       u32,
    pub gs:       u32,
    pub ldt:      u32,
    pub trap:     u16,
    pub iomap_base: u16,
    // Padding: 32-битная TSS должна быть минимум 104 байта (limit ≥ 0x67).
    // Без этого LTR генерирует #GP.
    pub _pad:     u32,
}

pub static mut TSS: Tss = Tss {
    prev_tss: 0,
    esp0: 0,
    ss0: selector::KERNEL_DATA as u32,
    esp1: 0, ss1: 0,
    esp2: 0, ss2: 0,
    cr3: 0,
    eip: 0, eflags: 0x0202,
    eax: 0, ecx: 0, edx: 0, ebx: 0,
    esp: 0, ebp: 0, esi: 0, edi: 0,
    es: 0, cs: 0, ss: 0, ds: 0, fs: 0, gs: 0,
    ldt: 0,
    trap: 0,
    iomap_base: 0xFFFF,
    _pad: 0,
};

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GdtDescriptor {
    limit_low:   u16,
    base_low:    u16,
    base_mid:    u8,
    access_byte: u8,
    gran:        u8,
    base_high:   u8,
}

impl GdtDescriptor {
    pub const fn null() -> Self {
        Self { limit_low: 0, base_low: 0, base_mid: 0, access_byte: 0, gran: 0, base_high: 0 }
    }

    pub const fn new(base: u32, limit: u32, access_byte: u8, flags: u8) -> Self {
        Self {
            limit_low: (limit & 0x0000FFFF) as u16,
            base_low:  (base  & 0x0000FFFF) as u16,
            base_mid:  ((base >> 16) & 0xFF) as u8,
            access_byte,
            gran: (flags & 0xF0) | (((limit >> 16) & 0x0F) as u8),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base:  u32,
}

// Naked-функция: полностью ручной asm, без вмешательства компилятора в стек.
// Принимает указатель на GdtPointer в регистре eax через соглашение о вызовах.
#[unsafe(naked)]
#[no_mangle]
unsafe extern "C" fn gdt_flush(ptr: *const GdtPointer) {
    unsafe {
        naked_asm!(
            // ptr лежит на стеке как аргумент функции (по соглашению cdecl)
            // Загружаем GDT через LGDT
            "mov eax, [esp+4]",     // eax = ptr (первый аргумент cdecl)
            "lgdt [eax]",

            // Обновляем сегментные регистры данных
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",

            // Сброс CS через дальний возврат
            "push 0x08",
            "lea eax, [2f]",
            "push eax",
            "retf",
            "2:",

            // Загружаем TSS
            "mov ax, 0x28",
            "ltr ax",

            "ret",
        );
    }
}

pub fn init() {
    use access::*;
    use gran::*;

    unsafe {
        GDT[0] = GdtDescriptor::null();

        GDT[1] = GdtDescriptor::new(
            0x00000000, 0x000FFFFF,
            PRESENT | RING0 | SYSTEM | EXECUTABLE | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        GDT[2] = GdtDescriptor::new(
            0x00000000, 0x000FFFFF,
            PRESENT | RING0 | SYSTEM | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        GDT[3] = GdtDescriptor::new(
            0x00000000, 0x000FFFFF,
            PRESENT | RING3 | SYSTEM | EXECUTABLE | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        GDT[4] = GdtDescriptor::new(
            0x00000000, 0x000FFFFF,
            PRESENT | RING3 | SYSTEM | READ_WRITE_ABLE,
            GRAN_4K | BIT32_MODE,
        );

        let tss_base = addr_of!(TSS) as u32;
        let tss_limit = (core::mem::size_of::<Tss>() - 1) as u32;
        GDT[5] = GdtDescriptor::new(
            tss_base, tss_limit,
            PRESENT | TSS_AVAILABLE,
            0x00,
        );

        GDT_POINTER.limit = (size_of::<[GdtDescriptor; GDT_SIZE]>() - 1) as u16;
        GDT_POINTER.base = addr_of!(GDT) as u32;

        gdt_flush(&raw const GDT_POINTER);
    }
}

#[used]
static mut GDT: [GdtDescriptor; GDT_SIZE] = [GdtDescriptor::null(); GDT_SIZE];
#[used]
static mut GDT_POINTER: GdtPointer = GdtPointer { limit: 0, base: 0 };