// © Realix > Programmable Interrupt Controller
// Исправленная версия
// ===========================================

use crate::{inb, outb};


const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;

const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;

const PIC1_VECTOR_OFFSET: u8 = 0x20;
const PIC2_VECTOR_OFFSET: u8 = 0x28;


/// Небольшая задержка для старых PIC/ISA-устройств.
#[inline(always)]
unsafe fn io_wait() {
    /*
     * Порт 0x80 исторически использовался для POST-кодов и подходит
     * для короткой I/O-задержки на старом оборудовании.
     */
    outb(0x80, 0);
}


/// Переназначение IRQ на векторы 32–47.
pub fn remap() {
    unsafe {
        /*
         * Сохраняем исходные маски. Сейчас они не восстанавливаются,
         * поскольку ниже ядро устанавливает собственную конфигурацию.
         */
        let _old_master_mask = inb(PIC1_DATA);
        let _old_slave_mask = inb(PIC2_DATA);

        /* ICW1: начало инициализации, ожидается ICW4. */
        outb(PIC1_COMMAND, 0x11);
        io_wait();

        outb(PIC2_COMMAND, 0x11);
        io_wait();

        /* ICW2: смещения в IDT. */
        outb(PIC1_DATA, PIC1_VECTOR_OFFSET);
        io_wait();

        outb(PIC2_DATA, PIC2_VECTOR_OFFSET);
        io_wait();

        /*
         * ICW3:
         * slave подключён к IRQ2 master PIC;
         * идентификатор slave равен 2.
         */
        outb(PIC1_DATA, 0x04);
        io_wait();

        outb(PIC2_DATA, 0x02);
        io_wait();

        /* ICW4: режим 8086. */
        outb(PIC1_DATA, 0x01);
        io_wait();

        outb(PIC2_DATA, 0x01);
        io_wait();

        /*
         * Сначала маскируем все IRQ. Затем разрешаем:
         *   IRQ0 — PIT;
         *   IRQ1 — клавиатура;
         *   IRQ2 — cascade для slave PIC.
         */
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }

    unmask_irq(0);
    unmask_irq(1);
    unmask_irq(2);
}


/// Маскирование IRQ.
pub fn mask_irq(irq: u8) {
    unsafe {
        match irq {
            0..=7 => {
                let mask = inb(PIC1_DATA);
                outb(PIC1_DATA, mask | (1u8 << irq));
            }

            8..=15 => {
                let slave_irq = irq - 8;
                let mask = inb(PIC2_DATA);
                outb(PIC2_DATA, mask | (1u8 << slave_irq));
            }

            _ => {}
        }
    }
}


/// Снятие маски IRQ.
pub fn unmask_irq(irq: u8) {
    unsafe {
        match irq {
            0..=7 => {
                let mask = inb(PIC1_DATA);
                outb(PIC1_DATA, mask & !(1u8 << irq));
            }

            8..=15 => {
                let slave_irq = irq - 8;
                let mask = inb(PIC2_DATA);
                outb(PIC2_DATA, mask & !(1u8 << slave_irq));

                /*
                 * Для прохождения slave IRQ cascade-линия IRQ2 master PIC
                 * также должна быть разрешена.
                 */
                let master_mask = inb(PIC1_DATA);
                outb(PIC1_DATA, master_mask & !(1u8 << 2));
            }

            _ => {}
        }
    }
}


/// Отправка End Of Interrupt.
///
/// Для IRQ8–IRQ15 EOI сначала отправляется slave PIC и только затем
/// master PIC.
pub fn send_eoi(irq: u8) {
    if irq >= 16 {
        return;
    }

    unsafe {
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }

        outb(PIC1_COMMAND, PIC_EOI);
    }
}
