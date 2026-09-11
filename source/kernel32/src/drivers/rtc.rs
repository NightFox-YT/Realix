// © Realix > Driver: RTC (CMOS Real-Time Clock)
// ================
// ❗️ kernel32 работает в защищённом режиме - BIOS-время (int 1Ah,
// см. bios-api/rtc.asm) недоступно. Читаем часы прямо из CMOS через порты
// 0x70 (индекс)/0x71 (данные) - как и остальные простые устройства
// (PIT/PIC/клавиатура) в этом ядре
// ❗️ Считаем, что железные часы выставлены в UTC (обычное умолчание для
// большинства ПК/ВМ) - см. commands::cliff::clock, который применяет
// фиксированные смещения для нескольких городов поверх этого значения.
// Без реальной базы часовых поясов - переход на летнее время не учитывается

use crate::utils::{inb, outb};

const CMOS_INDEX: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

const REG_SECONDS: u8 = 0x00;
const REG_MINUTES: u8 = 0x02;
const REG_HOURS: u8 = 0x04;
const REG_DAY: u8 = 0x07;
const REG_MONTH: u8 = 0x08;
const REG_YEAR: u8 = 0x09;
const REG_STATUS_A: u8 = 0x0A;
const REG_STATUS_B: u8 = 0x0B;

const STATUS_A_UPDATE_IN_PROGRESS: u8 = 0x80;
const STATUS_B_BINARY_MODE: u8 = 0x04;
const STATUS_B_24H_MODE: u8 = 0x02;

/// Момент времени, снятый с RTC (год - полный, считаем "20XX")
#[derive(Clone, Copy)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

fn read_reg(reg: u8) -> u8 {
    unsafe {
        outb(CMOS_INDEX, reg);
        inb(CMOS_DATA)
    }
}

fn bcd_to_bin(value: u8) -> u8 {
    (value & 0x0F) + ((value >> 4) * 10)
}

/// Снимок текущего времени RTC. Ждёт (с ограничением попыток), пока CMOS не
/// закончит внутреннее обновление (Status A, бит 7) - иначе поля могли бы
/// быть прочитаны наполовину обновлёнными и дать мусорное значение
pub fn now() -> DateTime {
    for _ in 0..100_000 {
        if read_reg(REG_STATUS_A) & STATUS_A_UPDATE_IN_PROGRESS == 0 {
            break;
        }
    }

    let status_b = read_reg(REG_STATUS_B);
    let binary = status_b & STATUS_B_BINARY_MODE != 0;

    let decode = |raw: u8| if binary { raw } else { bcd_to_bin(raw) };

    let raw_hour = read_reg(REG_HOURS);
    let pm = raw_hour & 0x80 != 0;
    let mut hour = decode(raw_hour & 0x7F);
    if status_b & STATUS_B_24H_MODE == 0 {
        hour %= 12;
        if pm { hour += 12; }
    }

    DateTime {
        year: 2000 + decode(read_reg(REG_YEAR)) as u16,
        month: decode(read_reg(REG_MONTH)),
        day: decode(read_reg(REG_DAY)),
        hour,
        minute: decode(read_reg(REG_MINUTES)),
        second: decode(read_reg(REG_SECONDS)),
    }
}
