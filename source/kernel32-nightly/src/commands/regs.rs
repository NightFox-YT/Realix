// © Realix > Command: Registers
// (07.09.26) v0.12
// ================
// ❗️ Показывает регистры на момент вызова `snapshot()` (уже с учётом работы shell)

// Подключение функций
use core::arch::asm;
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Снимок регистров общего назначения, сегментов и флагов
struct Snapshot {
    eax: u32, ebx: u32, ecx: u32, edx: u32,
    esi: u32, edi: u32, ebp: u32, esp: u32,
    cs: u32, ds: u32, es: u32, ss: u32,
    eflags: u32,
}

/// Считывание снимка регистров процессора
/// ❗️ Разбито на несколько `asm!`-блоков: "reg"-класс допускает лишь ~6
///    одновременно живых регистров общего назначения на один блок
fn snapshot() -> Snapshot {
    let (eax, ebx, ecx, edx): (u32, u32, u32, u32);
    unsafe {
        asm!(
            "mov {eax}, eax", "mov {ebx}, ebx", "mov {ecx}, ecx", "mov {edx}, edx",
            eax = out(reg) eax, ebx = out(reg) ebx, ecx = out(reg) ecx, edx = out(reg) edx,
            options(nomem, nostack, preserves_flags),
        );
    }

    let (esi, edi, ebp, esp): (u32, u32, u32, u32);
    unsafe {
        asm!(
            "mov {esi}, esi", "mov {edi}, edi", "mov {ebp}, ebp", "mov {esp}, esp",
            esi = out(reg) esi, edi = out(reg) edi, ebp = out(reg) ebp, esp = out(reg) esp,
            options(nomem, nostack, preserves_flags),
        );
    }

    let (cs, ds, es, ss): (u32, u32, u32, u32);
    unsafe {
        asm!(
            "mov {cs}, cs", "mov {ds}, ds", "mov {es}, es", "mov {ss}, ss",
            cs = out(reg) cs, ds = out(reg) ds, es = out(reg) es, ss = out(reg) ss,
            options(nomem, nostack, preserves_flags),
        );
    }

    let eflags: u32;
    unsafe {
        asm!("pushfd", "pop {0}", out(reg) eflags, options(preserves_flags));
    }

    Snapshot { eax, ebx, ecx, edx, esi, edi, ebp, esp, cs, ds, es, ss, eflags }
}

/// Команда вывода снимка регистров
pub fn run() {
    let regs: Snapshot = snapshot();
    let mut buf: [u8; 10] = [0u8; 10];

    vga::print_line("- Registers:\n", Color::LightGray);

    print_named_hex("EAX", regs.eax, &mut buf);
    print_named_hex("EBX", regs.ebx, &mut buf);
    print_named_hex("ECX", regs.ecx, &mut buf);
    print_named_hex("EDX", regs.edx, &mut buf);
    vga::new_line();

    print_named_hex("ESI", regs.esi, &mut buf);
    print_named_hex("EDI", regs.edi, &mut buf);
    print_named_hex("EBP", regs.ebp, &mut buf);
    print_named_hex("ESP", regs.esp, &mut buf);
    vga::new_line();

    print_named_hex("CS", regs.cs, &mut buf);
    print_named_hex("DS", regs.ds, &mut buf);
    print_named_hex("ES", regs.es, &mut buf);
    print_named_hex("SS", regs.ss, &mut buf);
    vga::new_line();

    print_named_hex("EFLAGS", regs.eflags, &mut buf);
    vga::new_line();
}

/// Вывод пары "label = value" с отступом справа
fn print_named_hex(label: &str, value: u32, buf: &mut [u8; 10]) {
    vga::print_line(label, Color::LightGray);
    vga::print_line(" = ", Color::LightGray);
    vga::print_line(utils::u32_to_hex_str(value, buf), Color::White);
    vga::print_line("  ", Color::LightGray);
}
