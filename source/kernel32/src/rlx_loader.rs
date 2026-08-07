// © Realix > Kernel32: .RLX 32-bit Loader & Ring 3 Execution
// =========================================================

use core::arch::asm;
use crate::drivers::vga::{self, Color};
use crate::x86::gdt;

#[repr(C, packed)]
struct RlxHeader32 {
    magic: [u8; 3],
    mode: u8,
    entry: u32,
    code_sz: u32,
    reserved: u32,
}

// Контекст для безопасного возврата в ядро (setjmp / longjmp)
pub static mut KERNEL_SAVED_ESP: u32 = 0;
pub static mut KERNEL_SAVED_EBP: u32 = 0;
pub static mut KERNEL_SAVED_EIP: u32 = 0;

// Границы буфера приложений User Mode (1MB базовая область)
const USER_APP_ADDR: u32 = 0x00400000;
const USER_STACK_TOP: u32 = 0x00420000;

// Выделенный стек TSS для прерываний из Ring 3 (изолирован от стека оболочки 0x90000)
const TSS_STACK_TOP: u32 = 0x00088000;

/// Возврат управления из Ring 3 приложения в Ring 0 Shell при вызове SYS_EXIT
pub fn exit_to_kernel() {
    unsafe {
        let esp = KERNEL_SAVED_ESP;
        let ebp = KERNEL_SAVED_EBP;
        let eip = KERNEL_SAVED_EIP;

        if esp != 0 && eip != 0 {
            KERNEL_SAVED_ESP = 0;
            KERNEL_SAVED_EBP = 0;
            KERNEL_SAVED_EIP = 0;

            vga::print_line(
                "[RLX32 Loader] Application exited cleanly. Returning to Ring 0 Shell...\n",
                Color::LightGreen,
            );

            asm!(
                "mov ax, 0x10",
                "mov ds, ax",
                "mov es, ax",
                "mov fs, ax",
                "mov gs, ax",
                "mov ss, ax",
                "mov esp, {esp}",
                "mov ebp, {ebp}",
                "jmp {eip}",
                esp = in(reg) esp,
                ebp = in(reg) ebp,
                eip = in(reg) eip,
                options(noreturn)
            );
        }
    }
}

/// Безопасное сохранение контекста ядра и переход в Ring 3
#[no_mangle]
pub unsafe fn rlx_launch_user_mode(entry: u32, user_stack: u32) {
    asm!(
        "mov [{saved_esp}], esp",
        "mov [{saved_ebp}], ebp",
        "lea eax, [2f]",
        "mov [{saved_eip}], eax",

        // 2. Формируем стековый фрейм iretd для переключения в Ring 3
        "push 0x23",        // User SS (0x23)
        "push edx",         // User ESP (user_stack)
        "push 0x0202",      // EFLAGS (IF = 1)
        "push 0x1B",        // User CS (0x1B)
        "push ecx",         // User EIP (entry)

        // 3. Устанавливаем пользовательские селекторы данных (0x23)
        "mov ax, 0x23",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "iretd",

        // 4. Метка прямого возврата из exit_to_kernel (longjmp)
        "2:",
        saved_esp = in(reg) &raw mut KERNEL_SAVED_ESP,
        saved_ebp = in(reg) &raw mut KERNEL_SAVED_EBP,
        saved_eip = in(reg) &raw mut KERNEL_SAVED_EIP,
        in("ecx") entry,
        in("edx") user_stack,
    );
}

/// Проверка и запуск 32-битного .RLX приложения в режиме Ring 3
pub fn run_rlx32(image: &[u8]) -> Result<(), &'static str> {
    if image.len() < core::mem::size_of::<RlxHeader32>() {
        return Err("File too small to contain a valid .RLX header");
    }

    let header_ptr = image.as_ptr() as *const RlxHeader32;
    let header = unsafe { &*header_ptr };

    if header.magic != [b'R', b'L', b'X'] {
        return Err("Invalid .RLX header magic signature");
    }

    if header.mode != 0x32 && header.mode != b'U' {
        return Err("Not a 32-bit or Universal .RLX executable");
    }

    vga::print_line(
        "[RLX32 Loader] Loading 32-bit / Universal application into Ring 3...\n",
        Color::Yellow,
    );

    // Копирование всего образа приложения (заголовок + код + данные) в USER_APP_ADDR (0x00400000)
    let app_dst = USER_APP_ADDR as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(image.as_ptr(), app_dst, image.len());
    }

    let entry_offset = if header.mode == b'U' {
        unsafe {
            let u32_ptr = image.as_ptr().add(8) as *const u32;
            *u32_ptr
        }
    } else {
        header.entry
    };

    let app_entry_point = USER_APP_ADDR + entry_offset;

    // Изолируем стек TSS прерываний (0x88000), чтобы он не затирал стек ядра оболочки (0x90000)
    gdt::set_tss_esp0(TSS_STACK_TOP);

    vga::print_line(
        "[RLX32 Loader] Switching CPU privilege level to Ring 3 (User Mode)...\n",
        Color::Cyan,
    );

    unsafe {
        rlx_launch_user_mode(app_entry_point, USER_STACK_TOP);
    }

    Ok(())
}
