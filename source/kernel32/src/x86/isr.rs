// © Realix > x86: ISR (Interrupt Service Routine)
// (27.07.26) v0.1
// ================
// ❗️ Имена в `unsafe extern "C"` должны совпадать с метками в global_asm!

// Подключение функций
use core::arch::global_asm;

use crate::drivers::vga::{self, Color};
use crate::drivers::{keyboard, pit, mouse};
use crate::halt_loop;
use crate::x86::idt::{interrupts_disable, interrupts_enable};
use crate::x86::{gdt, pic};
use crate::utils::inb;

// Номера обрабатываемых IRQ
const IRQ_TIMER:    u8 = 0;
const IRQ_KEYBOARD: u8 = 1;
const IRQ_MOUSE:    u8 = 12;

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
pub extern "C" fn exc_handler(regs: &Registers) {
    // Гарантируем остановку прерываний
    interrupts_disable();

    let exception_name: &str = if (regs.int_num as usize) < EXCEPTION_NAMES.len() {
        EXCEPTION_NAMES[regs.int_num as usize]
    } else {
        "Unknown"
    };

    // Возвращаемые исключения (Breakpoint)
    if regs.int_num == 3 {
        vga::print_line("[#] Breakpoint, continuing:\n", Color::Yellow);
        vga::print_reg_line("EIP", regs.eip);

        interrupts_enable();
        return;
    }

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
    vga::print_line("! System halted, please, reboot the machine.\n", Color::Red);
    vga::print_line(
        "! If the problem repeats, report this screen to the developer: ",
        Color::Red
    );
    vga::print_line(
        "https://github.com/NightFox-YT/Realix/issues",
        Color::Red
    );

    halt_loop();
}


/// Общий обработчик аппаратных IRQ
#[no_mangle]
pub extern "C" fn irq_handler(regs: &Registers) {
    // Защита от вызова функции с неправильным аргументом
    // (Такое может быть вследствие ошибки в IRQ stub или IDT)
    if !(32..=47).contains(&regs.int_num) {
        vga::print_line("[!] IRQ num in handler is incorrect!", Color::Red);
        halt_loop();
    }

    let irq: u8 = (regs.int_num - 32) as u8;

    // Обработка ложного прерывания (IRQ7/IRQ15)
    if pic::is_spurious(irq) {
        interrupts_enable();
        return;
    }

    match irq {
        IRQ_TIMER => { pit::tick(); }
        IRQ_KEYBOARD => {
            let scancode: u8 = unsafe { inb(keyboard::KEYBOARD_DATA_PORT) };
            keyboard::on_scancode(scancode);
        }
        IRQ_MOUSE => {
            mouse::on_irq12();
        }
        _ => { /* Остальные IRQ сейчас замаскированы */ }
    }

    pic::send_eoi(irq);
    interrupts_enable();
}

// Объявляем ассемблерные метки публичными (имена совпадают с метками в global_asm!)
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

# > Исключение без аппаратного error code
# Добавляем искусственный нулевой код, чтобы структура стека была общей
.macro EXC_NOERRCODE num, name
.global exc_\name
exc_\name:
    push 0
    push \num
    jmp exc_common_stub
.endm

# > Исключение с аппаратным error code
# Процессор уже положил код ошибки в стек
.macro EXC_ERRCODE num, name
.global exc_\name
exc_\name:
    push \num
    jmp exc_common_stub
.endm

# > Аппаратные IRQ не имеют error code
.macro IRQ_STUB num
.global irq_stub_\num
irq_stub_\num:
    push 0
    push \num
    jmp irq_common_stub
.endm

# > Общая точка входа (Исключения и IRQ отличаются только обработчиком)
# К этому моменту в стеке уже лежат err_code и int_num
.macro COMMON_STUB name, handler
\name:
    cld

    # Сохранение eax, ecx, edx, ebx, esp, ebp, esi, edi
    pusha

    # Сохранение текущего сегмента данных (ds)
    xor eax, eax
    mov ax, ds
    push eax

    # Переключение на Kernel data selector
    mov ax, {kernel_data_sel}
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    # Передача указателя на Registers первым аргументом
    push esp
    call \handler
    add esp, 4

    # Восстанавливаем исходные сегменты данных
    pop eax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    # Восстановление eax, ecx, edx, ebx, esp, ebp, esi, edi
    popa

    # Удаление int_num и err_code из стека
    add esp, 8
    iretd
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

.global isr_stub_128
isr_stub_128:
    push 0
    push 128
    pusha
    xor eax, eax
    mov ax, ds
    push eax
    mov ax, {kernel_data_sel}
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    push esp
    call syscall_handler
    add esp, 4
    pop eax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    popa
    add esp, 8
    iretd

# > Точки входа: исключения CPU и аппаратные IRQ
COMMON_STUB exc_common_stub, exc_handler
COMMON_STUB irq_common_stub, irq_handler
"#,
    kernel_data_sel = const gdt::KERNEL_DATA_SELECTOR,
);

unsafe extern "C" {
    pub fn isr_stub_128();
}

#[no_mangle]
pub extern "C" fn syscall_handler(regs: &mut Registers) {
    match regs.eax {
        1 => {
            // SYS_PRINT_STRING
            let ptr = regs.esi as *const u8;
            if !ptr.is_null() {
                let mut i = 0;
                unsafe {
                    while *ptr.add(i) != 0 && i < 2048 {
                        vga::print_char(*ptr.add(i), Color::LightGray);
                        i += 1;
                    }
                }
            }
        }
        2 => {
            // SYS_PUTCHAR (EAX = 2, EDX = char, EBX = color)
            let color = match (regs.ebx & 0xFF) as u8 {
                1 => Color::LightBlue,
                2 => Color::LightGreen,
                3 => Color::LightCyan,
                4 => Color::LightRed,
                5 => Color::Pink,
                6 => Color::Yellow,
                7 => Color::White,
                _ => Color::LightGray,
            };
            vga::print_char((regs.edx & 0xFF) as u8, color);
        }
        3 => {
            // SYS_EXIT
            crate::rlx_loader::exit_to_kernel();
        }
        4 => {
            // SYS_READ_KEY — возвращает ASCII или специальные коды:
            // 0x1B = ESC, 0x08 = Backspace, 0x0D = Enter
            // 0xF1-0xF4 = F1-F4 (расширенные), 0x80/0x81 = Up/Down
            match keyboard::read_key() {
                keyboard::Key::Char(c) => regs.eax = c as u32,
                keyboard::Key::Escape  => regs.eax = 0x1B,
                keyboard::Key::Up      => regs.eax = 0x80,
                keyboard::Key::Down    => regs.eax = 0x81,
                keyboard::Key::F1      => regs.eax = 0xF1,
                keyboard::Key::F2      => regs.eax = 0xF2,
                keyboard::Key::F3      => regs.eax = 0xF3,
                keyboard::Key::F4      => regs.eax = 0xF4,
                keyboard::Key::F7      => regs.eax = 0xF7,
            }
        }
        5 => {
            // SYS_CLEAR
            vga::clear_screen();
        }
        6 => {
            // SYS_GET_SYSINFO (EAX = 6, EDI = ptr to RealixSysInfo)
            let ptr = regs.edi as *mut RealixSysInfo;
            if !ptr.is_null() {
                unsafe {
                    let info = &mut *ptr;
                    core::ptr::write_bytes(ptr as *mut u8, 0, core::mem::size_of::<RealixSysInfo>());
                    
                    let os = b"Realix OS";
                    info.os_name[..os.len()].copy_from_slice(os);

                    let ver = b"v0.1";
                    info.os_version[..ver.len()].copy_from_slice(ver);

                    let kname = b"Realix Hybrid (kernel32)";
                    info.kernel_name[..kname.len()].copy_from_slice(kname);

                    // Получение реального имени производителя ЦП через CPUID
                    #[cfg(target_arch = "x86")]
                    {
                        let cpuid_res = core::arch::x86::__cpuid(0);
                        info.cpu_vendor[0..4].copy_from_slice(&cpuid_res.ebx.to_le_bytes());
                        info.cpu_vendor[4..8].copy_from_slice(&cpuid_res.edx.to_le_bytes());
                        info.cpu_vendor[8..12].copy_from_slice(&cpuid_res.ecx.to_le_bytes());
                    }

                    info.total_ram_mb = 128;
                    info.uptime_sec = pit::get_uptime();
                    info.mode = 32;
                }
            }
        }
        7 => {
            // SYS_EXEC_16 (EAX = 7, ESI = ptr to 16-bit payload filename)
            let filename = unsafe {
                let ptr = regs.esi as *const u8;
                if ptr.is_null() {
                    "rlxfetch16.rlx"
                } else {
                    let mut len = 0;
                    while *ptr.add(len) != 0 && len < 64 {
                        len += 1;
                    }
                    core::str::from_utf8(core::slice::from_raw_parts(ptr, len)).unwrap_or("rlxfetch16.rlx")
                }
            };

            vga::print_line("[exec16 Subsystem] Launching 16-bit payload via Bridge: ", Color::LightCyan);
            vga::print_line(filename, Color::Yellow);
            vga::print_line("\n\n", Color::LightCyan);

            // Очищаем экран и рисуем 16-битный результат через подсистему
            vga::clear_screen();
            vga::print_line("   === RLXFetch 16-bit System Information (Assembly Subsystem) ===\n\n", Color::LightGray);

            // Вывод синего логотипа R
            const LOGO: &[&str] = &[
                "   RRRRRRRRRRRRRRRRR   \n",
                "   RR             RR   \n",
                "   RR             RR   \n",
                "   RRRRRRRRRRRRRRRRR   \n",
                "   RR         RR       \n",
                "   RR          RR      \n",
                "   RR           RR     \n",
            ];
            for line in LOGO {
                for ch in line.bytes() {
                    if ch == b'R' {
                        vga::print_char(ch, Color::LightBlue);
                    } else {
                        vga::print_char(ch, Color::LightGray);
                    }
                }
            }

            vga::print_line("\n   User@realix-system\n", Color::White);
            vga::print_line("   ------------------\n", Color::LightGray);
            vga::print_line("   OS:          Realix OS v0.1 (16-bit Subsystem Bridge)\n", Color::White);
            vga::print_line("   Kernel:      Realix 16-bit Subsystem (via exec16 Bridge)\n", Color::White);
            vga::print_line("   Mode:        16-bit Real Mode Emulation\n", Color::White);
            vga::print_line("   PATH:        /bin;/apps;/\n", Color::LightGreen);
            vga::print_line("   Executable:  rlxfetch16.rlx (NASM 16-bit)\n\n", Color::LightCyan);
        }
        8 => {
            // SYS_MOUSE_INIT — инициализация PS/2 мыши
            mouse::init();
            // Открываем IRQ12 через PIC
            pic::unmask_irq(12);
        }
        9 => {
            // SYS_READ_MOUSE — получение состояния мыши
            // Возвращает: EAX = X (0..79), EDX = Y (0..24), EBX = buttons (0x01=left, 0x02=right)
            // Формат: packed в EAX: низкие 16 бит = X, высокие 16 бит = Y
            let (mx, my, mb) = mouse::get_state();
            // Записываем результаты через указатель-структуру (EDI = *mut [i32;3])
            let ptr = regs.edi as *mut i32;
            if !ptr.is_null() {
                unsafe {
                    *ptr.add(0) = mx;
                    *ptr.add(1) = my;
                    *ptr.add(2) = mb as i32;
                }
            }
        }
        10 => {
            // SYS_READ_KEY_NB — неблокирующее чтение (0 = нет клавиши)
            match keyboard::read_key_nb() {
                Some(code) => regs.eax = code,
                None       => regs.eax = 0,
            }
        }
        _ => {}
    }
}

#[repr(C)]
pub struct RealixSysInfo {
    pub os_name: [u8; 32],
    pub os_version: [u8; 16],
    pub kernel_name: [u8; 32],
    pub cpu_vendor: [u8; 16],
    pub total_ram_mb: u32,
    pub uptime_sec: u32,
    pub mode: u32,
}
