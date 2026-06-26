// STEP 4g: inline IDT + inline default_handler (никаких transmute)
// Обработчик определён прямо здесь, адрес получается через as u32.
// Это идентично рабочему TEST C3.

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
#[path = "drivers/gdt.rs"]
mod gdt;
#[path = "drivers/vga.rs"]
mod vga;
mod interrupts;
mod keyboard;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry { offset_low: u16, selector: u16, zero: u8, type_attr: u8, offset_high: u16 }
#[repr(align(8))]
struct IdtTable([IdtEntry; 256]);
static mut IDT: IdtTable = IdtTable([IdtEntry { offset_low: 0, selector: 0, zero: 0, type_attr: 0, offset_high: 0 }; 256]);
#[repr(C, packed)]
struct IdtPtr { limit: u16, base: u32 }

// ГОЛЫЙ inline-обработчик (без внешних зависимостей)
#[unsafe(naked)]
unsafe extern "C" fn my_default_handler() {
    core::arch::naked_asm!(
        "push eax",
        "mov al, 0x20",
        "out 0x20, al",
        "pop eax",
        "iret",
    );
}

fn delay_with_progress(label: &str) {
    vga::print_str(label, vga::Color::LightGray);
    for c in b'1'..=b'9' {
        vga::put_char(c as u8, vga::Color::LightGray);
        for _ in 0..8_000_000u32 {
            unsafe { asm!("nop", options(nomem, preserves_flags)); }
        }
    }
    vga::print_str(" OK\n", vga::Color::Green);
}

#[link_section = ".text.entry"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe { asm!("cli"); }

    vga::clear_screen();
    vga::print_str("STEP 4g: inline handler (no transmute)\n", vga::Color::Cyan);

    // [1] GDT
    vga::print_str("[1] GDT...", vga::Color::Yellow);
    gdt::init();
    unsafe { asm!("cli"); }
    vga::print_str(" OK\n", vga::Color::Green);
    delay_with_progress("    wait: ");

    // [2] IDT — прямой каст as u32, никаких transmute
    vga::print_str("[2] IDT...", vga::Color::Yellow);
    unsafe {
        let addr = my_default_handler as u32;
        for i in 0..=255u8 {
            IDT.0[i as usize] = IdtEntry {
                offset_low: (addr & 0xFFFF) as u16,
                selector: 0x08, zero: 0, type_attr: 0x8E,
                offset_high: ((addr >> 16) & 0xFFFF) as u16,
            };
        }
        let ptr = IdtPtr {
            limit: (core::mem::size_of::<IdtTable>() - 1) as u16,
            base: IDT.0.as_ptr() as u32,
        };
        asm!("lidt [{}]", in(reg) &ptr, options(nostack));
    }
    vga::print_str(" OK\n", vga::Color::Green);
    delay_with_progress("    wait: ");

    // [3] PIC
    vga::print_str("[3] PIC...", vga::Color::Yellow);
    interrupts::init_pic_mask(0xFE);
    vga::print_str(" OK\n", vga::Color::Green);
    delay_with_progress("    wait: ");

    // [4] STI
    vga::print_str("[4] STI...", vga::Color::Yellow);
    unsafe { asm!("sti"); }
    vga::print_str(" OK\n", vga::Color::Green);
    delay_with_progress("    wait: ");

    // [5] HLT forever
    vga::print_str("[5] HLT (forever)\n", vga::Color::Green);
    loop { unsafe { asm!("hlt"); } }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop { unsafe { asm!("hlt"); } } }