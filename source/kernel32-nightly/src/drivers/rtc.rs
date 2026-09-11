// © Realix > Driver: RTC (CMOS Real-Time Clock)
// (07.09.26) v0.12
// ================
// ❗️ Прямой доступ к CMOS через порты 0x70/0x71 (BIOS int 0x1A недоступен в protected mode)

// Подключение функций
use crate::utils::{inb, outb};

// Порты CMOS
const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

// Регистры CMOS
const REG_SECONDS: u8 = 0x00;
const REG_MINUTES: u8 = 0x02;
const REG_HOURS: u8 = 0x04;
const REG_DAY: u8 = 0x07;
const REG_MONTH: u8 = 0x08;
const REG_YEAR: u8 = 0x09;
const REG_CENTURY: u8 = 0x32;
const REG_STATUS_A: u8 = 0x0A;
const REG_STATUS_B: u8 = 0x0B;

// Биты регистров статуса
const STATUS_A_UPDATE_IN_PROGRESS: u8 = 0x80;
const STATUS_B_BINARY_MODE: u8 = 0x04;
const STATUS_B_24_HOUR: u8 = 0x02;
const HOUR_PM_BIT: u8 = 0x80;

// Запасной век, если регистр 0x32 не поддерживается BIOS (0 -> считаем "20xx")
const DEFAULT_CENTURY: u8 = 20;

/// Снимок времени и даты, считанный из CMOS RTC (уже переведён в двоичный 24-часовой вид)
#[derive(Clone, Copy, Default)]
pub struct DateTime {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16, // Полный год (например 2026)
}

/// Чтение одного байта регистра CMOS
fn read_reg(reg: u8) -> u8 {
    unsafe {
        outb(CMOS_ADDRESS, reg);
        inb(CMOS_DATA)
    }
}

/// Ожидание завершения обновления часов контроллером (Update In Progress)
fn wait_update_complete() {
    while read_reg(REG_STATUS_A) & STATUS_A_UPDATE_IN_PROGRESS != 0 {
        core::hint::spin_loop();
    }
}

/// Перевод BCD-байта в обычное число
fn bcd_to_bin(value: u8) -> u8 {
    (value & 0x0F) + ((value >> 4) * 10)
}

/// Разовое чтение всех полей "как есть" (Формат ещё не известен)
fn read_raw() -> (u8, u8, u8, u8, u8, u8, u8) {
    (
        read_reg(REG_SECONDS),
        read_reg(REG_MINUTES),
        read_reg(REG_HOURS),
        read_reg(REG_DAY),
        read_reg(REG_MONTH),
        read_reg(REG_YEAR),
        read_reg(REG_CENTURY),
    )
}

/// Чтение текущего времени и даты из CMOS RTC
/// Вывод:
///  - Согласованный снимок `DateTime` (защита от чтения "на лету" во время тика)
pub fn read() -> DateTime {
    // Повторяем чтение, пока два последовательных снимка не совпадут
    let mut prev = read_raw();
    loop {
        wait_update_complete();
        let cur = read_raw();
        if cur == prev {
            break;
        }
        prev = cur;
    }

    let (raw_sec, raw_min, raw_hour, mut day, mut month, mut year, mut century) = prev;
    let status_b: u8 = read_reg(REG_STATUS_B);

    let is_bcd: bool = status_b & STATUS_B_BINARY_MODE == 0;
    let is_12h: bool = status_b & STATUS_B_24_HOUR == 0;

    let pm: bool = raw_hour & HOUR_PM_BIT != 0;
    let mut sec: u8 = raw_sec;
    let mut min: u8 = raw_min;
    let mut hour: u8 = raw_hour & !HOUR_PM_BIT;

    if is_bcd {
        sec = bcd_to_bin(sec);
        min = bcd_to_bin(min);
        hour = bcd_to_bin(hour);
        day = bcd_to_bin(day);
        month = bcd_to_bin(month);
        year = bcd_to_bin(year);
        century = bcd_to_bin(century);
    }

    // Перевод 12-часового формата (с PM-битом) в 24-часовой
    if is_12h {
        if hour == 12 {
            hour = 0;
        }
        if pm {
            hour += 12;
        }
    }

    // Некоторые BIOS не заполняют регистр века - используем запасной вариант
    let full_century: u16 = if century == 0 { DEFAULT_CENTURY as u16 } else { century as u16 };

    DateTime {
        seconds: sec,
        minutes: min,
        hours: hour,
        day,
        month,
        year: full_century * 100 + year as u16,
    }
}
