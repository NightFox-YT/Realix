/* I/OAPIC the I/O APIC Realization for Realix
    Copyright(C) 2026 Alexander Silaev <thebinaryblob@gmail.com>
*/

use core::ptr::{read_volatile, write_volatile};

pub struct IoAPIC {
    base_virt: usize,
}

impl IoAPIC {
    pub fn new(base_virt: usize) -> Self {
        Self { base_virt }
    }

    #[inline(always)]
    fn read(&self, reg: u8) -> u32 {
        unsafe {
            let reg_sel = self.base_virt as *mut u32;
            let win = (self.base_virt + 0x10) as *const u32;
            write_volatile(reg_sel, reg as u32);
            read_volatile(win)
        }
    }

    #[inline(always)]
    fn write(&self, reg: u8, val: u32) {
        unsafe {
            let reg_sel = self.base_virt as *mut u32;
            let win = (self.base_virt + 0x10) as *mut u32;
            write_volatile(reg_sel, reg as u32);
            write_volatile(win, val);
        }
    }

    pub fn id(&self) -> u8 {
        ((self.read(0x00) >> 24) & 0x0F) as u8
    }

    /// Количество линий прерываний (Redirect Entries), обычно 24
    pub fn max_redirections(&self) -> u8 {
        (((self.read(0x01) >> 16) & 0xFF) + 1) as u8
    }

    /// Маппинг аппаратной линии (IRQ) на вектор IDT и целевое ядро CPU
    pub unsafe fn map_irq(&self, irq: u8, vector: u8, dest_apic_id: u8, mask: bool) {
        let low_reg = 0x10 + (irq * 2);
        let high_reg = low_reg + 1;

        // Нижележащие биты (low dword):
        // [0..7]   = Vector
        // [8..10]  = Delivery Mode (000 = Fixed)
        // [11]     = Destination Mode (0 = Physical)
        // [13]     = Pin Polarity (0 = Active High)
        // [15]     = Trigger Mode (0 = Edge)
        // [16]     = Mask (1 = Заблокировано, 0 = Включено)
        let mut low_val = vector as u32;
        if mask {
            low_val |= 1 << 16;
        }

        // Высокие биты (high dword): APIC ID ядра, принимающего IRQ
        let high_val = (dest_apic_id as u32) << 24;

        // Маскируем перед перенастройкой
        self.write(low_reg, low_val | (1 << 16));
        self.write(high_reg, high_val);
        self.write(low_reg, low_val);
    }

    pub unsafe fn unmask_irq(&self, irq: u8) {
        let reg = 0x10 + (irq * 2);
        let val = self.read(reg);
        self.write(reg, val & !(1 << 16));
    }

    pub unsafe fn disable_legacy_pic() 
    {
        core::arch::asm!("out dx, al", in("dx") 0x21u16, in("al") 0xFFu8);
        core::arch::asm!("out dx, al", in("dx") 0xA1u16, in("al") 0xFFu8);
    }
}