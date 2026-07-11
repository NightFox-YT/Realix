// © Realix > Kernel32: Main
// Исправленная версия
// ===================

#![no_std]
#![no_main]

mod commands;
mod drivers;
mod shell;
mod utils;
mod x86;

use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;

use drivers::{keyboard, pit, vga};
use x86::{gdt, idt, memory};


/// Структура PCINFO, формируемая загрузчиком.
///
/// Layout:
///   +0: low_memory_amount, u16
///   +2: disk_num, u8
///   +3: E820Map
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct PcInfo {
    low_memory_amount: u16,
    disk_num: u8,
    memory_map: memory::E820Map,
}


/*
 * Символы определены в linker.ld.
 *
 * Они обозначают начало и конец секции BSS. Поскольку BSS не записывается
 * rust-objcopy в плоский binary-файл, её необходимо обнулить вручную.
 */
unsafe extern "C" {
    static __bss_start: u8;
    static __bss_end: u8;
}


/// Низкоуровневая точка входа Kernel32.
///
/// При входе:
///   EBX = физический адрес PCINFO;
///   ESP = стек, настроенный загрузчиком.
///
/// Код обязан выполняться до обращения к глобальным Rust-переменным,
/// поскольку они могут находиться в ещё не обнулённой BSS.
#[link_section = ".text.entry"]
#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    naked_asm!(
        /*
         * Строковые инструкции должны двигаться вперёд независимо от
         * состояния Direction Flag, оставленного BIOS или загрузчиком.
         */
        "cld",

        /* Сохраняем переданный загрузчиком адрес PCINFO. */
        "push ebx",

        /* EDI = начало BSS. */
        "lea edi, [__bss_start]",

        /* ECX = размер BSS. */
        "lea ecx, [__bss_end]",
        "sub ecx, edi",

        /* Обнуляем BSS. */
        "xor eax, eax",
        "rep stosb",

        /* Восстанавливаем адрес PCINFO. */
        "pop ebx",

        /* Передаём его первым аргументом cdecl. */
        "push ebx",
        "call kmain",

        /*
         * kmain имеет возвращаемый тип !, но оставляем защиту на случай
         * нарушения контракта.
         */
        "2:",
        "cli",
        "hlt",
        "jmp 2b",
    );
}


/// Основная функция ядра.
#[no_mangle]
extern "C" fn kmain(pcinfo_addr: *const PcInfo) -> ! {
    idt::interrupts_disable();

    gdt::init();
    idt::init();
    pit::init(100);

    idt::interrupts_enable();

    /*
     * Пока структура PCINFO не используется активно, но проверяем,
     * что загрузчик передал ненулевой указатель.
     */
    if pcinfo_addr.is_null() {
        vga::clear_screen();
        vga::print_line(
            "[KERNEL PANIC] Invalid PCINFO address.\n",
            vga::Color::Red,
        );
        halt_loop();
    }

    vga::clear_screen();
    draw_logo(4, 2);

    for _ in 0..11 {
        vga::new_line();
    }

    vga::print_line(
        "   Press any key to continue...",
        vga::Color::LightGray,
    );

    keyboard::read_key();

    vga::clear_screen();
    vga::print_line(
        "Welcome to Realix (Protected Mode with Rust kernel)...\n",
        vga::Color::Cyan,
    );

    shell::run();

    halt_loop();
}


/// Отрисовка логотипа Realix.
fn draw_logo(start_x: usize, start_y: usize) {
    let letters: [[[u8; 6]; 8]; 6] = [
        // R
        [
            [1][1][1][1][0][0],
            [1][0][0][1][0][0],
            [1][0][0][1][0][0],
            [1][1][1][1][0][0],
            [1][1][0][0][0][0],
            [1][0][1][0][0][0],
            [1][0][0][1][0][0],
            [1][0][0][1][0][0],
        ],

        // E
        [
            [1][1][1][1][1][0],
            [1][0][0][0][0][0],
            [1][0][0][0][0][0],
            [1][1][1][1][0][0],
            [1][0][0][0][0][0],
            [1][0][0][0][0][0],
            [1][0][0][0][0][0],
            [1][1][1][1][1][0],
        ],

        // A
        [
            [0][1][1][1][0][0],
            [1][0][0][0][1][0],
            [1][0][0][0][1][0],
            [1][1][1][1][1][0],
            [1][0][0][0][1][0],
            [1][0][0][0][1][0],
            [1][0][0][0][1][0],
            [1][0][0][0][1][0],
        ],

        // L
        [
            [1][0][0][0][0][0],
            [1][0][0][0][0][0],
            [1][0][0][0][0][0],
            [1][0][0][0][0][0],
            [1][0][0][0][0][0],
            [1][0][0][0][0][0],
            [1][0][0][0][0][0],
            [1][1][1][1][1][0],
        ],

        // I
        [
            [0][1][1][1][0][0],
            [0][0][1][0][0][0],
            [0][0][1][0][0][0],
            [0][0][1][0][0][0],
            [0][0][1][0][0][0],
            [0][0][1][0][0][0],
            [0][0][1][0][0][0],
            [0][1][1][1][0][0],
        ],

        // X
        [
            [1][0][0][0][1][0],
            [0][1][0][1][0][0],
            [0][0][1][0][0][0],
            [0][0][1][0][0][0],
            [0][0][1][0][0][0],
            [0][1][0][1][0][0],
            [1][0][0][0][1][0],
            [1][0][0][0][1][0],
        ],
    ];

    let colors = [
        vga::Color::Red,
        vga::Color::Yellow,
        vga::Color::Green,
        vga::Color::Cyan,
        vga::Color::Blue,
        vga::Color::Magenta,
    ];

    for (letter_index, letter) in letters.iter().enumerate() {
        let color = colors[letter_index % colors.len()];
        let offset_x = start_x + letter_index * letter[0].len();

        for (row, line) in letter.iter().enumerate() {
            for (column, &pixel) in line.iter().enumerate() {
                if pixel == 1 {
                    vga::write_char_at(
                        start_y + row,
                        offset_x + column,
                        0xDB,
                        color,
                    );
                }
            }
        }
    }
}


/// Бесконечная остановка процессора.
pub(crate) fn halt_loop() -> ! {
    idt::interrupts_disable();

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}


/// Обработчик паники.
///
/// VGA здесь намеренно не используется: паника может произойти внутри
/// VGA-драйвера или обработчика исключения.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_loop()
}


/// Запись байта в I/O-порт.
#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nostack, nomem, preserves_flags),
    );
}


/// Чтение байта из I/O-порта.
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nostack, nomem, preserves_flags),
    );

    value
}