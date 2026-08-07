// © Realix > Driver: PS/2 Mouse
// (28.07.26) v0.1
// ================
// Инициализация PS/2 мыши через контроллер 8042,
// обработка IRQ12, хранение глобального состояния (X, Y, кнопки)

use core::sync::atomic::{AtomicI32, AtomicU8, AtomicBool, Ordering::{Relaxed, Release}};
use crate::utils::{inb, outb};

// Порты контроллера 8042
const PS2_DATA:    u16 = 0x60;
const PS2_CMD:     u16 = 0x64;
const PS2_STATUS:  u16 = 0x64;

// Пределы экрана
pub const MOUSE_MAX_X: i32 = 79;
pub const MOUSE_MAX_Y: i32 = 24;

// Состояние мыши
static MOUSE_X:       AtomicI32 = AtomicI32::new(40);   // Начало по центру
static MOUSE_Y:       AtomicI32 = AtomicI32::new(12);
static MOUSE_BUTTONS: AtomicU8  = AtomicU8::new(0);
static MOUSE_READY:   AtomicBool = AtomicBool::new(false);

// Буфер для трёх байт пакета мыши
static mut MOUSE_PACKET: [u8; 3] = [0; 3];
static mut MOUSE_PACKET_IDX: u8 = 0;

/// Ждать пока контроллер готов принять данные (input buffer пуст)
unsafe fn wait_input() {
    let mut timeout = 100_000u32;
    while inb(PS2_STATUS) & 0x02 != 0 && timeout > 0 {
        timeout -= 1;
    }
}

/// Ждать пока есть данные для чтения (output buffer заполнен)
unsafe fn wait_output() {
    let mut timeout = 100_000u32;
    while inb(PS2_STATUS) & 0x01 == 0 && timeout > 0 {
        timeout -= 1;
    }
}

/// Отправить команду мыши через контроллер 8042
unsafe fn mouse_write(data: u8) {
    wait_input();
    outb(PS2_CMD, 0xD4);   // Следующий байт идет aux-порту (мышь)
    wait_input();
    outb(PS2_DATA, data);
}

/// Прочитать байт ответа
unsafe fn mouse_read() -> u8 {
    wait_output();
    inb(PS2_DATA)
}

/// Инициализация PS/2 мыши
pub fn init() {
    unsafe {
        // Включить aux-порт
        wait_input();
        outb(PS2_CMD, 0xA8);

        // Прочитать текущую конфигурацию контроллера
        wait_input();
        outb(PS2_CMD, 0x20);
        wait_output();
        let config = inb(PS2_DATA) | 0x02; // Установить бит 1 (IRQ12 enable)

        // Записать обновлённую конфигурацию
        wait_input();
        outb(PS2_CMD, 0x60);
        wait_input();
        outb(PS2_DATA, config);

        // Включить стандартный режим (F4 — Enable Data Reporting)
        mouse_write(0xF6); // Set Defaults
        mouse_read();      // ACK

        mouse_write(0xF4); // Enable streaming
        mouse_read();      // ACK

        MOUSE_READY.store(true, Release);
    }
}

/// Вызывается из irq_handler при IRQ12
pub fn on_irq12() {
    unsafe {
        let byte = inb(PS2_DATA);
        let idx = MOUSE_PACKET_IDX;

        MOUSE_PACKET[idx as usize] = byte;

        if idx == 2 {
            // Получили полный 3-байтовый пакет — обрабатываем
            process_packet();
            MOUSE_PACKET_IDX = 0;
        } else {
            // Первый байт: проверка бита валидности (бит 3 должен быть 1)
            if idx == 0 && (byte & 0x08) == 0 {
                return; // Несинхронизованный пакет — пропускаем
            }
            MOUSE_PACKET_IDX = idx + 1;
        }
    }
}

/// Разбор пакета мыши и обновление позиции
unsafe fn process_packet() {
    let flags  = MOUSE_PACKET[0];
    let dx_raw = MOUSE_PACKET[1] as i32;
    let dy_raw = MOUSE_PACKET[2] as i32;

    // Учитываем знаковые биты
    let dx = if flags & 0x10 != 0 { dx_raw - 256 } else { dx_raw };
    let dy = if flags & 0x20 != 0 { dy_raw - 256 } else { dy_raw };

    // Кнопки (бит 0 = левая, бит 1 = правая, бит 2 = средняя)
    MOUSE_BUTTONS.store(flags & 0x07, Release);

    // Обновление позиции с делением для замедления курсора в текстовом режиме
    let nx = (MOUSE_X.load(Relaxed) + dx / 8).clamp(0, MOUSE_MAX_X);
    let ny = (MOUSE_Y.load(Relaxed) - dy / 4).clamp(0, MOUSE_MAX_Y); // Y инвертирован

    MOUSE_X.store(nx, Release);
    MOUSE_Y.store(ny, Release);
}

/// Получить текущее состояние мыши
#[allow(dead_code)]
pub fn get_state() -> (i32, i32, u8) {
    (
        MOUSE_X.load(Relaxed),
        MOUSE_Y.load(Relaxed),
        MOUSE_BUTTONS.load(Relaxed),
    )
}

/// Проверить готовность мыши
pub fn is_ready() -> bool {
    MOUSE_READY.load(Relaxed)
}
