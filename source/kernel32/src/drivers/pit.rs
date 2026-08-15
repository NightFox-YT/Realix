// © Realix > Driver: PIT (Programmable Interval Timer)
// (27.07.26) v0.1
// ================

// Подключение функций
use core::sync::atomic::{AtomicUsize, AtomicU32, Ordering::Relaxed};
use crate::drivers::vga;
use crate::utils::outb;

// Порты PIT и базовая частота генератора PIT (Гц)
const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const BASE_FREQUENCY: u32 = 1_193_182;

// Нижний предел частоты (Делитель должен влезать в 16 бит)
const MIN_FREQUENCY: u32 = 20;

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
    if !(MIN_FREQUENCY..=BASE_FREQUENCY).contains(&frequency) {
        vga::print_line("[!] PIT frequency isn't in valid range!", vga::Color::Red);
        panic!();
    }

    // Во сколько раз замедлить базовую частоту генератора
    let divisor: u32 = BASE_FREQUENCY / frequency;
    FREQUENCY.store(frequency, Relaxed);

    set_raw_divisor(divisor as u16);
}

/// Программирует делитель PIT напрямую, без проверки минимальной частоты -
/// используется x86::realmode для временного возврата к "родному" делителю
/// BIOS (0, что аппаратно означает 65536 -> ~18.2 Гц) перед вызовом BIOS.
/// Некоторые BIOS-сервисы (напр. чтение/запись гибкого диска: раскрутка
/// мотора, таймауты) сами используют PIT для отсчёта задержек - если он
/// тикает в divisor/65536 раз чаще, чем BIOS рассчитывает, эти отсчёты
/// сбиваются (независимо от маскирования прерываний - делитель влияет на
/// сам счётчик PIT, не только на доставку прерывания от него)
/// Параметры:
///  - divisor: делитель (0 аппаратно означает 65536)
pub fn set_raw_divisor(divisor: u16) {
    unsafe {
        // Отправляем команду выбора режима
        outb(PIT_COMMAND, PIT_COMMAND_BYTE);

        // Отправляем делитель побайтово
        // (Сначала младший байт, потом старший байт)
        outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);
    }
}

/// Переключает PIT на "родной" делитель BIOS (0 -> 65536, ~18.2 Гц) не
/// трогая сохранённую FREQUENCY - см. set_raw_divisor. get_uptime()/sleep()
/// при этом продолжат считать по прежней (настоящей) частоте kernel32, а не
/// по временной - это не проблема, т.к. окно использования короткое
/// (Real Mode BIOS вызов) и не рассчитано быть точным источником времени
pub fn use_bios_rate() {
    set_raw_divisor(0);
}

/// Возвращает PIT к частоте kernel32, сохранённой в FREQUENCY (см. init())
pub fn restore_rate() {
    let frequency: u32 = FREQUENCY.load(Relaxed);
    if frequency == 0 {
        return; // init() ещё не вызывался - восстанавливать нечего
    }
    set_raw_divisor((BASE_FREQUENCY / frequency) as u16);
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
    (TICKS.load(Relaxed) as u32) / cur_frequency
}

/// Ждать `ms` миллисекунд
pub fn sleep(ms: u32) {
    let frequency: u64 = FREQUENCY.load(Relaxed) as u64;

    // Проверка: Если частота 0, то мы не можем посчитать target
    if frequency == 0 {
        return;
    }

    let target: u64 = get_ticks() as u64 + (ms as u64 * frequency / 1000u64);

    // Ожидаем, когда кол-во тиков достигнет нужное значение
    while (get_ticks() as u64) < target {
        // Останавливаем процессор между прерываниями (тиками)
        unsafe { core::arch::asm!("sti; hlt", options(nostack, nomem)); }
    }
}