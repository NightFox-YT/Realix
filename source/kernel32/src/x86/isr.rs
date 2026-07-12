// © Realix > ISR (Int service routine)
// (12.07.26) v0.09
// ================

// Подключение функций
use core::arch::global_asm;

use crate::drivers::vga::{self, Color};
use crate::drivers::{keyboard, pit};
use crate::halt_loop;
use crate::x86::pic;

/// Состояние процессора, сформированное ассемблерной заглушкой.
/// (Порядок полей соответствует обратному порядку PUSH)
#[repr(C)]
pub struct Registers {
    pub ds: u32,

    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,  // *Не используется (Указатель во время pusha)

    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,

    pub int_num: u32,
    pub err_code: u32,

    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
}

const EXCEPTION_NAMES: [&str; 20] = [
    "Divide By Zero",
    "Debug",
    "Non Maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating-Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
];


/// Общий обработчик исключений CPU (0-19)
#[no_mangle]
pub extern "C" fn exc_handler(regs: &Registers) -> ! {
    // Гарантируем остановку прерываний
    // (Прерывания уже отключены в exc_common_stub через cli)

    let exception_name: &str = if (regs.int_num as usize) < EXCEPTION_NAMES.len() {
        EXCEPTION_NAMES[regs.int_num as usize]
    } else {
        "Unknown"
    };

    vga::print_line(
        "[KERNEL PANIC] Realix got exception: ",
        Color::Red,
    );
    vga::print_line(exception_name, Color::Red);
    vga::new_line();
    vga::new_line();

    vga::print_line("! Registers:\n", Color::Red);

    vga::print_reg_line("EAX", regs.eax);
    vga::print_reg_line("EBX", regs.ebx);
    vga::print_reg_line("ECX", regs.ecx);
    vga::print_reg_line("EDX", regs.edx);
    vga::print_reg_line("ESI", regs.esi);
    vga::print_reg_line("EDI", regs.edi);
    vga::print_reg_line("EBP", regs.ebp);
    vga::print_reg_line("EIP", regs.eip);
    vga::print_reg_line("CS", regs.cs);
    vga::print_reg_line("DS", regs.ds);
    vga::print_reg_line("EFLAGS", regs.eflags);
    vga::print_reg_line("INT_NUM", regs.int_num);
    vga::print_reg_line("ERR_CODE", regs.err_code);

    vga::new_line();
    vga::print_line(
        "! System halted. Please reboot the machine.",
        Color::Red,
    );

    crate::halt_loop()
}


/// Общий обработчик аппаратных IRQ.
#[no_mangle]
pub extern "C" fn irq_handler(regs: &Registers) {
    // Защита от вызова функции с неправильным аргументом
    if regs.int_num < 32 || regs.int_num > 47 {
        vga::print_line("[!] IRQ num in handler is incorrect!", Color::Red);
        halt_loop();
    }

    let irq: u8 = (regs.int_num - 32) as u8;

    // Обработка ложного прерывания (IRQ7/IRQ15)
    if pic::is_spurious(irq) {
        return; // Прерывания будут включены через iretd
    }

    match irq {
        0 => { pit::tick(); }
        1 => {
            let scancode = unsafe {
                crate::inb(keyboard::KEYBOARD_DATA_PORT)
            };
            keyboard::on_scancode(scancode);
        }
        _ => { /* Остальные IRQ сейчас замаскированы */ }
    }

    pic::send_eoi(irq);
    // Не включаем прерывания здесь - это сделает iretd при возврате,
    // который восстановит EFLAGS с установленным битом IF
}

// Объявляем ассемблерные метки публичными
unsafe extern "C" {
    pub fn exc_divide_by_zero();
    pub fn exc_debug();
    pub fn exc_non_maskable_interrupt();
    pub fn exc_breakpoint();
    pub fn exc_overflow();
    pub fn exc_bound_range_exceeded();
    pub fn exc_invalid_opcode();
    pub fn exc_device_not_available();
    pub fn exc_double_fault();
    pub fn exc_coprocessor_segment_overrun();
    pub fn exc_invalid_tss();
    pub fn exc_segment_not_present();
    pub fn exc_stack_segment_fault();
    pub fn exc_general_protection_fault();
    pub fn exc_page_fault();
    pub fn exc_x86_floating_point_exception();
    pub fn exc_alignment_check();
    pub fn exc_machine_check();
    pub fn exc_simd_floating_point_exception();

    pub fn irq_stub_32();
    pub fn irq_stub_33();
    pub fn irq_stub_34();
    pub fn irq_stub_35();
    pub fn irq_stub_36();
    pub fn irq_stub_37();
    pub fn irq_stub_38();
    pub fn irq_stub_39();
    pub fn irq_stub_40();
    pub fn irq_stub_41();
    pub fn irq_stub_42();
    pub fn irq_stub_43();
    pub fn irq_stub_44();
    pub fn irq_stub_45();
    pub fn irq_stub_46();
    pub fn irq_stub_47();
}

// "Заглушка" на ассемблере (GAS синтаксис)
global_asm!(
    r#"
.code32

# Исключение без аппаратного error code
.macro EXC_NOERRCODE num, name
.global exc_\name
exc_\name:
    push 0
    push \num
    jmp exc_common_stub
.endm

# Исключение с аппаратным error code
.macro EXC_ERRCODE num, name
.global exc_\name
exc_\name:
    push \num
    jmp exc_common_stub
.endm

# Аппаратные IRQ не имеют error code
.macro IRQ_STUB num
.global irq_stub_\num
irq_stub_\num:
    push 0
    push \num
    jmp irq_common_stub
.endm


EXC_NOERRCODE 0,  divide_by_zero
EXC_NOERRCODE 1,  debug
EXC_NOERRCODE 2,  non_maskable_interrupt
EXC_NOERRCODE 3,  breakpoint
EXC_NOERRCODE 4,  overflow
EXC_NOERRCODE 5,  bound_range_exceeded
EXC_NOERRCODE 6,  invalid_opcode
EXC_NOERRCODE 7,  device_not_available
EXC_ERRCODE   8,  double_fault
EXC_NOERRCODE 9,  coprocessor_segment_overrun
EXC_ERRCODE   10, invalid_tss
EXC_ERRCODE   11, segment_not_present
EXC_ERRCODE   12, stack_segment_fault
EXC_ERRCODE   13, general_protection_fault
EXC_ERRCODE   14, page_fault
EXC_NOERRCODE 16, x86_floating_point_exception
EXC_ERRCODE   17, alignment_check
EXC_NOERRCODE 18, machine_check
EXC_NOERRCODE 19, simd_floating_point_exception


IRQ_STUB 32
IRQ_STUB 33
IRQ_STUB 34
IRQ_STUB 35
IRQ_STUB 36
IRQ_STUB 37
IRQ_STUB 38
IRQ_STUB 39
IRQ_STUB 40
IRQ_STUB 41
IRQ_STUB 42
IRQ_STUB 43
IRQ_STUB 44
IRQ_STUB 45
IRQ_STUB 46
IRQ_STUB 47

# Общая точка входа исключений
exc_common_stub:
    cld

    # Сохранение eax, ecx, edx, ebx, esp, ebp, esi, edi
    pusha

    # Сохранение текущего сегмента данных (ds)
    xor eax, eax
    mov ax, ds
    push eax

    # Переключение на Kernel data selector
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    # Передача указателя на Registers первым аргументом
    push esp
    call exc_handler
    # exc_handler не возвращается, поэтому код ниже не выполняется

    # Восстанавливаем старый сегмент данных
    pop eax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    # Восстановление eax, ecx, edx, ebx, esp, ebp, esi, edi
    popa

    # Удаление err_code и int_num из стека
    add esp, 8
    iretd

# > Общая точка входа для IRQ-прерываний
irq_common_stub:
    cld
    
    # Сохранение eax, ecx, edx, ebx, esp, ebp, esi, edi
    pusha

    # Сохранение текущего сегмента данных (ds)
    xor eax, eax
    mov ax, ds
    push eax

    # Переключение на Kernel data selector
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    # Передача указателя на Registers первым аргументом
    push esp
    call irq_handler
    add esp, 4

    # Восстанавливаем исходные сегменты данных
    pop eax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    # Восстановление eax, ecx, edx, ebx, esp, ebp, esi, edi
    popa

    # Удаление int_num и искусственного err_code
    add esp, 8
    iretd
"#
);
