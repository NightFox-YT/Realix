// © Realix > Commands: Clock (time / date)
// (07.09.26) v0.12
// ================
// ❗️ Зависимости: drivers::rtc

// Подключение функций
use crate::drivers::rtc;
use crate::drivers::vga::{self, Color};
use crate::utils;

/// Печать числа с ведущим нулём, если оно меньше 10
fn print_2digit(value: u8) {
    let mut buf: [u8; 10] = [0u8; 10];

    if value < 10 {
        vga::print_char(b'0', Color::White);
    }
    vga::print_line(utils::u32_to_dec_str(value as u32, &mut buf), Color::White);
}

/// Команда вывода текущего времени (ЧЧ:ММ:СС) из CMOS RTC
pub fn time() {
    let now: rtc::DateTime = rtc::read();

    vga::print_line("Time: ", Color::LightGray);
    print_2digit(now.hours);
    vga::print_char(b':', Color::White);
    print_2digit(now.minutes);
    vga::print_char(b':', Color::White);
    print_2digit(now.seconds);
    vga::new_line();
}

/// Команда вывода текущей даты (ДД.ММ.ГГГГ) из CMOS RTC
pub fn date() {
    let now: rtc::DateTime = rtc::read();
    let mut buf: [u8; 10] = [0u8; 10];

    vga::print_line("Date: ", Color::LightGray);
    print_2digit(now.day);
    vga::print_char(b'.', Color::White);
    print_2digit(now.month);
    vga::print_char(b'.', Color::White);
    vga::print_line(utils::u32_to_dec_str(now.year as u32, &mut buf), Color::White);
    vga::new_line();
}
