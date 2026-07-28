/* LAPIC the Local APIC Realization for Realix
    Copyright(C) 2026 Alexander Silaev <thebinaryblob@gmail.com>
*/

use core::ptr::{read_volatile, write_volatile};

#[repr(usize)]
pub enum LAPICReg
{
    Id = 0x020,
    Version = 0x030,
    Tpr = 0x080,         // Task Priority Register
    Eoi = 0x0B0,         // End of Interrupt
    Ldr = 0x0D0,         // Logical Destination Register
    Svr = 0x0F0,         // Spurious Interrupt Vector Register
    Isr0 = 0x100,        // In-Service Register
    Esr = 0x280,         // Error Status Register
    LvtTimer = 0x320,    // LVT Timer Register
    LvtLint0 = 0x350,    // Local Interrupt 0
    LvtLint1 = 0x360,    // Local Interrupt 1
    LvtError = 0x370,    // Local Interrupt Error
    TimerInitCount = 0x380,
    TimerCurrCount = 0x390,
    TimerDivConfig = 0x3E0,
}

pub struct LocalAPIC {
    base_virt: usize,
}

impl LocalAPIC {
    pub fn new(base_virt: usize) -> Self {
        Self { base_virt }
    }

    #[inline(always)]
    fn read(&self, reg: LAPICReg) -> u32 {
        unsafe { read_volatile((self.base_virt + reg as usize) as *const u32) }
    }

    #[inline(always)]
    fn write(&self, reg: LAPICReg, val: u32) {
        unsafe { write_volatile((self.base_virt + reg as usize) as *mut u32, val) }
    }

    pub unsafe fn init(&self, spurious_vector: u8) {
        // Очищаем Error Status Register (двойная запись сбрасывает старые ошибки)
        self.write(LAPICReg::Esr, 0);
        self.write(LAPICReg::Esr, 0);

        // Сбрасываем приоритет задач (принимаем все прерывания)
        self.write(LAPICReg::Tpr, 0);

        // Включаем Software APIC Enable (бит 8) + привязываем Spurious Vector
        let svr = self.read(LAPICReg::Svr);
        self.write(LAPICReg::Svr, svr | (1 << 8) | (spurious_vector as u32));
    }

    #[inline(always)]
    pub fn send_eoi(&self) {
        self.write(LAPICReg::Eoi, 0);
    }

    pub fn id(&self) -> u8 {
        ((self.read(LAPICReg::Id) >> 24) & 0xFF) as u8
    }

    pub unsafe fn setup_timer(&self, vector: u8, initial_count: u32, divider: u32) {
        // Делитель частоты (0x3 = делитель на 16)
        self.write(LAPICReg::TimerDivConfig, divider);

        // Бит 17 = 1 (Periodic mode) + Номер вектора IDT
        let timer_flags = (1 << 17) | (vector as u32);
        self.write(LAPICReg::LvtTimer, timer_flags);

        // Старт отсчета
        self.write(LAPICReg::TimerInitCount, initial_count);
    }
}