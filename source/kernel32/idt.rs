use core::arch::asm;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_high: u16,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    zero: 0,
    type_attr: 0,
    offset_high: 0,
}; 256];

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u32,
}

pub fn init() {
    unsafe {
        let ptr = IdtPtr {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: IDT.as_ptr() as u32,
        };
        asm!("lidt [{}]", in(reg) &ptr);
    }
}

pub fn set_handler(interrupt: u8, handler: unsafe extern "C" fn(), flags: u8) {
    unsafe {
        let addr = handler as u32;
        IDT[interrupt as usize] = IdtEntry {
            offset_low: (addr & 0xFFFF) as u16,
            selector: 0x08,
            zero: 0,
            type_attr: flags,
            offset_high: ((addr >> 16) & 0xFFFF) as u16,
        };
    }
}
