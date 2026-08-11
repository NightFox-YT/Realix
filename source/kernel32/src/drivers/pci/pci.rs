/* Realix PCI realization
    Copyright(C) 2026 Alexander Silaev <thebinaryblob@gmail.com>
*/
use crate::utils;
use crate::drivers::vga::{self, Color};

/* ИИ подсказал мне отличную идею! Структуры, как я до этого не додумался?

*/
pub struct PCIDevice {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub vendorid: u16,
    pub david: u16,
    pub base_class: u8,
    pub sub_class: u8,
}

impl PCIDevice {
    pub fn pci_match_class(&self) -> &'static str /* Note: Data from wiki.osdev.org/PCI */
    {
        match (self.base_class, self.sub_class) {
            (0x00, 0x00) => "Non-VGA Compatible Unclassified Device",
            (0x00, 0x01) => "VGA-Compatible Unclassified Device",
            (0x01, 0x00) => "Mass Storage Controller: SCSI Bus",
            (0x01, 0x01) => "Mass Storage Controller: ATA(Parallel, IDE) Disk",
            (0x01, 0x02) => "Mass Storage Controller: Floppy",
            (0x01, 0x03) => "Mass Storage Controller: IPI Bus",
            (0x01, 0x04) => "Mass Storage Controller: RAID",
            (0x01, 0x05) => "Mass Storage Controller: ATA(Parallel, DMA)",
            (0x01, 0x06) => "Mass Storage Controller: ATA(Serial) Disk/Adapter",
            (0x01, 0x07) => "Mass Storage Controller: Serial Attached SCSI",
            (0x01, 0x08) => "Mass Storage Controller: Non-Volatile Memory(NVM)",
            (0x01, _)    => "Mass Storage Controller: Unknown",
            (0x02, 0x00) => "Network Controller: Ethernet",
            (0x02, 0x01) => "Network Controller: Token Ring",
            (0x02, 0x02) => "Network Controller: FDDI",
            (0x02, 0x03) => "Network Controller: ATM",
            (0x02, 0x04) => "Network Controller: ISDN",
            (0x02, 0x05) => "Network Controller: WorldFip",
            (0x02, 0x06) => "Network Controller: PICMG 2.14 Multi",
            (0x02, 0x07) => "Network Controller: InfiniBand",
            (0x02, 0x08) => "Network Controller: Fabric",
            (0x02, _)    => "Network Controller: Unknown",
            (0x03, 0x00) => "Display Controller: VGA Compatible Controller",
            (0x03, 0x01) => "Display Controller: XGA Controller",
            (0x03, 0x02) => "Display Controller: 3D Videocard, Non-VGA Compatible.",
            (0x03, _)    => "Display Controller: Unknown Controller",
            (0x04, 0x00) => "Multimedia Controller: Video Multimedia",
            (0x04, 0x01) => "Multimedia Controller: Audio Multimedia",
            (0x04, 0x02) => "Multimedia Controller: Telephony",
            (0x04, 0x03) => "Multimedia Controller: Audio",
            (0x04, _)    => "Multimedia Controller: Unknown",
            (0x05, 0x00) => "Memory Controller: RAM",
            (0x05, 0x01) => "Memory Controller: Flash",
            (0x05, _)    => "Memory Controller: Unknown",
            (0x06, 0x00) => "Bridge: Host Bridge",
            (0x06, 0x01) => "Bridge: ISA Bridge",
            (0x06, 0x02) => "Bridge: EISA Bridge",
            (0x06, 0x03) => "Bridge: MCA Bridge",
            (0x06, 0x04) => "Bridge: PCI2PCI Bridge",
            (0x06, 0x05) => "Bridge: PCMCIA Bridge",
            (0x06, 0x06) => "Bridge: NuBus: Bridge",
            (0x06, 0x07) => "Bridge: CardBus Bridge",
            (0x06, 0x08) => "Bridge: RACEway Bridge",
            (0x06, 0x09) => "Bridge: PCI2PCI Bridge(2)", /* я хз просто в osdev два PCI2PCI моста*/
            (0x06, 0x0A) => "Bridge: InfiniBand2PCI Host Bridge",
            (0x06, _)    => "Bridge: Unknown",
            (0x07, 0x00) => "SCC: Serial Controller", /* SCC -> Simple Communication Controller*/
            (0x07, 0x01) => "SCC: Parallel Controller",
            (0x07, 0x02) => "SCC: Multiport Serial Controller",
            (0x07, 0x03) => "SCC: Modem",
            (0x07, 0x04) => "SCC: IEEE 488.1/2 (GPIB) Controller",
            (0x07, 0x05) => "SCC: Smart Card Controller",
            (0x07, _)    => "SCC: Unknown",
            (0x08, 0x00) => "BSP: PIC", /* BSP -> Base System Peripheral*/
            (0x08, 0x01) => "BSP: DMA Controller",
            (0x08, 0x02) => "BSP: Timer",
            (0x08, 0x03) => "BSP: RTC Controller",
            (0x08, 0x04) => "BSP: PCI Hot-Plug Controller",
            (0x08, 0x05) => "BSP: SD Host Controller",
            (0x08, 0x06) => "BSP: IOMMU",
            (0x08, _)    => "BSP: Unknown",
            (0x09, 0x00) => "IDC: Keyboard", /* IDC -> Input Device Controller*/
            (0x09, 0x01) => "IDC: Digitizer Pen",
            (0x09, 0x02) => "IDC: Mouse",
            (0x09, 0x03) => "IDC: Scanner",
            (0x09, 0x04) => "IDC: Gameport",
            (0x09, _)    => "IDC: Unknown",
            (0x0A, 0x00) => "Docking Station",
            (0x0A, _)    => "Unknown Docking Station",
            (0x0B, 0x00) => "Processor: 386",
            (0x0B, 0x01) => "Processor: 486",
            (0x0B, 0x02) => "Processor: Intel(R) Pentium(TM)",
            (0x0B, 0x03) => "Processor: Intel(R) Pentium(TM) Pro",
            (0x0B, 0x10) => "Processor: Alpha",
            (0x0B, 0x20) => "Processor: IBM PowerPC",
            (0x0B, 0x30) => "Processor: MIPS",
            (0x0B, 0x40) => "Processor: Co-Processor",
            (0x0B, _)    => "Processor: Unknown",
            (0x0C, 0x00) => "SBC: FireWire", /* Serial Bus Controller aka. USB Controller*/
            (0x0C, 0x01) => "SBC: ACCESS Bus",
            (0x0C, 0x03) => "SBC: USB",
            (0x0C, 0x04) => "SBC: Fibre Channel",
            (0x0C, 0x05) => "SBC: SMBus",
            (0x0C, 0x06) => "SBC: InfiniBand",
            (0x0C, 0x07) => "SBC: IPMI Interface",
            (0x0C, 0x08) => "SBC: SERCOS Interface",
            (0x0C, 0x09) => "SBC: CANBus",
            (0x0C, _)    => "SBC: Unknown",
            (0x0D, 0x00) => "Wireless Controller: iRDA Compatible",
            (0x0D, 0x01) => "Wireless Controller: Consumer IR",
            (0x0D, 0x10) => "Wireless Controller: RF",
            (0x0D, 0x11) => "Wireless Controller: Bluetooth",
            (0x0D, 0x12) => "Wireless Controller: Broadband",
            (0x0D, 0x20) => "Wireless Controller: Ethernet(802.1a)",
            (0x0D, 0x21) => "Wireless Controller: Ethernet(802.1b)",
            (0x0D, _)    => "Wireless Controller: Unknown",
            (0x0E, 0x00) => "Intelligent Controller: I20",
            (0x0F, 0x01) => "Satellite Controller: TV",
            (0x0F, 0x02) => "Satellite Controller: Audio",
            (0x0F, 0x03) => "Satellite Controller: Voice",
            (0x0F, 0x04) => "Satellite Controller: Data",
            (0x10, 0x00) => "Encryption Controller: Network and Computing Enc/Dec",
            (0x10, 0x10) => "Encryption Controller: Entertainment Enc/Dec",
            (0x10, _)    => "Encryption Controller: Unknown",
            (0x11, 0x00) => "SPC: DPIO", /* SPC -> Signal Processing Controller*/
            (0x11, 0x01) => "SPC: Performance Counters",
            (0x11, 0x10) => "SPC: Communication Synchronizer",
            (0x11, 0x20) => "SPC: Signal Processing Management",
            (0x11, _)    => "SPC: Unknown",
            (0x12, _)    => "Processing Accelerator",
            (0x13, _)    => "Non-Essential Instrumentation",
            (0x14, _)    => "Reserved to 0x3F",
            (0x40, _)    => "Co-Processor",
            (0x41, _)    => "Reserved to 0xFE",
            _            => "Device Class Reserved or Unknown."
        }
    }
}

const PCI_CONFIG_ADDR: u32 = 0xCF8;
const PCI_CONFIG_DATA: u32 = 0xCFC;
const PCI_WRITE_INFO_ON_VGA_SCREEN: u8 = 23;
/* вывел в отдельную функцию потому что я не ИИ и мне впадлу писать одно и тоже два раза,
 не знаю зачем в си версии я делал по два раза
 */
fn pci_get_address(bus: u8, slot: u8, func: u8, offset: u8) -> u32
{
    let bobbus: u32 = (bus as u32) & 0xFF;
    let sunslop: u32 = (slot as u32) & 0x1F;
    let funfunc: u32 = (func as u32) & 0x07;
    let offsunset: u32 = (offset as u32) & 0xFC;

    let address: u32 =  (1       << 31) |
                        (bobbus  << 16) |
                        (sunslop << 11) |
                        (funfunc << 8)  |
                        offsunset;

    return address;
}

/* Чтение конфига PCI */
pub unsafe fn pci_read_config(bus: u8, slot: u8, func: u8, off: u8) -> u32
{
    let pciaddress: u32 = pci_get_address(bus, slot, func, off);

    utils::outl(PCI_CONFIG_ADDR, pciaddress);

    utils::io_wait();

    return utils::inl(PCI_CONFIG_DATA as u16);
}

/* Проверка на устройства */
pub unsafe fn pci_check_device(bus: u8, device: u8, flags: u8)
{
    let reg0: u32 = pci_read_config(bus, device, 0, 0);
    let vendorid: u16 = (reg0 & 0xFFFF) as u16;
    if vendorid == 0xFFFF || vendorid == 0x0000 { return; }

    let reg3: u32 = pci_read_config(bus, device, 0, 0x0C);
    let hedr_typo: u8 = ((reg3 >> 16) & 0xFF) as u8; /* header type*/

    let total_funcs: u8 = if (hedr_typo & 0x80) != 0 { 8 } else { 1 };
    for function in 0..total_funcs
    {
        let curreg0: u32 = pci_read_config(bus, device, function, 0);
        let curven = (curreg0 & 0xFFFF) as u16;
        let curdev: u16 = ((curreg0 >> 16) & 0xFFFF) as u16;

        if curven == 0xFFFF || curven == 0x0000 { continue; }

        let reg2: u32 = pci_read_config(bus, device, function, 0x08);
        let base_class: u8 = ((reg2 >> 24) & 0xFF) as u8;
        let sub_class: u8 = ((reg2 >> 16) & 0xFF) as u8;

        let pci_dev = PCIDevice {
            bus,
            slot: device,
            func: function,
            vendorid: curven,
            david: curven,
            base_class,
            sub_class,
        };

        let class_name = pci_dev.pci_match_class();

        if flags == PCI_WRITE_INFO_ON_VGA_SCREEN {
            vga::print_line("INFO: PCI Device Found:", Color::White);
            let mut num_buf = [0u8; 10];
            let vendor_str = utils::u32_to_hex_str(pci_dev.vendorid as u32, &mut num_buf);
            vga::print_line(vendor_str, Color::White);
            let device_str = utils::u32_to_hex_str(pci_dev.david as u32, &mut num_buf);
            vga::print_line(device_str, Color::White);
            vga::print_line(class_name, Color::White);
        }
    }
}

/* запись конфига */
pub unsafe fn pci_write_config(bus: u8, slot: u8, func: u8, off: u8, value: u32)
{
    let address: u32 = pci_get_address(bus, slot, func, off);

    utils::outl(PCI_CONFIG_ADDR, address);
    utils::io_wait();

    utils::outl(PCI_CONFIG_DATA, value);
    utils::io_wait();
}

pub unsafe fn pci_start() -> i16
{
    /* Честно не знаю зачем оно нужно, когда я пилил свой форк с RCOM, это было обязательно, сейчас нет.
        Но я оставлю на будущее.
    */
    for device in 0..32 {
        pci_check_device(0, device, 0);
    }
    0 /* Буду наверное разбираться в PCI Express 1.0, хотя не думаю что оно нужно ведь ОС делается под x86.
    И цель сейчас статическая линковка.*/
}