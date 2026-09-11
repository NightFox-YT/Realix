// © Realix > Cliff: приложение "My PC" (сведения об оборудовании)
// ================
// ❗️ Показывает только то, что реально можно узнать в kernel32 без лишних
// драйверов: CPU (вендор/модель через CPUID) и память (тот же счётчик
// фреймов, что использует команда meminfo). Про GPU - честно "не
// определено", а не выдумка: для этого нужен драйвер PCI (перечисление
// шины, чтение Vendor/Device ID), которого в kernel32 пока нет

use crate::drivers::cpuid;
use crate::memory::frame_allocator;
use crate::utils;

pub const LINE_CAP: usize = 48;
pub const LINE_COUNT: usize = 6;

const BYTES_PER_MB: usize = 1024 * 1024;

pub struct MyPcApp;

impl MyPcApp {
    pub fn new() -> Self {
        MyPcApp
    }

    pub fn lines(&self) -> ([[u8; LINE_CAP]; LINE_COUNT], [usize; LINE_COUNT], usize) {
        let mut lines: [[u8; LINE_CAP]; LINE_COUNT] = [[0; LINE_CAP]; LINE_COUNT];
        let mut lens: [usize; LINE_COUNT] = [0; LINE_COUNT];
        let mut count = 0;

        {
            let vendor = cpuid::vendor_string();
            let vendor_str = core::str::from_utf8(&vendor).unwrap_or("?");
            let mut w = LineWriter::new(&mut lines[count]);
            w.push_str("CPU vendor: ");
            w.push_str(vendor_str);
            lens[count] = w.pos;
        }
        count += 1;

        {
            let mut w = LineWriter::new(&mut lines[count]);
            w.push_str("CPU model:  ");
            match cpuid::brand_string() {
                Some(brand) => {
                    let text = core::str::from_utf8(&brand).unwrap_or("?").trim();
                    w.push_str(text);
                }
                None => w.push_str("(brand string not supported)"),
            }
            lens[count] = w.pos;
        }
        count += 1;

        lens[count] = 0; // пустая строка-разделитель
        count += 1;

        {
            let total_mb = (frame_allocator::get_total_memory() / BYTES_PER_MB) as u32;
            let free_mb = (frame_allocator::get_free_memory() / BYTES_PER_MB) as u32;
            let mut total_buf = [0u8; 10];
            let total_str = utils::u32_to_dec_str(total_mb, &mut total_buf);
            let mut free_buf = [0u8; 10];
            let free_str = utils::u32_to_dec_str(free_mb, &mut free_buf);

            let mut w = LineWriter::new(&mut lines[count]);
            w.push_str("Memory: ");
            w.push_str(total_str);
            w.push_str(" MB total, ");
            w.push_str(free_str);
            w.push_str(" MB free");
            lens[count] = w.pos;
        }
        count += 1;

        {
            let mut w = LineWriter::new(&mut lines[count]);
            w.push_str("GPU: not detected (no PCI driver yet)");
            lens[count] = w.pos;
        }
        count += 1;

        (lines, lens, count)
    }
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

    fn push_str(&mut self, text: &str) {
        for &b in text.as_bytes() {
            if self.pos < self.buf.len() {
                self.buf[self.pos] = b;
                self.pos += 1;
            }
        }
    }
}
