// © Realix > PIC (Programmable Interrupt Controller)
// (03.07.26) v0.08
// ================

// Подключение функций
use crate::{inb, outb};

// Порты Master & Slave
const PIC1_CMD:  u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD:  u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

/// Перепрошивка PIC на векторы 32-47, чтобы не пересекаться с исключениями 0-19
pub fn remap() {
    // Настройка ICW (Initialization Command Words)
    unsafe {
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
    }

    // Ставим маски, кроме таймера (IRQ0) и клавиатуры (IRQ1)
    for i in 2..=15 {
        mask_irq(i);
    }
    unmask_irq(0);
    unmask_irq(1);
}

/// Замаскировать IRQ для отключения прерывания
pub fn mask_irq(irq: u8) {
    unsafe {
        if irq < 8 {
            let pic1_value: u8 = inb(PIC1_DATA);
            outb(PIC1_DATA, pic1_value | (1 << irq));
        } else if irq <= 15 {
            let pic2_value: u8 = inb(PIC2_DATA);
            outb(PIC2_DATA, pic2_value | (1 << (irq - 8)));
        } else { return; }
    }
}

/// Убрать маску с IRQ для включения прерывания
pub fn unmask_irq(irq: u8) {
    unsafe {
        if irq < 8 {
            let pic1_value: u8 = inb(PIC1_DATA);
            outb(PIC1_DATA, pic1_value & !(1 << irq));
        } else if irq <= 15 {
            let pic2_value: u8 = inb(PIC2_DATA);
            outb(PIC2_DATA, pic2_value & !(1 << (irq - 8)));
        } else { return; }
    }
}

/// Чтение регистра ISR (Какие IRQ обслуживаются)
fn read_isr(cmd_port: u16) -> u8 {
    unsafe {
        // OCW3: Запрос на чтение ISR
        outb(cmd_port, 0x0B);
        inb(cmd_port)
    }
}

/// Проверка ложного прерывания на IRQ7/IRQ15
pub fn is_spurious(irq: u8) -> bool {
    match irq {
        7 => read_isr(PIC1_CMD) & 0x80 == 0,
        15 => {
            if read_isr(PIC2_CMD) & 0x80 == 0 {
                // Master считает slave-прерывание реальным, отправляем EOI
                unsafe { outb(PIC1_CMD, 0x20); }
                true
            } else { false }
        }
        _ => false,
    }
}

/// Отправка сигнала об успешной обработке прерывания
/// Параметры:
///  - irq: номер IRQ прерывания, а не вектора прерывания
pub fn send_eoi(irq: u8) {
    // Выходим из функции, если отправили номер вектора, а не IRQ
    if irq >= 16 { return; }

    unsafe { outb(PIC1_CMD, 0x20); }
    if irq >= 8 {
        unsafe { outb(PIC2_CMD, 0x20); }
    }
}