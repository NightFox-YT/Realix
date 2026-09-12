// © Realix > Driver: PC Speaker
// ø Copyright by @jeksonkd
// (15.08.26) v0.1
// ================
// ❗️ Зависимости: pit (BASE_FREQUENCY, sleep)
// ❗️ Использует PIT channel 2 (порты 0x42/0x43) и speaker gate (порт 0x61)

// Подключение функций
use crate::drivers::pit;
use crate::utils::{inb, outb};

// Порты PIT и gate-контроля спикера
const PIT_CHANNEL2: u16 = 0x42;
const PIT_COMMAND: u16 = 0x43;
const SPEAKER_GATE: u16 = 0x61;

// > Command byte:
// channel 2 (7-6) | low/high byte accessed (5-4)
// square wave generator (3-1) | binary (0)
const PIT_COMMAND_BYTE: u8 = 0b10110110;

// > NMI status:
// timer 2 gate (0) | speaker data enable (1)
const SPEAKER_GATE_MASK: u8 = 0b11;

// Параметры короткого сигнала
const SHORT_BEEP_FREQUENCY: u32 = 1000;
const SHORT_BEEP_DURATION_MS: u32 = 100;

/// Включить PC-спикер на заданной частоте
/// Параметры:
///  - frequency: частота звука в Гц
pub fn on(frequency: u32) {
    let divisor: u16 = (pit::BASE_FREQUENCY / frequency) as u16;

    unsafe {
        // Отправляем PIT инструкцию (приём во 2-ой канал)
        outb(PIT_COMMAND, PIT_COMMAND_BYTE);

        // Загружаем делитель частоты (сначала младший байт, потом старший)
        outb(PIT_CHANNEL2, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL2, ((divisor >> 8) & 0xFF) as u8);

        // Открываем gate 2-ого канала и разрешаем использование динамика
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

/// Издать короткий звуковой сигнал (для обратной связи)
pub fn beep_short() {
    on(SHORT_BEEP_FREQUENCY);
    pit::sleep(SHORT_BEEP_DURATION_MS);
    off();
}
