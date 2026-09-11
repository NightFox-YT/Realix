// © Realix > Driver: ISA DMA (Channel 2 - Floppy)
// (07.09.26) v0.12
// ================
// ❗️ 8-битный DMA-контроллер (8237): адресует только нижние 16 МБ,
//    буфер обмена не должен пересекать границу 64 КБ

// Подключение функций
use crate::utils::outb;

// Порты DMA-контроллера 1 (Каналы 0-3)
const DMA_MASK: u16 = 0x0A;
const DMA_CLEAR_FF: u16 = 0x0C;
const DMA_MODE: u16 = 0x0B;
const DMA_ADDR_CH2: u16 = 0x04;
const DMA_COUNT_CH2: u16 = 0x05;
const DMA_PAGE_CH2: u16 = 0x81;

// Номер канала DMA, закреплённый BIOS/железом за флоппи-контроллером
const CHANNEL2: u8 = 2;

// Команда "Write Single Mask Register": бит 2 - маска (1 - выключить канал)
const MASK_SET_CH2: u8 = CHANNEL2 | 0b100;
const MASK_CLEAR_CH2: u8 = CHANNEL2;

// > Байт режима: mode(7-6)=01 (single) | autoinit(4)=0 | direction(3-2) | channel(1-0)=10
const MODE_READ_CH2: u8 = 0b0100_0110;  // Периферия -> память (Чтение сектора с диска)
const MODE_WRITE_CH2: u8 = 0b0100_1010; // Память -> периферия (Запись сектора на диск)

/// Программирование канала 2 DMA на однократную передачу `len` байт
/// ❗️ `addr` обязан быть физическим адресом ниже 16 МБ; буфер размером `len`
///    не должен пересекать границу 64 КБ (Иначе передача "обернётся" в начало региона)
/// Параметры:
///  - addr: физический адрес буфера обмена
///  - len: размер передачи в байтах (1-65536)
///  - write: true - память -> периферия (Запись на диск), false - периферия -> память (Чтение)
pub fn setup_transfer(addr: u32, len: u16, write: bool) {
    let page: u8 = ((addr >> 16) & 0xFF) as u8;
    let offset: u16 = (addr & 0xFFFF) as u16;
    let count: u16 = len - 1; // Контроллер считает передачу от 0

    unsafe {
        // Маскируем канал на время настройки
        outb(DMA_MASK, MASK_SET_CH2);

        // Сброс триггера "старший/младший байт" перед адресом
        outb(DMA_CLEAR_FF, 0xFF);
        outb(DMA_ADDR_CH2, (offset & 0xFF) as u8);
        outb(DMA_ADDR_CH2, ((offset >> 8) & 0xFF) as u8);
        outb(DMA_PAGE_CH2, page);

        // Сброс триггера перед счётчиком байт
        outb(DMA_CLEAR_FF, 0xFF);
        outb(DMA_COUNT_CH2, (count & 0xFF) as u8);
        outb(DMA_COUNT_CH2, ((count >> 8) & 0xFF) as u8);

        // Режим передачи и снятие маски канала
        outb(DMA_MODE, if write { MODE_WRITE_CH2 } else { MODE_READ_CH2 });
        outb(DMA_MASK, MASK_CLEAR_CH2);
    }
}
