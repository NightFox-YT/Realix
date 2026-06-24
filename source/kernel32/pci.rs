use core::arch::asm;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// Читает 32-битное значение из PCI-конфигурации
fn pci_read(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
    let address: u32 = 
        0x80000000 |
        ((bus as u32) << 16) |
        (((device & 0x1F) as u32) << 11) |
        (((func & 0x07) as u32) << 8) |
        ((offset & 0xFC) as u32);
    
    unsafe {
        asm!("out dx, eax", in("dx") PCI_CONFIG_ADDRESS, in("eax") address);
        let result: u32;
        asm!("in eax, dx", out("eax") result, in("dx") PCI_CONFIG_DATA);
        result
    }
}

/// Ищет устройство по VendorID и DeviceID
/// Возвращает (bus, device, function, BAR0) или None
pub fn find_device(vendor: u16, device_id: u16) -> Option<(u8, u8, u8, u32)> {
    for bus in 0..2u8 {
        for device in 0..32u8 {
            for func in 0..8u8 {
                let id = pci_read(bus, device, func, 0x00);
                let vend = (id & 0xFFFF) as u16;
                let dev = ((id >> 16) & 0xFFFF) as u16;
                
                if vend == vendor && dev == device_id {
                    // Читаем BAR0 (смещение 0x10)
                    let bar0 = pci_read(bus, device, func, 0x10);
                    return Some((bus, device, func, bar0 & 0xFFFFFFFC));
                }
            }
        }
    }
    None
}
