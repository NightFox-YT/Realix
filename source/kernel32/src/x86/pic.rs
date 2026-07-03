// © Realix > PIC (Programmable Interrupt Controller)
// (03.07.26) v0.08
// ================

// Подключение функций
use crate::outb;

// Порты Master & Slave
const PIC1_CMD:  u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD:  u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

/// Перепрошивка PIC на векторы 32-47, чтобы не пересекаться с исключениями 0-19
/// (Настройка ICW (Initialization Command Words))
pub fn remap() {
    // Инициализация (ICW1)
    outb(PIC1_CMD, 0x11);
    outb(PIC2_CMD, 0x11);

    // Задаём смещение для векторов IRQ (ICW2)
    outb(PIC1_DATA, 0x20);
    outb(PIC2_DATA, 0x28);

    // Связываем master & slave (slave подключён к IRQ2 master'а, ICW3)
    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);

    // Включение режима 8086 (x86)
    outb(PIC1_DATA, 0x01);
    outb(PIC2_DATA, 0x01);

    // Ставим маски, разрешаем только таймер (IRQ0) и клавиатуру (IRQ1)
    outb(PIC1_DATA, 0b11111100);
    outb(PIC2_DATA, 0b11111111);
}

/// Отправка сигнала об успешной обработке прерывания
pub fn send_eoi(irq: u8) {
    outb(PIC1_CMD, 0x20);
    if irq >= 8 {
        outb(PIC2_CMD, 0x20);
    }
}