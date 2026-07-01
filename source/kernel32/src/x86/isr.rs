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
    pub fn exc_divide_by_zero();          pub fn exc_debug();
    pub fn exc_non_maskable_interrupt();  pub fn exc_breakpoint();
    pub fn exc_overflow();                pub fn exc_bound_range_exceeded();
    pub fn exc_invalid_opcode();          pub fn exc_device_not_available();
    pub fn exc_double_fault();            pub fn exc_coprocessor_segment_overrun();
    pub fn exc_invalid_tss();             pub fn exc_segment_not_present();
    pub fn exc_stack_segment_fault();     pub fn exc_general_protection_fault();
    pub fn exc_page_fault();              pub fn exc_x86_floating_point_exception();
    pub fn exc_alignment_check();         pub fn exc_machine_check();
    pub fn exc_simd_floating_point_exception();
}

// "Заглушка" на ассемблере (GAS синтаксис)
global_asm!(
    r#"
.code32

# > Макрос для исключений без error code (Пушим вместо него 0)
.macro EXC_NOERRCODE num, name
.global exc_\name
exc_\name:
    push 0
    push \num
    jmp exc_common_stub
.endm

# > Макрос для исключений с error code
.macro EXC_ERRCODE num, name
.global exc_\name
exc_\name:
    push \num
    jmp exc_common_stub
.endm

# Объявляем создание функций по вышенаписанному макросу
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

# > Общая точка входа для всех исключений
exc_common_stub:
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
