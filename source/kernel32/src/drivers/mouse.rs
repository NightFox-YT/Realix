// © Realix > Driver: PS/2 Mouse
// ================
// ❗️ Используется только графическим Cliff (см. commands::cliff_gfx) - у
// текстового Cliff/shell нет понятия "координата курсора" во всём
// остальном интерфейсе
// ❗️ Стандартный PS/2-протокол (не IntelliMouse/колесо) - 3-байтовые
// пакеты: байт0 - флаги (переполнение X/Y, знак X/Y, всегда-1, ЛКМ/ПКМ/СКМ),
// байт1 - dx (доп. код, если установлен бит знака X), байт2 - dy (аналогично)
// ❗️ Контроллер 8042 (порты 0x60/0x64) обслуживает и клавиатуру, и мышь -
// см. init() про включение "вспомогательного устройства" (мыши) и её
// собственную команду "включить отчёты о движении"

use crate::utils::{inb, outb};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering::Relaxed};

const CTRL_DATA: u16 = 0x60;
const CTRL_CMD: u16 = 0x64;
const CTRL_STATUS: u16 = 0x64;

const STATUS_OUTPUT_FULL: u8 = 0x01;
const STATUS_INPUT_FULL: u8 = 0x02;

// Сборка 3-байтового пакета (см. заголовок файла)
static PACKET: [AtomicU8; 3] = [AtomicU8::new(0), AtomicU8::new(0), AtomicU8::new(0)];
static PACKET_INDEX: AtomicU8 = AtomicU8::new(0);

// Накопленное с последнего poll() смещение и текущее состояние ЛКМ
static PENDING_DX: AtomicI32 = AtomicI32::new(0);
static PENDING_DY: AtomicI32 = AtomicI32::new(0);
static LEFT_BUTTON: AtomicBool = AtomicBool::new(false);

fn wait_input_ready() {
    for _ in 0..100_000 {
        if unsafe { inb(CTRL_STATUS) } & STATUS_INPUT_FULL == 0 { return; }
    }
}

fn wait_output_ready() {
    for _ in 0..100_000 {
        if unsafe { inb(CTRL_STATUS) } & STATUS_OUTPUT_FULL != 0 { return; }
    }
}

fn write_ctrl_cmd(cmd: u8) {
    wait_input_ready();
    unsafe { outb(CTRL_CMD, cmd); }
}

fn write_data(byte: u8) {
    wait_input_ready();
    unsafe { outb(CTRL_DATA, byte); }
}

fn read_data() -> u8 {
    wait_output_ready();
    unsafe { inb(CTRL_DATA) }
}

/// Команда самой МЫШИ (не контроллеру) - идёт через префикс 0xD4
fn write_mouse(byte: u8) {
    write_ctrl_cmd(0xD4);
    write_data(byte);
}

/// Включение PS/2-мыши: разрешаем IRQ12 в Configuration Byte контроллера,
/// затем просим саму мышь присылать пакеты движения (Enable Data
/// Reporting). Безопасно вызывать, даже если мыши физически нет - просто
/// не придёт ни одного пакета/прерывания, poll() всегда будет отдавать (0,0,false)
pub fn init() {
    // ВАЖНО: interrupts_enable() в main.rs уже отработал до этого вызова -
    // без cli здесь клавиатурное прерывание (или что угодно ещё) могло бы
    // вклиниться посреди этого handshake с контроллером 8042 (Read/Write
    // Configuration Byte - чтение, модификация пары бит, запись обратно).
    // Контроллер один на клавиатуру И мышь - если что-то читает/пишет порты
    // 0x60/0x64 в СЕРЕДИНЕ этой последовательности, ответ на "прочитать
    // Configuration Byte" мог бы прийти не тем, что ожидается (напр.
    // самим скан-кодом клавиши), и тогда запись "изменённого" байта назад
    // реально писала бы мусор - в т.ч. потенциально СБРОСИТЬ бит0
    // (разрешение прерывания клавиатуры) и тем самым "заморозить" её вплоть
    // до следующего нажатия клавиши, каждое из которых лишь подтверждало бы
    // уже сломанное состояние. Именно так выглядела бы жалоба "любая клавиша
    // всё морозит, включая мышь"
    crate::x86::idt::interrupts_disable();

    // Включаем "вспомогательное устройство" (мышь)
    write_ctrl_cmd(0xA8);

    // Читаем Configuration Byte, взводим бит1 (разрешить IRQ12), снимаем
    // бит5 (включить тактирование мыши - изначально выключено), пишем назад
    write_ctrl_cmd(0x20);
    let mut config = read_data();
    config |= 0b0000_0010;
    config &= !0b0010_0000;
    write_ctrl_cmd(0x60);
    write_data(config);

    // Просим мышь присылать пакеты движения (Enable Data Reporting)
    write_mouse(0xF4);
    let ack = read_data();

    crate::x86::idt::interrupts_enable();

    // Разрешаем IRQ12, только если мышь реально подтвердила команду (0xFA) -
    // если её физически нет (или ответ не пришёл), не включаем прерывание
    // "вслепую" ради лишней защиты, а не потому что это САМО по себе
    // вызывало бы зависание
    if ack == 0xFA {
        crate::x86::pic::unmask_irq(12);
    }
}

/// Вызывается из irq_handler (isr.rs) на каждый байт, пришедший по IRQ12
pub fn on_byte(byte: u8) {
    let index = PACKET_INDEX.load(Relaxed);

    // Первый байт пакета ДОЛЖЕН иметь установленный бит 3 (всегда-1 в
    // стандартном протоколе) - если нет, поток байт рассинхронизирован
    // (напр. потерянное прерывание) - ждём как первый байт следующего пакета
    if index == 0 && byte & 0x08 == 0 {
        return;
    }

    PACKET[index as usize].store(byte, Relaxed);

    if index < 2 {
        PACKET_INDEX.store(index + 1, Relaxed);
        return;
    }
    PACKET_INDEX.store(0, Relaxed);

    let flags = PACKET[0].load(Relaxed);

    // Биты переполнения (6/7) - пакет гарантированно мусорный, пропускаем
    if flags & 0xC0 != 0 {
        return;
    }

    let raw_dx = PACKET[1].load(Relaxed);
    let raw_dy = PACKET[2].load(Relaxed);

    // Расширение знака по флагам X/Y (биты 4/5)
    let dx = if flags & 0x10 != 0 { raw_dx as i32 - 256 } else { raw_dx as i32 };
    let dy = if flags & 0x20 != 0 { raw_dy as i32 - 256 } else { raw_dy as i32 };

    PENDING_DX.fetch_add(dx, Relaxed);
    PENDING_DY.fetch_add(dy, Relaxed);
    LEFT_BUTTON.store(flags & 0x01 != 0, Relaxed);
}

/// Забирает накопленное смещение с прошлого вызова (dx, dy - ещё в
/// "мышиных" координатах: положительный dy = движение ВВЕРХ; экранные Y
/// растут вниз - переворот делает вызывающая сторона) и держится ли ЛКМ
pub fn poll() -> (i32, i32, bool) {
    let dx = PENDING_DX.swap(0, Relaxed);
    let dy = PENDING_DY.swap(0, Relaxed);
    let left = LEFT_BUTTON.load(Relaxed);
    (dx, dy, left)
}
