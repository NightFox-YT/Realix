// © Realix > PIT (Programmable Interval Timer)
// (03.07.26) v0.08
// ================

// Подключение функций
use core::sync::atomic::{AtomicUsize, AtomicU32, Ordering::Relaxed};
use crate::drivers::vga;
use crate::outb;

// Порты PIT и базовая частота генератора PIT (Гц)
const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const BASE_FREQUENCY: u32 = 1_193_182;

// > Command byte:
// channel 0 (6-7) | low/high byte accessed (5-4)
// rate generator (3-1) | binary (0)
const PIT_COMMAND_BYTE: u8 = 0b00110100;

// Счётчик тиков с момента загрузки и установленная частота
static TICKS: AtomicUsize = AtomicUsize::new(0);
static FREQUENCY: AtomicU32 = AtomicU32::new(0);

/// Инициализация PIT на заданную частоту (Гц)
pub fn init(frequency: u32) {
    // Проверка, что частота в нужном диапазоне
    if frequency < 20 || frequency > BASE_FREQUENCY {
        vga::print_line("[!] PIT frequency isn't in valid range!", vga::Color::Red);
        panic!();
    }

    // Во сколько раз замедлить базовую частоту генератора
    let divisor: u32 = BASE_FREQUENCY / frequency;
    FREQUENCY.store(frequency, Relaxed);

    unsafe {
        // Отправляем команду выбора режима
        outb(PIT_COMMAND, PIT_COMMAND_BYTE);

        // Отправляем делитель побайтово
        // (Сначала младший байт, потом старший байт)
        outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);
    }
}

/// Вызывается irq_handler на каждый тик (IRQ0)
pub fn tick() {
    TICKS.fetch_add(1, Relaxed);
}

/// Возвращает кол-во тиков, которых прошло с момента загрузки
pub fn get_ticks() -> u32 {
    TICKS.load(Relaxed) as u32
}

/// Возвращает uptime (сек.)
pub fn get_uptime() -> u32 {
    let cur_frequency: u32 = FREQUENCY.load(Relaxed);

    // Защита от Zero Division
    if cur_frequency == 0 {
        return 0;
    }
    return TICKS.load(Relaxed) as u32 / cur_frequency;
}

/// Ждать `ms` миллисекунд
pub fn sleep(ms: u32) {
    let target: u64 = get_ticks() as u64 + (ms as u64 * (FREQUENCY.load(Relaxed)) as u64 / 1000);

    // Ожидаем, когда кол-во тиков достигнет нужное значение
    while (get_ticks() as u64) < target {
        // Останавливаем процессор между прерываниями (тиками)
        unsafe { core::arch::asm!("sti; hlt"); }
    }
}