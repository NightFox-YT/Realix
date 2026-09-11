// © Realix > x86: Syscall interface (int 0x80) для 32-битных RLX-приложений
// (07.09.26) v0.12
// ================
// ❗️ Номера вызовов совпадают с kernel16/syscall.asm и shared/config.asm (SYS_*):
//    опкод в AH (Верхний байт EAX), как и в 16-битной версии.
//    ESI используется как плоский (линейный) указатель на строку - вместо DS:SI

// Подключение функций
use crate::drivers::{keyboard, pit, vga};
use crate::memory::frame_allocator;
use crate::x86::isr::Registers;
use crate::OS_VERSION_CSTR;

// Номера системных вызовов (Должны совпадать с shared/config.asm)
const SYS_PRINT_STRING: u32 = 1;
const SYS_PUTCHAR: u32 = 2;
const SYS_EXIT: u32 = 3;
const SYS_READ_KEY: u32 = 4;
const SYS_CLEAR: u32 = 5;
const SYS_GET_UPTIME: u32 = 6;
const SYS_GET_MEMINFO: u32 = 7;
const SYS_GET_VERSION: u32 = 8;

/// Флаг работы 32-битного приложения (Аналог `app16_running` в kernel16/syscall.asm)
/// ❗️ Как и в 16-битной версии, само по себе не прерывает выполнение - приложение
///    обязано после `SYS_EXIT` самостоятельно сделать `ret` в загрузчик
pub static mut APP_RUNNING: bool = false;

/// Печать строки по плоскому указателю до нуль-терминатора
/// ❗️ Указатель приходит из RLX-приложения "как есть" - доверенная модель (Как и в 16-бит)
unsafe fn print_c_str(ptr: *const u8) {
    let mut cursor: *const u8 = ptr;
    loop {
        let byte: u8 = *cursor;
        if byte == 0 {
            break;
        }
        vga::print_char(byte, vga::Color::LightGray);
        cursor = cursor.add(1);
    }
}

/// Блокирующее чтение клавиши, возвращает ASCII-код (Игнорирует спец-клавиши)
fn blocking_read_ascii() -> u8 {
    loop {
        if let keyboard::Key::Char(byte) = keyboard::read_key() {
            return byte;
        }
    }
}

/// Обработчик `int 0x80`, вызывается общей заглушкой `syscall_common_stub` (isr.rs)
#[no_mangle]
pub extern "C" fn syscall_handler(regs: &mut Registers) {
    let syscall_num: u32 = (regs.eax >> 8) & 0xFF;

    match syscall_num {
        SYS_PRINT_STRING => unsafe {
            print_c_str(regs.esi as *const u8);
        },
        SYS_PUTCHAR => {
            vga::print_char((regs.eax & 0xFF) as u8, vga::Color::LightGray);
        }
        SYS_EXIT => unsafe {
            APP_RUNNING = false;
        },
        SYS_READ_KEY => {
            regs.eax = blocking_read_ascii() as u32;
        }
        SYS_CLEAR => {
            vga::clear_screen();
        }
        SYS_GET_UPTIME => {
            regs.ecx = pit::get_uptime();
        }
        SYS_GET_MEMINFO => {
            regs.ecx = (frame_allocator::get_total_memory() / 1024) as u32;
            regs.edx = (frame_allocator::get_free_memory() / 1024) as u32;
        }
        SYS_GET_VERSION => {
            regs.ecx = OS_VERSION_CSTR.as_ptr() as u32;
        }
        _ => {}
    }
}
