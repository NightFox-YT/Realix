// © Realix > Cliff: приложение "Clock" (мировое время)
// ================
// ❗️ Читает drivers::rtc (CMOS, см. её заголовок про допущение UTC).
// Смещения городов - фиксированные целые часы+минуты, без учёта перехода
// на летнее время (нет базы часовых поясов - "ультра-базовое" приложение)
// ❗️ Обновляется только при перерисовке кадра (т.е. на любое нажатие
// клавиши, даже не используемое этим приложением) - в Cliff нет таймерного
// пробуждения главного цикла (read_key() блокирует до следующей клавиши),
// так что часы не "тикают" сами по себе, пока окно просто открыто

use crate::drivers::rtc;

pub const LINE_CAP: usize = 32;
pub const LINE_COUNT: usize = 12;

struct City { name: &'static str, offset_minutes: i32 }

const CITIES: [City; 9] = [
    City { name: "UTC",         offset_minutes: 0 },
    City { name: "London",     offset_minutes: 0 },
    City { name: "Berlin",     offset_minutes: 60 },
    City { name: "Moscow",     offset_minutes: 180 },
    City { name: "Dubai",      offset_minutes: 240 },
    City { name: "New Delhi",  offset_minutes: 330 },
    City { name: "Tokyo",      offset_minutes: 540 },
    City { name: "Sydney",     offset_minutes: 600 },
    City { name: "New York",   offset_minutes: -300 },
];

pub struct ClockApp;

impl ClockApp {
    pub fn new() -> Self {
        ClockApp
    }

    /// Строки для отображения: заголовок (UTC-дата+время из RTC), пустая
    /// строка, затем время для каждого города из CITIES (см. wrapped_time).
    /// Возвращает буферы, длину КАЖДОЙ строки (хвост буфера за длиной -
    /// нулевые байты, не текст) и общее кол-во заполненных строк
    pub fn lines(&self) -> ([[u8; LINE_CAP]; LINE_COUNT], [usize; LINE_COUNT], usize) {
        let now = rtc::now();
        let mut lines: [[u8; LINE_CAP]; LINE_COUNT] = [[0; LINE_CAP]; LINE_COUNT];
        let mut lens: [usize; LINE_COUNT] = [0; LINE_COUNT];
        let mut count = 0;

        {
            let mut w = LineWriter::new(&mut lines[count]);
            w.push_str("RTC (assumed UTC): ");
            w.push_num4(now.year);
            w.push_byte(b'-');
            w.push_num2(now.month);
            w.push_byte(b'-');
            w.push_num2(now.day);
            w.push_byte(b' ');
            w.push_num2(now.hour);
            w.push_byte(b':');
            w.push_num2(now.minute);
            w.push_byte(b':');
            w.push_num2(now.second);
            lens[count] = w.pos;
        }
        count += 1;
        count += 1; // пустая строка-разделитель (lines[1]/lens[1] остаются 0)

        for city in CITIES.iter() {
            if count >= LINE_COUNT { break; }
            let (hour, minute) = wrapped_time(&now, city.offset_minutes);

            let mut w = LineWriter::new(&mut lines[count]);
            w.push_str(city.name);
            for _ in city.name.len()..12 { w.push_byte(b' '); }
            w.push_str(": ");
            w.push_num2(hour);
            w.push_byte(b':');
            w.push_num2(minute);
            lens[count] = w.pos;
            count += 1;
        }

        (lines, lens, count)
    }
}

/// Время в городе со смещением `offset_minutes` от RTC (предполагаемого UTC),
/// с переносом через полночь по часам/минутам (без изменения даты - см.
/// заголовок файла)
fn wrapped_time(dt: &rtc::DateTime, offset_minutes: i32) -> (u8, u8) {
    let total = dt.hour as i32 * 60 + dt.minute as i32 + offset_minutes;
    let wrapped = ((total % 1440) + 1440) % 1440;
    ((wrapped / 60) as u8, (wrapped % 60) as u8)
}

/// Мини-writer для сборки строки в фиксированный буфер (без Vec/String)
struct LineWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> LineWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        LineWriter { buf, pos: 0 }
    }

    fn push_byte(&mut self, byte: u8) {
        if self.pos < self.buf.len() {
            self.buf[self.pos] = byte;
            self.pos += 1;
        }
    }

    fn push_str(&mut self, text: &str) {
        for &b in text.as_bytes() {
            self.push_byte(b);
        }
    }

    fn push_num2(&mut self, value: u8) {
        self.push_byte(b'0' + (value / 10) % 10);
        self.push_byte(b'0' + value % 10);
    }

    fn push_num4(&mut self, value: u16) {
        self.push_byte(b'0' + ((value / 1000) % 10) as u8);
        self.push_byte(b'0' + ((value / 100) % 10) as u8);
        self.push_byte(b'0' + ((value / 10) % 10) as u8);
        self.push_byte(b'0' + (value % 10) as u8);
    }
}
