// © Realix > Driver: Floppy Disk Controller (i8272A/82077AA)
// (07.09.26) v0.12
// ================
// ❗️ Прямая работа с контроллером через порты 0x3F0-0x3F7 + DMA канал 2
//    (BIOS int 0x13 недоступен в protected mode). Геометрия жёстко на
//    стандартный 3.5" 1.44 МБ диск (80 дорожек, 2 головки, 18 секторов) -
//    ровно то, что `mformat -f 1440` кладёт на образ этой ОС.
// ❗️ Только чтение (Запись на диск в проекте пока нигде не реализована)

// Подключение функций
use core::arch::asm;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use crate::drivers::{dma, pit};
use crate::utils::{inb, outb};

// Порты контроллера
const FDC_DOR: u16 = 0x3F2;  // Digital Output Register (запись)
const FDC_MSR: u16 = 0x3F4;  // Main Status Register (чтение)
const FDC_FIFO: u16 = 0x3F5; // Data FIFO (чтение/запись)
const FDC_CCR: u16 = 0x3F7;  // Configuration Control Register (запись)

// Биты Main Status Register
const MSR_RQM: u8 = 0x80; // FIFO готов к обмену байтом
const MSR_DIO: u8 = 0x40; // Направление: 1 - контроллер -> CPU, 0 - CPU -> контроллер

// Команды контроллера
const CMD_SPECIFY: u8 = 0x03;
const CMD_SENSE_INTERRUPT: u8 = 0x08;
const CMD_RECALIBRATE: u8 = 0x07;
const CMD_SEEK: u8 = 0x0F;
const CMD_READ_DATA: u8 = 0x06 | 0x40; // MFM=1, MT=0, SK=0

// Геометрия диска (Стандартный 3.5" 1.44 МБ)
const SECTORS_PER_TRACK: u32 = 18;
const HEADS: u32 = 2;
pub const SECTOR_SIZE: usize = 512;

// Значения DOR для управления мотором/сбросом (Диск 0)
const DOR_RESET: u8 = 0x00;              // Полный сброс (Все биты сняты)
const DOR_ENABLED: u8 = 0x0C;            // !RESET(2) | DMA/IRQ gate(3), мотор выкл.
const DOR_MOTOR_ON: u8 = 0x1C;           // + мотор диска 0(4)

// Ограничение ожидания готовности FIFO и прерывания (Защита от зависания)
const FIFO_TIMEOUT_SPINS: u32 = 5_000_000;
const IRQ_TIMEOUT_TICKS: u32 = 300; // ~3 сек. при PIT 100 Гц

// Флаг: контроллер уже инициализирован (`reset()` вызван)
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// Флаг получения IRQ6 (Выставляется из `irq_handler`, см. `on_irq6`)
static IRQ_FIRED: AtomicBool = AtomicBool::new(false);

// Буфер для DMA-обмена: заведомо ниже 16 МБ и не пересекает границу 64 КБ
// (Выравнивание 512 <= размера гарантирует последнее при size == 512)
#[repr(align(512))]
struct DmaBuffer([u8; SECTOR_SIZE]);
static mut DMA_BUFFER: DmaBuffer = DmaBuffer([0; SECTOR_SIZE]);

/// Вызывается из `irq_handler` (isr.rs) при получении IRQ6
pub fn on_irq6() {
    IRQ_FIRED.store(true, Relaxed);
}

/// "Взводит" ожидание IRQ6 - обязательно вызывать НЕПОСРЕДСТВЕННО перед командой,
/// которая его порождает (Иначе прерывание может прийти раньше `wait_for_irq`
/// и будет потеряно гонкой между отправкой команды и сбросом флага)
fn arm_irq() {
    IRQ_FIRED.store(false, Relaxed);
}

/// Ожидание прерывания IRQ6, "взведённого" через `arm_irq` (С ограничением по времени)
fn wait_for_irq() -> bool {
    let start: u32 = pit::get_ticks();

    while !IRQ_FIRED.load(Relaxed) {
        if pit::get_ticks().wrapping_sub(start) > IRQ_TIMEOUT_TICKS {
            return false;
        }
        unsafe { asm!("sti", "hlt", options(nostack, nomem)); }
    }

    true
}

/// Запись байта в FIFO (Ожидает готовности контроллера к приёму)
fn write_fifo(byte: u8) -> bool {
    for _ in 0..FIFO_TIMEOUT_SPINS {
        let msr: u8 = unsafe { inb(FDC_MSR) };
        if msr & MSR_RQM != 0 && msr & MSR_DIO == 0 {
            unsafe { outb(FDC_FIFO, byte); }
            return true;
        }
    }
    false
}

/// Чтение байта из FIFO (Ожидает готовности контроллера к выдаче)
fn read_fifo() -> Option<u8> {
    for _ in 0..FIFO_TIMEOUT_SPINS {
        let msr: u8 = unsafe { inb(FDC_MSR) };
        if msr & MSR_RQM != 0 && msr & MSR_DIO != 0 {
            return Some(unsafe { inb(FDC_FIFO) });
        }
    }
    None
}

/// Команда "Sense Interrupt Status": Считывает ST0 + текущий цилиндр
fn sense_interrupt() -> Option<(u8, u8)> {
    if !write_fifo(CMD_SENSE_INTERRUPT) {
        return None;
    }
    let st0: u8 = read_fifo()?;
    let cyl: u8 = read_fifo()?;
    Some((st0, cyl))
}

/// Команда "Specify": Параметры таймингов головки (Стандартные значения из спецификации)
fn specify() -> bool {
    write_fifo(CMD_SPECIFY)
        && write_fifo(0xDF) // SRT=3, HUT=0xF
        && write_fifo(0x02) // HLT=1, ND=0 (DMA-режим)
}

/// Включение мотора диска 0 и ожидание раскрутки
fn motor_on() {
    unsafe { outb(FDC_DOR, DOR_MOTOR_ON); }
    pit::sleep(300);
}

/// Выключение мотора (Контроллер остаётся включённым)
fn motor_off() {
    unsafe { outb(FDC_DOR, DOR_ENABLED); }
}

/// Полный сброс контроллера (Вызывается один раз перед первым обращением к диску)
fn reset() -> bool {
    arm_irq();

    unsafe {
        outb(FDC_DOR, DOR_RESET);
        for _ in 0..10_000 { core::hint::spin_loop(); }
        outb(FDC_DOR, DOR_ENABLED);
    }

    if !wait_for_irq() {
        return false;
    }

    // Сброс генерирует прерывание сразу для всех (до 4х) приводов - разбираем их все
    for _ in 0..4 {
        if sense_interrupt().is_none() {
            return false;
        }
    }

    // Скорость передачи 500 Кбит/с (Стандарт для 3.5" 1.44 МБ)
    unsafe { outb(FDC_CCR, 0x00); }

    if !specify() {
        return false;
    }

    recalibrate()
}

/// Команда "Recalibrate": Возврат головки на дорожку 0
fn recalibrate() -> bool {
    for _ in 0..2 {
        arm_irq();
        if !write_fifo(CMD_RECALIBRATE) || !write_fifo(0x00) {
            return false;
        }
        if !wait_for_irq() {
            return false;
        }

        match sense_interrupt() {
            Some((_st0, cyl)) if cyl == 0 => return true,
            _ => continue,
        }
    }
    false
}

/// Команда "Seek": Перемещение головки на дорожку `cylinder`
fn seek(cylinder: u8, head: u8) -> bool {
    arm_irq();
    if !write_fifo(CMD_SEEK) || !write_fifo(head << 2) || !write_fifo(cylinder) {
        return false;
    }
    if !wait_for_irq() {
        return false;
    }

    match sense_interrupt() {
        // Бит 5 ST0 (Seek End) должен быть выставлен, цилиндр должен совпасть
        Some((st0, cyl)) => st0 & 0x20 != 0 && cyl == cylinder,
        None => false,
    }
}

/// Перевод LBA в CHS (Cylinder/Head/Sector) для геометрии 1.44 МБ
fn lba_to_chs(lba: u32) -> (u8, u8, u8) {
    let cylinder: u32 = lba / (SECTORS_PER_TRACK * HEADS);
    let temp: u32 = lba % (SECTORS_PER_TRACK * HEADS);
    let head: u32 = temp / SECTORS_PER_TRACK;
    let sector: u32 = temp % SECTORS_PER_TRACK + 1; // Секторы нумеруются с 1

    (cylinder as u8, head as u8, sector as u8)
}

/// Чтение одного сектора (512 байт) по его LBA-номеру
/// Параметры:
///  - lba: линейный номер сектора (0 - загрузочный сектор)
///  - out: буфер назначения (Ровно 512 байт)
/// Вывод:
///  - true - успех (`out` заполнен), false - ошибка чтения/таймаут
pub fn read_sector(lba: u32, out: &mut [u8; SECTOR_SIZE]) -> bool {
    if !INITIALIZED.load(Relaxed) {
        if !reset() {
            return false;
        }
        INITIALIZED.store(true, Relaxed);
    }

    let (cylinder, head, sector) = lba_to_chs(lba);

    motor_on();

    if !seek(cylinder, head) {
        motor_off();
        return false;
    }

    let buf_addr: u32 = &raw const DMA_BUFFER as u32;
    dma::setup_transfer(buf_addr, SECTOR_SIZE as u16, false);

    arm_irq();
    let ok: bool = write_fifo(CMD_READ_DATA)
        && write_fifo(head << 2)
        && write_fifo(cylinder)
        && write_fifo(head)
        && write_fifo(sector)
        && write_fifo(2)    // Код размера сектора: 512 байт (128 << 2)
        && write_fifo(SECTORS_PER_TRACK as u8) // EOT: последний сектор дорожки
        && write_fifo(0x1B) // GAP3 (Стандарт для 3.5" 1.44 МБ)
        && write_fifo(0xFF); // DTL (Не используется при ненулевом коде размера)

    if !ok || !wait_for_irq() {
        motor_off();
        return false;
    }

    // 7 результатных байт: ST0, ST1, ST2, C, H, R, N
    let mut result: [u8; 7] = [0; 7];
    for slot in result.iter_mut() {
        match read_fifo() {
            Some(byte) => *slot = byte,
            None => { motor_off(); return false; }
        }
    }

    motor_off();

    // Успех: ни один из битов кода прерывания (ST0, биты 7-6) и ошибок ST1 не выставлен
    let success: bool = (result[0] & 0xC0 == 0) && result[1] == 0;
    if success {
        unsafe {
            let src: *const [u8; SECTOR_SIZE] = &raw const DMA_BUFFER.0;
            out.copy_from_slice(&*src);
        }
    }

    success
}
