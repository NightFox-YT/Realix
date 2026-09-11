// © Realix > PIC (Programmable Interrupt Controller)
// (11.07.26) v0.09
// ================

// Подключение функций
use crate::utils::{inb, outb};

// Командные порты и порты данных Master & Slave
const PIC1_CMD:  u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD:  u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

// Сигнал "End Of Interrupt"
const PIC_EOI: u8 = 0x20;

// Смещения векторов IRQ в IDT (32-47)
const PIC1_OFFSET: u8 = 0x20;
const PIC2_OFFSET: u8 = 0x28;


/// Короткая задержка для старых PIC/ISA-устройств
/// (Порт 0x80 исторически под POST-коды, годится для паузы ввода-вывода)
#[inline(always)]
unsafe fn io_wait() {
    outb(0x80, 0);
}


/// Перепрошивка PIC на векторы 32-47, чтобы не пересекаться с исключениями 0-19
pub fn remap() {
    // Настройка ICW (Initialization Command Words)
    unsafe {
        // ICW1: Старт инициализации, ожидается ICW4
        outb(PIC1_CMD, 0x11);
        io_wait();
        outb(PIC2_CMD, 0x11);
        io_wait();

        // ICW2: Задаём смещение для векторов IRQ в IDT
        outb(PIC1_DATA, PIC1_OFFSET);
        io_wait();
        outb(PIC2_DATA, PIC2_OFFSET);
        io_wait();

        // ICW3: Связываем master & slave
        // (slave подключён к IRQ2 master'а)
        outb(PIC1_DATA, 0x04);
        io_wait();
        outb(PIC2_DATA, 0x02);
        io_wait();

        // ICW4: режим 8086
        outb(PIC1_DATA, 0x01);
        io_wait();
        outb(PIC2_DATA, 0x01);
        io_wait();

        // Маскируем все IRQ, нужные откроем ниже
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }

    // IRQ0 - PIT, IRQ1 - клавиатура, IRQ2 - slave PIC, IRQ6 - флоппи-контроллер
    unmask_irq(0);
    unmask_irq(1);
    unmask_irq(2);
    unmask_irq(6);
}


/// Маскирование IRQ (запрет прерывания)
#[allow(dead_code)]
pub fn mask_irq(irq: u8) {
    unsafe {
        match irq {
            0..=7 => {
                let mask = inb(PIC1_DATA);
                outb(PIC1_DATA, mask | (1u8 << irq));
            }
            8..=15 => {
                let mask = inb(PIC2_DATA);
                outb(PIC2_DATA, mask | (1u8 << (irq - 8)));
            }
            _ => {}
        }
    }
}

/// Снятие маски IRQ (разрешение прерывания)
pub fn unmask_irq(irq: u8) {
    unsafe {
        match irq {
            0..=7 => {
                let mask = inb(PIC1_DATA);
                outb(PIC1_DATA, mask & !(1u8 << irq));
            }
            8..=15 => {
                let mask = inb(PIC2_DATA);
                outb(PIC2_DATA, mask & !(1u8 << (irq - 8)));

                // Для slave-IRQ надо обязательно открыть и IRQ2 master'а
                let master = inb(PIC1_DATA);
                outb(PIC1_DATA, master & !(1u8 << 2));
            }
            _ => {}
        }
    }
}


/// Отправка "End Of Interrupt" об успешной обработке прерывания
/// (Для IRQ8-15 EOI идёт сначала slave, затем master)
/// Параметры:
///  - irq: номер IRQ прерывания, а не вектора прерывания
pub fn send_eoi(irq: u8) {
    // Выходим из функции, если отправили номер вектора, а не IRQ
    if irq > 15 { return; }

    unsafe {
        if irq >= 8 { outb(PIC2_CMD, PIC_EOI); }
        outb(PIC1_CMD, PIC_EOI);
    }
}


/// Чтение In-Service Register (ISR) контроллера
fn read_isr(cmd_port: u16) -> u8 {
    unsafe {
        // OCW3: Запрос на чтение ISR
        outb(cmd_port, 0x0B);
        inb(cmd_port)
    }
}


/// Детект ложного (spurious) прерывания IRQ7/IRQ15.
pub fn is_spurious(irq: u8) -> bool {
    match irq {
        // IRQ7: ложное, если бит 7 в ISR master'а не установлен
        7 => read_isr(PIC1_CMD) & 0x80 == 0,

        // IRQ15: настоящее, если ISR slave не пуст
        15 if read_isr(PIC2_CMD) & 0x80 != 0 => false,

        // IRQ15: ложное, но master всё равно ждёт EOI за проброс через IRQ2
        15 => {
            unsafe { outb(PIC1_CMD, PIC_EOI); }
            true
        }
        _ => false,
    }
}
