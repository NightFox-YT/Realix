use core::sync::atomic::{AtomicUsize, Ordering};
use crate::pci;
use core::arch::asm;

static mut IO_BASE: u16 = 0;

use core::sync::atomic::AtomicU16;

// Приёмный буфер: 8192 + 16 байт + выравнивание по 4K
// Должен быть выровнен по 8 байт (спецификация RTL8139C+)
#[repr(align(4096))]
struct RxBuf([u8; 8192 + 16 + 4096]);

static mut RX_BUF: RxBuf = RxBuf([0; 8192 + 16 + 4096]);
static RX_PTR: AtomicU16 = AtomicU16::new(0);

/// Инициализация RTL8139
pub fn init() -> bool {
    if let Some((_bus, _dev, _func, bar0)) = pci::find_device(0x10EC, 0x8139) {
        unsafe {
            IO_BASE = bar0 as u16;
        }

        // 1. Включаем карту (выход из режима low-power)
        write_config1(0x00);

        // 2. Программный сброс
        write_cr(0x10);
        while read_cr() & 0x10 != 0 {}

        // 3. Настройка приёмного буфера (RBSTART)
        let rx_phys = unsafe { RX_BUF.0.as_ptr() as u32 };
        write_rbstart(rx_phys);

        // 4. Конфигурация приёмника (RCR)
        //    AAP  = 0x01 — Promiscuous (принимать все, даже не нам)
        //    APM  = 0x02 — Accept Physical Match (unicast)
        //    AM   = 0x04 — Accept Multicast
        //    AB   = 0x08 — Accept Broadcast
        //    AR   = 0x10 — Accept Runt (< 64 байт) — критично для ARP
        //    Также устанавливаем бит 7 (WRAP) — кольцевой буфер 8K
        //    Итого: 0x1F
        write_rcr(0x1F);

        // 5. CAPR = 0 (после сброса уже 0, явно не помешает)
        write_reg16(0x38, 0x0000);

        // 6. Включаем приёмник (RE=0x08) и передатчик (TE=0x04)
        write_cr(0x0C);

        // 7. Запрещаем все прерывания (IMR=0) — работаем через polling
        write_reg16(0x3C, 0x0000);

        return true;
    }
    false
}

/// Читает MAC-адрес (6 байт) в буфер
pub fn get_mac(buf: &mut [u8; 6]) {
    for i in 0..6 {
        buf[i] = read_mac_byte(i);
    }
}

/// Проверяет, инициализирована ли карта
pub fn is_ready() -> bool {
    unsafe { IO_BASE != 0 }
}

// --- Приватные функции ---

fn read_mac_byte(offset: usize) -> u8 {
    unsafe {
        let port = IO_BASE + offset as u16;
        let result: u8;
        asm!("in al, dx", out("al") result, in("dx") port);
        result
    }
}

fn read_cr() -> u8 {
    unsafe {
        let port = IO_BASE + 0x37;
        let result: u8;
        asm!("in al, dx", out("al") result, in("dx") port);
        result
    }
}

fn write_cr(val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") IO_BASE + 0x37u16, in("al") val);
    }
}

fn write_config1(val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") IO_BASE + 0x52u16, in("al") val);
    }
}

fn write_rbstart(addr: u32) {
    unsafe {
        asm!("out dx, eax", in("dx") IO_BASE + 0x30u16, in("eax") addr);
    }
}

fn write_rcr(val: u32) {
    unsafe {
        asm!("out dx, eax", in("dx") IO_BASE + 0x44u16, in("eax") val);
    }
}

fn write_reg16(off: u16, val: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") IO_BASE + off, in("ax") val);
    }
}

fn read_reg16(off: u16) -> u16 {
    unsafe {
        let v: u16;
        asm!("in ax, dx", out("ax") v, in("dx") IO_BASE + off);
        v
    }
}

/// Отправка сырого Ethernet-пакета
static NEXT_TX_DESC: AtomicUsize = AtomicUsize::new(0);

pub fn send_packet(data: &[u8]) {
    unsafe {
        let port = IO_BASE;

        let desc_idx = NEXT_TX_DESC.load(Ordering::Relaxed);
        let tsd_port  = port + 0x10 + (desc_idx * 4) as u16;
        let tsad_port = port + 0x20 + (desc_idx * 4) as u16;

        let phys_addr = data.as_ptr() as u32;

        // Устанавливаем физический адрес пакета
        asm!("out dx, eax", in("dx") tsad_port, in("eax") phys_addr);
        // Размер пакета (бит 0 = OWN, автосброс после отправки)
        asm!("out dx, eax", in("dx") tsd_port, in("eax") data.len() as u32);

        // Ждём завершения передачи
        loop {
            let status: u32;
            asm!("in eax, dx", out("eax") status, in("dx") tsd_port);
            if status & 0x8000 != 0 { break; } // TOK
        }

        // Очищаем TOK в ISR
        write_reg16(0x3E, 0x0004);

        NEXT_TX_DESC.store((desc_idx + 1) % 4, Ordering::Relaxed);
    }
}

/// Приём пакета. Использует прямой polling кольцевого буфера (без ISR/прерываний).
/// Сравнивает CAPR (регистр 0x38) с RX_PTR — если различаются, есть новые данные.
/// Возвращает Some(длина) или None если нет новых пакетов.
pub fn receive_packet(buf: &mut [u8; 2048]) -> Option<usize> {
    unsafe {
        // 1. Читаем CAPR (Current Address of Packet Read) — регистр 0x38
        //    Карта обновляет его при записи каждого нового пакета в буфер.
        let capr = read_reg16(0x38);
        let rx_ptr = RX_PTR.load(Ordering::Relaxed);

        // 2. Если CAPR == наш указатель — карта ничего нового не записала
        if capr == rx_ptr {
            return None;
        }

        // Адрес буфера и текущая позиция
        let rx_addr = RX_BUF.0.as_ptr();
        let ptr = rx_ptr as usize;

        // 3. Читаем 4-байтный заголовок пакета:
        //    bits 0–15:  статус (ROK=0x0001, FAE, CRC, RUNT, LONG и т.д.)
        //    bits 16–31: длина пакета (включая сам заголовок)
        let header = core::ptr::read_unaligned(rx_addr.add(ptr) as *const u32);
        let status = (header & 0xFFFF) as u16;
        let length = ((header >> 16) & 0xFFFF) as usize;

        // 4. Проверяем ROK (Receive OK) — бит 0 в статусе заголовка
        if status & 0x0001 == 0 {
            // Пакет ещё не готов или битый — НЕ продвигаем указатель
            return None;
        }

        // 5. Проверяем валидность длины
        if length < 4 || length > 8192 {
            // Битый пакет — сбрасываем CAPR, пропускаем
            write_reg16(0x38, capr);
            RX_PTR.store(capr, Ordering::Relaxed);
            return None;
        }

        // 6. Копируем данные в буфер пользователя
        //    length включает 4 байта заголовка
        let data_len = length - 4;
        let copy_len = if data_len > buf.len() { buf.len() } else { data_len };
        let src = rx_addr.add(ptr + 4);
        for i in 0..copy_len {
            buf[i] = core::ptr::read_volatile(src.add(i));
        }

        // 7. Вычисляем следующий указатель (округление до 4 байт)
        //    Формула из спецификации: next = (current + length + 4 + 3) & !3
        let next = ((ptr + length + 4 + 3) & !3) % 8192;

        // 8. Обновляем RX_PTR и CAPR
        RX_PTR.store(next as u16, Ordering::Relaxed);
        // CAPR = CBA − 16, но мы просто двигаем его за прочитанным пакетом
        write_reg16(0x38, next as u16);

        // 9. Очищаем ROK в ISR (на случай, если прерывания когда-нибудь включатся)
        write_reg16(0x3E, 0x0001);

        Some(copy_len)
    }
}