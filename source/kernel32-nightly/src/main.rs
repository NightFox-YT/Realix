// © Realix > Kernel32: Main
// (27.07.26) v0.1
// ================
// ❗️ Загружается Switcher по адресу KERNEL32_PHYS_ADDR, адрес PCINFO в ebx

#![no_std]
#![no_main]

// Объявление модулей
mod commands;
mod drivers;
mod fs;
mod memory;
mod shell;
mod utils;
mod x86;

// Подключение функций
use core::arch::{asm, naked_asm};
use core::fmt::Write as _;
use core::panic::PanicInfo;
use drivers::{keyboard, pit, vga};
use memory::{frame_allocator, pmm};
use x86::{gdt, idt};

use crate::memory::pmm::E820Entry;

/// Максимум записей карты (Значение берёт E820_MAX_ENTRIES в high.asm)
pub const E820_MAX_ENTRIES: usize = 64;
pub const PCINFO_ADDR: usize = 0x4500;

/// Текущая версия ОС (Должна совпадать с OS_VERSION в shared/config.asm)
pub const OS_VERSION: &str = "v0.11";

/// Нуль-терминированная версия `OS_VERSION` - для отдачи сырого указателя через syscall
/// (SYS_GET_VERSION), т.к. RLX-приложения печатают строки по указателю до `\0`
pub const OS_VERSION_CSTR: &[u8] = b"v0.11\0";

/// Структура PCINFO, формируемая загрузчиком
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PcInfo {
    pub low_memory_kb: u16,
    pub boot_drive_num: u8,
    pub mmap_count: u16,
    pub memory_map: [pmm::E820Entry; E820_MAX_ENTRIES],
}

// Доступ-обёртка к элементам структуры
impl PcInfo {
    #[inline]
    fn mmap_entry(&self, i: usize) -> E820Entry {
        if i > E820_MAX_ENTRIES {
            return pmm::E820Entry {
                address: 0,
                size: 0,
                seg_type: 0,
                attributes: 0,
            };
        }
        unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.memory_map[i as usize])) }
    }
}

/// Предоставление доступа-обёртки к структуре PcInfo
#[inline]
unsafe fn pcinfo() -> &'static PcInfo {
    &*(PCINFO_ADDR as *const PcInfo)
}

// Границы секции BSS (из linker.ld) для обнуления вручную
unsafe extern "C" {
    unsafe static __bss_start: u8;
    unsafe static __bss_end: u8;
}

/// Низкоуровневая точка входа Kernel32 (Настройка окружения)
/// Параметры:
///  - ebx: адрес PCINFO
///  - esp: стек от загрузчика
#[link_section = ".text.entry"]
#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    naked_asm!(
        // Установка: Сброс DF, edi - начало BSS, ecx - её размер
        "cld",
        "lea edi, [__bss_start]",
        "lea ecx, [__bss_end]",
        "sub ecx, edi",
        // Обнуляем BSS через eax
        "xor eax, eax",
        "rep stosb",
        // Передаём адрес PCINFO первым аргументом по cdecl
        "push ebx",
        "call kmain",
        // Защита на случай незапланированного возвращения из функции
        "2:",
        "cli",
        "hlt",
        "jmp 2b"
    );
}

/// Основной цикл работы ядра
/// Параметры:
///  - pcinfo_addr: адрес структуры PCINFO, собранной загрузчиком
#[no_mangle]
extern "C" fn kmain(pcinfo_addr: *const PcInfo) -> ! {
    // Инициализация модулей
    idt::interrupts_disable();
    gdt::init();
    idt::init();
    pit::init(100);
    pmm::init_kernel_page_allocator();
    idt::interrupts_enable();

    // Проверка указателя PCINFO
    if pcinfo_addr.is_null() {
        vga::clear_screen();
        vga::print_line("[KERNEL PANIC] Invalid PCINFO address.\n", vga::Color::Red);
        halt_loop();
    }

    // Инициализация аллокатора фреймов по карте памяти E820 из PCINFO
    unsafe {
        let pcinfo: &PcInfo = &*pcinfo_addr;
        frame_allocator::init(&pcinfo.memory_map);
    }

    // Вывод логотипа и приглашения
    vga::clear_screen();
    draw_logo(4, 2);

    vga::print_line("   Press any key to continue...", vga::Color::LightGray);
    keyboard::read_key();

    // Вывод заголовка Shell с его бесконечной работой
    vga::clear_screen();
    vga::print_line(
        "Welcome to Realix (Protected Mode with Rust kernel)...\n",
        vga::Color::Cyan,
    );

    // ! Вывод заметки о экспериментальной функции NovaAI
    vga::print_line(
        "Integration with NovaAI (type 'nova -a' to chat)\n",
        vga::Color::LightCyan,
    );

    shell::run();
    halt_loop();
}

/// Отрисовка логотипа Realix
fn draw_logo(start_x: usize, start_y: usize) {
    // Массив из 2 уровней:
    // 1. Массивы для каждой буквы
    // 2. Массивы для каждой строки буквы (0 - пробел, 1 - блок)
    let letters: [[[u8; 6]; 8]; 6] = [
        [[1,1,1,1,0,0],[1,0,0,1,0,0],[1,0,0,1,0,0],[1,1,1,1,0,0],
         [1,1,0,0,0,0],[1,0,1,0,0,0],[1,0,0,1,0,0],[1,0,0,1,0,0]],
        [[1,1,1,1,1,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,1,1,1,0,0],
         [1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,1,1,1,1,0]],
        [[0,1,1,1,0,0],[1,0,0,0,1,0],[1,0,0,0,1,0],[1,1,1,1,1,0],
         [1,0,0,0,1,0],[1,0,0,0,1,0],[1,0,0,0,1,0],[1,0,0,0,1,0]],
        [[1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],
         [1,0,0,0,0,0],[1,0,0,0,0,0],[1,0,0,0,0,0],[1,1,1,1,1,0]],
        [[0,1,1,1,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],
         [0,0,1,0,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],[0,1,1,1,0,0]],
        [[1,0,0,0,1,0],[0,1,0,1,0,0],[0,0,1,0,0,0],[0,0,1,0,0,0],
         [0,0,1,0,0,0],[0,1,0,1,0,0],[1,0,0,0,1,0],[1,0,0,0,1,0]],
    ];

    // Список цветов для букв (1 буква - 1 цвет)
    let colors = [
        vga::Color::Red,
        vga::Color::Yellow,
        vga::Color::Green,
        vga::Color::Cyan,
        vga::Color::Blue,
        vga::Color::Magenta,
    ];

    for (letter_idx, letter) in letters.iter().enumerate() {
        let color = colors[letter_idx % colors.len()];
        let offset_x = start_x + letter_idx * letter[0].len();

        for (row, line) in letter.iter().enumerate() {
            for (col, &pixel) in line.iter().enumerate() {
                if pixel == 1 {
                    vga::write_char_at(start_y + row, offset_x + col, 0xDB, color);
                }
            }
        }
    }

    for _ in 0..11 {
        vga::new_line();
    }
}

/// Бесконечная остановка процессора
pub fn halt_loop() -> ! {
    idt::interrupts_disable();
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}

/// Обёртка для вывода `core::fmt::Arguments` через VGA-драйвер (Без аллокатора)
struct VgaWriter;

impl core::fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        vga::print_line(s, vga::Color::Red);
        Ok(())
    }
}

/// Обработчик паники: экран диагностики (сообщение + место) и остановка процессора
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    idt::interrupts_disable();

    vga::print_line("\n[KERNEL PANIC] Realix Rust panic:\n", vga::Color::Red);

    vga::print_line("Reason: ", vga::Color::Red);
    let mut writer: VgaWriter = VgaWriter;
    let _ = write!(writer, "{}", info.message());
    vga::new_line();

    if let Some(location) = info.location() {
        let mut buf: [u8; 10] = [0u8; 10];

        vga::print_line("Location: ", vga::Color::Red);
        vga::print_line(location.file(), vga::Color::Red);
        vga::print_line(":", vga::Color::Red);
        vga::print_line(utils::u32_to_dec_str(location.line(), &mut buf), vga::Color::Red);
        vga::new_line();
    }

    vga::new_line();
    vga::print_line("! System halted, please, reboot the machine.\n", vga::Color::Red);

    halt_loop()
}
