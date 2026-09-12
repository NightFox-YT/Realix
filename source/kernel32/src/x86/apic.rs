/* APIC a APIC System Realization for Realix, uses LAPIC, I/OAPIC, VMM, ACPI
    Copyright(C) 2026 Alexander Silaev <thebinaryblob@gmail.com>
*/

use crate::x86::acpi::ACPIParser;
use crate::memory::vmm::KernelVmm;
use crate::x86::local_apic::LocalAPIC;
use crate::x86::ioapic::IoAPIC;

pub struct APICSystem {
    pub lapic: LocalAPIC,
    pub ioapic: IoAPIC,
}

impl APICSystem {
    pub unsafe fn init() -> Option<Self> 
    {
        let acpi = ACPIParser::init()?;
        
        IoAPIC::disable_legacy_pic();

        let lapic_phys = acpi.cpu_info.lapic_base;
        let lapic_virt = KernelVmm::map_mmio_region(lapic_phys as u32, 4096).ok()?;

        let ioapic_phys = 0xFEC0_0000u32; 
        let ioapic_virt = KernelVmm::map_mmio_region(ioapic_phys, 4096).ok()?;

        let lapic = LocalAPIC::new(lapic_virt as usize);
        let ioapic = IoAPIC::new(ioapic_virt as usize);

        lapic.init(0xFF);

        let bsp_id = lapic.id();
        ioapic.map_irq(1, 0x21, bsp_id, false);

        Some(Self { lapic, ioapic })
    }
}