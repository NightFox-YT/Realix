use crate::vga;

/// Читает текущее время из RTC (CMOS)
pub fn read_time() -> (u8, u8, u8) {
    let hour = read_cmos(0x04);
    let minute = read_cmos(0x02);
    let second = read_cmos(0x00);
    (hour, minute, second)
}

/// Читает текущую дату
pub fn read_date() -> (u8, u8, u8, u8) {
    let century = read_cmos(0x32);
    let year = read_cmos(0x09);
    let month = read_cmos(0x08);
    let day = read_cmos(0x07);
    (century, year, month, day)
}

/// Выводит время на экран
pub fn print_time() {
    let (h, m, s) = read_time();
    vga::print_str("Time: ", vga::Color::Green);
    print_bcd(h);
    vga::put_char(b':', vga::Color::White);
    print_bcd(m);
    vga::put_char(b':', vga::Color::White);
    print_bcd(s);
    vga::put_char(b'\n', vga::Color::LightGray);
}

/// Выводит дату
pub fn print_date() {
    let (c, y, m, d) = read_date();
    vga::print_str("Date: 20", vga::Color::Green);
    print_bcd(y);
    vga::put_char(b'-', vga::Color::White);
    print_bcd(m);
    vga::put_char(b'-', vga::Color::White);
    print_bcd(d);
    vga::put_char(b'\n', vga::Color::LightGray);
}

fn read_cmos(reg: u8) -> u8 {
    unsafe {
        core::arch::asm!("out 0x70, al", in("al") reg);
        let result: u8;
        core::arch::asm!("in al, 0x71", out("al") result);
        result
    }
}

fn print_bcd(n: u8) {
    let high = (n >> 4) & 0x0F;
    let low = n & 0x0F;
    vga::put_char(b'0' + high, vga::Color::White);
    vga::put_char(b'0' + low, vga::Color::White);
}
