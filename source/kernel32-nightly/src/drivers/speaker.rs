// © Realix > Driver: PC Speaker
// (07.09.26) v0.12
// ================
// ❗️ Использует канал 2 PIT + gate-биты порта 0x61 (BIOS-гудок недоступен в protected mode)

// Подключение функций
use crate::drivers::pit;
use crate::utils::{inb, outb};

// Порты
const PIT_CHANNEL2: u16 = 0x42;
const PIT_COMMAND: u16 = 0x43;
const SPEAKER_PORT: u16 = 0x61;

// Базовая частота генератора PIT (Гц)
const BASE_FREQUENCY: u32 = 1_193_182;

// > Command byte: channel 2 (7-6) | low/high byte (5-4) | square wave (3-1) | binary (0)
const PIT_CHANNEL2_COMMAND: u8 = 0b1011_0110;

// Биты порта 0x61: бит 0 - gate таймера, бит 1 - подключение динамика к каналу 2
const SPEAKER_GATE_MASK: u8 = 0b0000_0011;

// Параметры короткого системного гудка
const BEEP_FREQUENCY: u32 = 1000; // Гц
const BEEP_DURATION_MS: u32 = 200;

/// Включение динамика на заданной частоте (Гц)
fn start(frequency: u32) {
    let divisor: u32 = BASE_FREQUENCY / frequency;

    unsafe {
        outb(PIT_COMMAND, PIT_CHANNEL2_COMMAND);
        outb(PIT_CHANNEL2, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL2, ((divisor >> 8) & 0xFF) as u8);

        let cur: u8 = inb(SPEAKER_PORT);
        outb(SPEAKER_PORT, cur | SPEAKER_GATE_MASK);
    }
}

/// Выключение динамика (Возвращает исходные биты порта, кроме gate-битов)
fn stop() {
    unsafe {
        let cur: u8 = inb(SPEAKER_PORT);
        outb(SPEAKER_PORT, cur & !SPEAKER_GATE_MASK);
    }
}

/// Короткий системный гудок фиксированной частоты и длительности
pub fn beep() {
    start(BEEP_FREQUENCY);
    pit::sleep(BEEP_DURATION_MS);
    stop();
}
