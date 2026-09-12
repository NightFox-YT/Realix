/* ACPI parser for Realix, parse tables: MADT and FADT
    Copyright(C) 2026 Alexander Silaev <thebinaryblob@gmail.com>
*/

use core::ptr;
use core::slice;
use core::str;
use crate::x86::local_apic;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct AcpiHeader {
    pub sign: [u8; 4],
    pub len: u32,
    pub rev: u8,
    pub chcksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub crtor_id: u32,
    pub crtor_rev: u32,
}

impl AcpiHeader {
    pub fn sigstr(&self) -> &str 
    {
        str::from_utf8(&self.sign).unwrap_or("????")
    }

    pub fn validate_checksum(&self) -> bool 
    {
        let bytes = unsafe {
            slice::from_raw_parts(self as *const AcpiHeader as *const u8, self.len as usize)
        };
        bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)) == 0
    }
}

// RSDP для ACPI 1.0
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RSDPDescriptor {
    pub sign:       [u8; 8], // "RSD PTR "
    pub chcksum:    u8,
    pub oem_id:     [u8; 6],
    pub rev:        u8,
    pub rsdt_addr:  u32,
}

#[repr(C, packed)]
pub struct MADTHeader {
    pub header:             AcpiHeader,
    pub local_apic_address: u32,
    pub flags:              u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MADTEntryHeader {
    pub entry_type: u8,
    pub len:        u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MADTLocalAPIC {
    pub header: MADTEntryHeader,
    pub procid: u8,
    pub apicid: u8,
    pub flags:  u32, // Bit 0 - Enabled
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MADTInterruptOverride {
    pub header:     MADTEntryHeader,
    pub bus:        u8,
    pub source:     u8,
    pub gsi:        u32,
    pub flags:      u16,
}

#[derive(Debug, Default)]
pub struct SystemCPUInfo {
    pub total_cores:    u16,
    pub lapic_base:     u32,
    pub core_apic_ids:  [u8; 32], 
}

#[derive(Debug)]
pub struct SystemHadrwareInfo {
    pub cpu:        SystemCPUInfo,
    pub ioapic:     u32,
    pub isa_irq:    [u8; 16],
}

impl SystemCPUInfo {
    pub fn acpi_parse_madt(madt_ptr: *const MADTHeader) -> Self
    {
        let mut info = SystemCPUInfo::default();
        let madt = unsafe { &*madt_ptr };

        info.lapic_base = madt.local_apic_address;

        let totlen = madt.header.len as usize;
        let headlen = core::mem::size_of::<MADTHeader>();

        let mut offset = headlen;
        let base_ptr = madt_ptr as *const u8;

        while offset + core::mem::size_of::<MADTEntryHeader>() <= totlen 
        {
            let entry = unsafe { &*(base_ptr.add(offset) as *const MADTEntryHeader) };

            if entry.len == 0 { break; }

            if entry.entry_type == 0 
            {
                let lapic = unsafe { &*(base_ptr.add(offset) as *const MADTLocalAPIC) };

                if (lapic.flags & 1) != 0 {
                    if (info.total_cores as usize) < info.core_apic_ids.len() {
                        info.core_apic_ids[info.total_cores as usize] = lapic.apicid;
                        info.total_cores += 1;
                    }
                }
            }

            offset += entry.len as usize;
        }
        info
    }
}



#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GenericAddressStructure {
    pub addrspace: u8, // 0 - SysMem, 1 - SysIO
    pub bit_width: u8,
    pub bit_offst: u8,
    pub accessize: u8,
    pub poaddress: u64, // idk why but i insert po only for align, love it. 
}

#[repr(C, packed)]
pub struct FADTHeader {
    pub header:               AcpiHeader,
    pub firmware_ctrl:        u32,
    pub dsdt:                 u32,
    pub reserved:             u32,
    pub preferred_pm_profile: u8,
    pub sci_interrupt:        u16,
    pub smi_command_port:     u32,
    pub acpi_enable:          u8,
    pub acpi_disable:         u8,
    pub s4bios_req:           u8,
    pub pstate_control:       u8,
    pub pm1a_event_block:     u32,
    pub pm1b_event_block:     u32,
    pub pm1a_cntrl_block:     u32,
    pub pm1b_cntrl_block:     u32,
    pub pm_timer_len:         u8,
    pub gpe0_len:             u8,
    pub gpe1_len:             u8,
    pub gpe1_base:            u8,
    pub cstate_control:       u8,
    pub worst_c2_latency:     u16,
    pub worst_c3_latency:     u16,
    pub flush_size:           u16,
    pub flush_stride:         u16,
    pub duty_offset:          u8,
    pub duty_width:           u8,
    pub day_alarm:            u8,
    pub month_alarm:          u8,
    pub century:              u8,
    pub boot_arch_flags:      u16,
    pub reserved2:            u8,
    pub flags:                u32,
    /* ACPI 2.0 Reset Register */
    pub reset_register:       GenericAddressStructure,
    pub reset_value:          u8,
}

pub struct PowerManager {
    pub pm1a_cnt:           u16,
    pub reset_register:     Option<(u16, u8)>, // IO Port, Value
}

impl PowerManager {
    pub fn from_fadt(fadt: &FADTHeader) -> Self
    {
        let reset_info = if fadt.header.rev >= 2 && fadt.reset_register.addrspace == 1 {
            Some((fadt.reset_register.poaddress as u16, fadt.reset_value))
        } else {
            None
        };

        Self {
            pm1a_cnt: fadt.pm1a_cntrl_block as u16,
            reset_register: reset_info,
        }
    }

    pub unsafe fn sys_reboot(&self) -> !
    {
        if let Some((port, val)) = self.reset_register {
            core::arch::asm!("out dx, al", in("dx") port, in("al") val);
        }
        
        // фолбек на 8042 Keyboard Controller
        let mut good: u8 = 0x02;
        while (good & 0x02) != 0 {
            core::arch::asm!("in al, dx", out("al") good, in("dx") 0x64u16);
        }
        core::arch::asm!("out dx, al", in("dx") 0x64u16, in("al") 0xFEu8);

        // если ничего не работает, то отправляем систему в TripleFault
        core::arch::asm!("lidt [{}]", in(reg) &0u64);
        core::arch::asm!("int3");

        loop {}
    }

    pub unsafe fn sys_shutdown(&self) 
    {
        if self.pm1a_cnt != 0 {
            //вроде бы 5 << 10 для S5 state
            let shutdown_cmd: u16 = (5 << 10) | (1 << 13);
            core::arch::asm!("out dx, ax", in("dx") self.pm1a_cnt, in("ax") shutdown_cmd);
        }

        //фолбек для QEMU/Bochs/VB
        core::arch::asm!("out dx, ax", in("dx") 0x604u16, in("ax") 0x2000u16); // QEMU
        core::arch::asm!("out dx, ax", in("dx") 0xB004u16, in("ax") 0x2000u16); // Bochs
    }
}

pub struct ACPIParser {
    pub cpu_info: SystemCPUInfo,
    pub power: Option<PowerManager>,
}

impl ACPIParser {
    pub unsafe fn find_rsdp() -> Option<*const RSDPDescriptor>
    {
        let mut addr = 0xE0000usize;
        while addr < 0x100000 {
            let sig = ptr::read_volatile(addr as *const u64);
            
            if sig == 0x2052545020445352 // это если что "RSD PTR  " в LE
            {
                let rsdp = addr as *const RSDPDescriptor;
                if (*rsdp).rev == 0 { // ACPI 1.0
                    return Some(rsdp);
                }   
            }
            addr += 16;
        }
        None
    }

    pub unsafe fn init() -> Option<Self>
    {
        let rsdp_ptr = Self::find_rsdp()?;
        let rsdp = &*rsdp_ptr;

        let rsdt = rsdp.rsdt_addr as *const AcpiHeader;
        if !(*rsdt).validate_checksum() { return None; }

        let headache = core::mem::size_of::<AcpiHeader>();
        let entries_count = ((*rsdt).len as usize - headache) / 4;
        let entryptr = (rsdt as usize + headache) as *const u32;
        let mut cpu_info = SystemCPUInfo::default();
        let mut power_mgr = None;

        for i in 0..entries_count {
            let table_ptr = ptr::read_volatile(entryptr.add(i)) as *const AcpiHeader;
            if table_ptr.is_null() || !(*table_ptr).validate_checksum() { continue; }

            let sig = unsafe { table_ptr.as_ref() }.unwrap().sigstr();

            match sig {
                "APIC" => {
                    cpu_info = SystemCPUInfo::acpi_parse_madt(table_ptr as *const MADTHeader);
                }
                "FACP" => {
                    let fadt = &*(table_ptr as *const FADTHeader);
                    power_mgr = Some(PowerManager::from_fadt(fadt));
                }
                _ => {}
            }
        }

        Some(Self {
            cpu_info,
            power: power_mgr
        })
    }
}