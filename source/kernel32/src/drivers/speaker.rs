// © Realix > Driver: PC Speaker
// (15.08.26) v0.1
// ================
// ❗️ Зависимости: pit (BASE_FREQUENCY, sleep)
// ❗️ Использует PIT channel 2 (порты 0x42/0x43) и speaker gate (порт 0x61)

// Подключение функций
use crate::drivers::pit;
use crate::utils::{inb, outb};

// Порты PIT (канал 2) и gate-контроля спикера
const PIT_CHANNEL2: u16 = 0x42;
const PIT_COMMAND: u16 = 0x43;
const SPEAKER_GATE: u16 = 0x61;

// Command byte:
// channel 2 (7-6) | low/high byte accessed (5-4)
// square wave generator (3-1) | binary (0)
const PIT_COMMAND_BYTE: u8 = 0b10110110;

// Биты gate-порта: бит 0 - gate канала 2, бит 1 - вход динамика от PIT
const SPEAKER_GATE_MASK: u8 = 0b11;

// Параметры короткого CLI-сигнала
const SHORT_BEEP_FREQUENCY: u32 = 1000;
const SHORT_BEEP_DURATION_MS: u32 = 100;

/// Включить PC-спикер на заданной частоте
/// Параметры:
///  - frequency: частота звука в Гц
pub fn on(frequency: u32) {
    let divisor: u16 = (pit::BASE_FREQUENCY / frequency) as u16;

    unsafe {
        // Выбираем канал 2 и загружаем делитель (сначала младший байт, потом старший)
        outb(PIT_COMMAND, PIT_COMMAND_BYTE);
        outb(PIT_CHANNEL2, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL2, ((divisor >> 8) & 0xFF) as u8);

        // Открываем gate канала 2 и подключаем его выход к динамику
        let gate: u8 = inb(SPEAKER_GATE);
        if gate & SPEAKER_GATE_MASK != SPEAKER_GATE_MASK {
            outb(SPEAKER_GATE, gate | SPEAKER_GATE_MASK);
        }
    }
}

/// Выключить PC-спикер
pub fn off() {
    unsafe {
        let gate: u8 = inb(SPEAKER_GATE);
        outb(SPEAKER_GATE, gate & !SPEAKER_GATE_MASK);
    }
}

/// Короткий звуковой сигнал обратной связи (используется, например,
/// когда CLI отклоняет ввод — переполнение буфера строки)
pub fn beep_short() {
    on(SHORT_BEEP_FREQUENCY);
    pit::sleep(SHORT_BEEP_DURATION_MS);
    off();
}
