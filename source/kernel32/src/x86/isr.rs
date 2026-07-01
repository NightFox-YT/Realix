// © Realix > ISR (Int service routine)
// (01.07.26) v0.08
// ================

// Подключение функций
use core::arch::global_asm;
use crate::drivers::vga;

// Структура, в которой хранится информация с "заглушки" на ассемблере
// (Обратный порядок push в ассемблерной заглушке)
#[repr(C)]
pub struct Registers {
    pub ds: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32, // *Не используется (Указатель до push ds)
    pub edx: u32,
    pub ecx: u32,
    pub ebx: u32,
    pub eax: u32,
    pub int_num: u32,
    pub err_code: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
}

// Названия исключений 0-19 для вывода
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

// Общий обработчик для векторов прерываний 0-19
#[no_mangle]
pub fn isr_handler(regs: &Registers) {
    let name: &str = if (regs.int_num as usize) < EXCEPTION_NAMES.len() {
        EXCEPTION_NAMES[regs.int_num as usize]
    } else {
        "Unknown"
    };

    vga::print_str("[KERNEL PANIC] Realix got exception: ", vga::Color::Red);
    vga::print_str(name, vga::Color::Red);

    vga::new_line();
    vga::new_line();

    vga::print_str("! Registers:\n", vga::Color::Red);
    vga::print_reg_line("EAX", regs.eax);
    vga::print_reg_line("EBX", regs.ebx);
    vga::print_reg_line("ECX", regs.ecx);
    vga::print_reg_line("EDX", regs.edx);
    vga::print_reg_line("ESI", regs.esi);
    vga::print_reg_line("EDI", regs.edi);
    vga::print_reg_line("EBP", regs.ebp);
    vga::print_reg_line("EIP", regs.eip);
    vga::print_reg_line("CS",  regs.cs);
    vga::print_reg_line("DS",  regs.ds);
    vga::print_reg_line("EFLAGS", regs.eflags);
    vga::print_reg_line("INT_NUM", regs.int_num);
    vga::print_reg_line("ERR_CODE", regs.err_code);
    
    vga::new_line();
    vga::print_str("! System halted. Please reboot the machine.", vga::Color::Red);
    panic!();
}

// Объявляем ассемблерные метки публичными (имена совпадают с метками в global_asm!)
unsafe extern "C" {
    pub fn isr_divide_by_zero();
    pub fn isr_debug();
    pub fn isr_non_maskable_interrupt();
    pub fn isr_breakpoint();
    pub fn isr_overflow();
    pub fn isr_bound_range_exceeded();
    pub fn isr_invalid_opcode();
    pub fn isr_device_not_available();
    pub fn isr_double_fault();
    pub fn isr_coprocessor_segment_overrun();
    pub fn isr_invalid_tss();
    pub fn isr_segment_not_present();
    pub fn isr_stack_segment_fault();
    pub fn isr_general_protection_fault();
    pub fn isr_page_fault();
    // ... (вектор 15 зарезервирован под Intel)
    pub fn isr_x86_floating_point_exception();
    pub fn isr_alignment_check();
    pub fn isr_machine_check();
    pub fn isr_simd_floating_point_exception();
}

// "Заглушка" на ассемблере (GAS синтаксис)
global_asm!(
    r#"
.code32

# > Макрос для исключений без error code (Пушим вместо него 0)
.macro ISR_NOERRCODE num, name
.global isr_\name
isr_\name:
    push 0
    push \num
    jmp isr_common_stub
.endm

# > Макрос для исключений с error code
.macro ISR_ERRCODE num, name
.global isr_\name
isr_\name:
    push \num
    jmp isr_common_stub
.endm

# Объявляем создание функций по вышенаписанному макросу
ISR_NOERRCODE 0,  divide_by_zero
ISR_NOERRCODE 1,  debug
ISR_NOERRCODE 2,  non_maskable_interrupt
ISR_NOERRCODE 3,  breakpoint
ISR_NOERRCODE 4,  overflow
ISR_NOERRCODE 5,  bound_range_exceeded
ISR_NOERRCODE 6,  invalid_opcode
ISR_NOERRCODE 7,  device_not_available
ISR_ERRCODE   8,  double_fault
ISR_NOERRCODE 9,  coprocessor_segment_overrun
ISR_ERRCODE   10, invalid_tss
ISR_ERRCODE   11, segment_not_present
ISR_ERRCODE   12, stack_segment_fault
ISR_ERRCODE   13, general_protection_fault
ISR_ERRCODE   14, page_fault
ISR_NOERRCODE 16, x86_floating_point_exception
ISR_ERRCODE   17, alignment_check
ISR_NOERRCODE 18, machine_check
ISR_NOERRCODE 19, simd_floating_point_exception

# > Общая точка входа для всех обработчиков
isr_common_stub:
    # Сохранение eax, ecx, edx, ebx, esp, ebp, esi, edi
    pusha

    # Сохранение текущего сегмента данных (ds)
    xor eax, eax
    mov ax, ds
    push eax

    # Передача номера Kernel data selector
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    # Передача указателя на Registers первым аргументом
    push esp
    call isr_handler
    add esp, 4

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
    iret
    "#
);