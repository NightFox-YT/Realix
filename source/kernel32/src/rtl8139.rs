use core::sync::atomic::{AtomicUsize, Ordering};
use crate::pci;
use core::arch::asm;

static mut IO_BASE: u16 = 0;

/// Инициализация RTL8139
pub fn init() -> bool {
    // Ищем карту на PCI
    if let Some((_bus, _dev, _func, bar0)) = pci::find_device(0x10EC, 0x8139) {
        unsafe {
            IO_BASE = bar0 as u16;
        }
        
        // Включаем питание
        write_config1(0x00);
        
        // Сброс
        write_cr(0x10); // Reset
        while read_cr() & 0x10 != 0 {}
        
        // Настройка приёма
        set_rx_buffer(0x00070000);
        
        // Включаем приёмник и передатчик
        write_cr(0x0C); // RE + TE
        
        // Конфигурация приёма
        write_rcr(0x0000000A); // AB + AM
        
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

fn set_rx_buffer(addr: u32) {
    unsafe {
        asm!("out dx, eax", in("dx") IO_BASE + 0x30u16, in("eax") addr);
    }
}

fn write_rcr(val: u32) {
    unsafe {
        asm!("out dx, eax", in("dx") IO_BASE + 0x44u16, in("eax") val);
    }
}

/// Отправка сырого Ethernet-пакета
static NEXT_TX_DESC: AtomicUsize = AtomicUsize::new(0);

pub fn send_packet(data: &[u8]) {
    unsafe {
        let port = IO_BASE;
        
        let desc_idx = NEXT_TX_DESC.load(Ordering::Relaxed);
        let tsd_port = port + 0x10 + (desc_idx * 4) as u16;
        let tsad_port = port + 0x20 + (desc_idx * 4) as u16;
        
        let phys_addr = data.as_ptr() as u32;
        
        // Устанавливаем адрес
        asm!("out dx, eax", in("dx") tsad_port, in("eax") phys_addr);
        // Размер + старт передачи
        asm!("out dx, eax", in("dx") tsd_port, in("eax") data.len() as u32);
        
        // Ждём ТОЛЬКО наш дескриптор
        loop {
            let status: u32;
            asm!("in eax, dx", out("eax") status, in("dx") tsd_port);
            if status & 0x8000 != 0 {
                break;
            }
        }
        
        // Очищаем ISR (критически важно!)
        asm!("out dx, ax", in("dx") port + 0x3Eu16, in("ax") 0x0004u16);
        
        // Следующий дескриптор
        NEXT_TX_DESC.store((desc_idx + 1) % 4, Ordering::Relaxed);
    }
}
