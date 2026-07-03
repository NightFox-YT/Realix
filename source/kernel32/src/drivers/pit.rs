// © Realix > PIT (Programmable Interval Timer)
// (03.07.26) v0.08
// ================

// Подключение функций
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
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
static FREQUENCY: AtomicUsize = AtomicUsize::new(0);

/// Инициализация PIT на заданную частоту (Гц)
pub fn init(frequency: u32) {
    // Во сколько раз замедлить базовую частоту генератора
    let divisor: u32 = BASE_FREQUENCY / frequency;
    FREQUENCY.store(frequency as usize, Relaxed);

    // Отправляем команду выбора режима
    outb(PIT_COMMAND, PIT_COMMAND_BYTE);

    // Отправляем делитель побайтово
    // (Сначала младший байт, потом старший байт)
    outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
    outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);
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
    (TICKS.load(Relaxed) / FREQUENCY.load(Relaxed)) as u32
}

/// Ждать `ms` миллисекунд
pub fn sleep(ms: u32) {
    let target: u32 = get_ticks() + (ms * (FREQUENCY.load(Relaxed) / 1000) as u32);

    // Ожидаем, когда кол-во тиков достигнет нужное значение
    while get_ticks() < target {
        // Останавливаем процессор между прерываниями (тиками)
        unsafe { core::arch::asm!("hlt"); }
    }
}