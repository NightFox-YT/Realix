// © Realix > Driver: Pata(IDE) on UDMA.
// (25.08.26) v0.1
// ================
// ❗️ Зависимости: PCI, memory/mmio, memory/memory

#![allow(dead_code)]

use core::ffi::c_void;
use core::ptr;
use crate::memory::memory::{kfree, kmalloc, kernel_vmm, vmm_virt_to_phys};
use crate::memory::mmio::MMIOPtr;
use crate::{inw, outl, inb, outb, io_wait};
use crate::drivers::pci;

const PCI_VENDOR_DEVICE: u8  = 0x00;
const PCI_COMMAND_STATUS: u8 = 0x04;
const PCI_CLASS_REV: u8 = 0x08;
const PCI_HEADER_MISC: u8 = 0x0C;
const PCI_BAR4: u8 = 0x20;
const PCI_MASS_STORAGE: u8 = 0x01;
const PCI_SUBCLASS_IDE: u8 = 0x01;
const PCI_CMD_IO_SPACE: u32 = 1 << 0;
const PCI_CMD_MEM_SPACE: u32 = 1 << 1;
const PCI_CMD_BUS_MSTR: u32 = 1 << 2;

pub struct PciAddr {
    pub bus:  u8,
    pub slot: u8,
    pub func: u8,
}

pub struct PciIdeInfo {
    pub addr:     PciAddr,
    pub bar4_raw: u32,
    pub io_space: bool,
}
/// Поиск IDE порта
pub unsafe fn ata_pci_find_ide() -> Option<PciIdeInfo>
{
    for bus in 0u16..256 
    {
        let bus = bus as u8;
        for slot in 0u8..32 {
            for func in 0u8..8 {
                let vendor_dev = pci::pci_read_config(bus, slot, func, PCI_VENDOR_DEVICE);
                let vendor_id = vendor_dev & 0xFFFF;
                if vendor_id == 0xFFFF {
                    if func == 0 { break; }
                    continue;
                }

                let class_rev = pci::pci_read_config(bus, slot, func, PCI_CLASS_REV);
                let class_code = ((class_rev >> 24) & 0xFF) as u8;
                let subclass = ((class_rev >> 16) & 0xFF) as u8;

                if class_code == PCI_MASS_STORAGE && subclass == PCI_SUBCLASS_IDE {
                    let cmd = pci::pci_read_config(bus, slot, func, PCI_COMMAND_STATUS);
                    let ncmd = cmd | PCI_CMD_IO_SPACE | PCI_CMD_MEM_SPACE | PCI_CMD_BUS_MSTR;
                    pci::pci_write_config(bus, slot, func, PCI_COMMAND_STATUS, ncmd);

                    let bar4_raw = pci::pci_read_config(bus, slot, func, PCI_BAR4);
                    let io_space = (bar4_raw & 0x1) != 0;

                    return Some(PciIdeInfo {
                        addr: PciAddr { bus, slot, func },
                        bar4_raw,
                        io_space,
                    });
                }

                if func == 0 {
                    let header_misc = pci::pci_read_config(bus, slot, func, PCI_HEADER_MISC);
                    let header_type = ((header_misc >> 16) & 0xFF) as u8;
                    if header_type & 0x80 == 0 { break; }
                }
            }
        }
    }
    None
}

/// Получение адреса BAR4
fn bar4_addr(info: &PciIdeInfo) -> u32 {
    if info.io_space {
        info.bar4_raw & 0xFFFF_FFFC
    } else {
        info.bar4_raw & 0xFFFF_FFF0
    }
}

pub struct DMARegion {
    pub virt:   *mut u8,
    pub phys:   u32,
    pub len:    usize,
}

/// Аллокация DMA буфера
pub unsafe fn dma_alloc(len: usize) -> Result<DMARegion, AtaError> {
    let virt = kmalloc(len as u32) as *mut u8;
    if virt.is_null() { return Err(AtaError("kmalloc failed for DMA Buffer")); }
    let phys = vmm_virt_to_phys(kernel_vmm, virt as u32);
    Ok(DMARegion { virt, phys, len })
}
/// очищение DMA буфера
pub unsafe fn dma_free(region: DMARegion) {
    kfree(region.virt as *mut c_void);
}

pub const ATA_PRIMARY_IO:   u16 = 0x1F0;
pub const ATA_PRIMARY_CTRL: u16 = 0x3F6;
pub const ATA_SECONDY_IO:   u16 = 0x170;
pub const ATA_SECONDY_CTRL: u16 = 0x376;

mod reg {
    pub const DATA:             u16 = 0;
    pub const ERROR_FEATURES:   u16 = 1;
    pub const SECTOR_COUNT:     u16 = 2;
    pub const LBA_LO:           u16 = 3;
    pub const LBA_MID:          u16 = 4;
    pub const LBA_HI:           u16 = 5;
    pub const DRIVE_HEAD:       u16 = 6;
    pub const STATUS_CMD:       u16 = 7;
}

const ATA_SR_ERR: u8 = 1 << 0;
const ATA_SR_DRQ: u8 = 1 << 3;
const ATA_SR_DF:  u8 = 1 << 5;
const ATA_SR_BSY: u8 = 1 << 7;

const CMD_READ_DMA_EXT: u8 = 0x25;
const CMD_WRITE_DMA_EXT: u8 = 0x35;
const CMD_ID: u8 = 0xEC;
const CMD_SET_FEATURES: u8 = 0xEF;
const FEATURE_SET_TRANSFER_MODE: u8 = 0x03;

fn udma_transfer_mode_byte(mode: u8) -> u8 {
    0x40 | (mode & 0x07)
}

mod bm_reg {
    pub const COMMAND: u16      = 0x0;
    pub const STATUS: u16       = 0x2;
    pub const PRDT_ADDR: u16    = 0x4;
}

const BM_CMD_START:     u8 = 0x01;
const BM_CMD_STOP:      u8 = 0x00;
const BM_CMD_READ:      u8 = 0x01 << 3; // Bit 3 set = Read from PCI memory (Write to Disk)
const BM_CMD_WRITE:     u8 = 0x00 << 3; // Bit 3 clear = Write to PCI memory (Read from Disk)
const BM_STATUS_ACTIVE: u8 = 1 << 0;
const BM_STATUS_ERROR:  u8 = 1 << 1;
const BM_STATUS_IRQ:    u8 = 1 << 2;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PrdEntry {
    phys_addr:          u32,
    byte_count_and_eot: u32,
}

const PRD_MAX_ENTRIES: usize = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChanId {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DriveSelect {
    Master,
    Slave,
}

#[derive(Debug)]
pub struct AtaError(pub &'static str);

pub struct DataId {
    pub lba48_supp:         bool,
    pub udma_supp_mods:     u8,
    pub udma_active_mode:   Option<u8>,
    pub sector_count_48:    u64,
}

enum BmAccess {
    Io(u16),
    Mmio(u32),
}

pub struct AtaChan {
    iobase:     u16,
    ctrlbase:   u16,
    bm:         BmAccess,
    prdt:       DMARegion,
}

impl AtaChan {
    /// Создание нового экземпляра AtaChan
    pub unsafe fn new(chan: ChanId, ide: &PciIdeInfo) -> Result<Self, AtaError> 
    {
        let (iobase, ctrlbase, bmoff) = match chan {
            ChanId::Primary => (ATA_PRIMARY_IO, ATA_PRIMARY_CTRL, 0x00u32),
            ChanId::Secondary => (ATA_SECONDY_IO, ATA_SECONDY_CTRL, 0x08u32),
        };

        let bmbase = bar4_addr(ide) + bmoff;

        let bm = if ide.io_space {
            BmAccess::Io(bmbase as u16)
        } else {
            BmAccess::Mmio(bmbase)
        };

        let prdt = dma_alloc(PRD_MAX_ENTRIES * core::mem::size_of::<PrdEntry>())?;

        Ok(Self {
            iobase,
            ctrlbase,
            bm,
            prdt,
        })
    }

    /// Читает и возвращает статус
    unsafe fn read_status(&self) -> u8 
    {
        inb(self.iobase + reg::STATUS_CMD)
    }
    /// Альтернативно читает и возвращает статус
    unsafe fn read_alt_status(&self) -> u8 
    {
        inb(self.ctrlbase)
    }

    /// Ждать пока диск занят
    unsafe fn wait_not_busy(&self) -> Result<(), AtaError> 
    {
        for _ in 0..1_000_000u32 
        {
            if self.read_alt_status() & ATA_SR_BSY == 0 {
                return Ok(());
            }
            io_wait();
        }
        Err(AtaError("timeout waiting for BSY to clear"))
    }

    /// Ждать DRQ(Запрос данных) или ошибку
    unsafe fn wait_drq_or_error(&self) -> Result<(), AtaError> 
    {
        for _ in 0..1_000_000u32 
        {
            let s = self.read_alt_status();
            if s & ATA_SR_ERR != 0 || s & ATA_SR_DF != 0 {
                return Err(AtaError("device reported error"));
            }
            if s & ATA_SR_DRQ != 0 {
                return Ok(());
            }
            io_wait();
        }
        Err(AtaError("timeout waiting for DRQ"))
    }

    /// Выбор диска
    unsafe fn select_drive(&self, drive: DriveSelect) 
    {
        let drive_bit = match drive {
            DriveSelect::Master => 0x00,
            DriveSelect::Slave => 0x10,
        };
        outb(self.iobase + reg::DRIVE_HEAD, 0xE0 | drive_bit);
        io_wait();
    }

    /// Чтение и возвращение байта из базового модуля
    unsafe fn bm_read8(&self, offset: u16) -> u8 
    {
        match self.bm {
            BmAccess::Io(base) => inb(base.wrapping_add(offset)),
            BmAccess::Mmio(base) => {
                MMIOPtr::<u8>::new(base as usize + offset as usize).read()
            }
        }
    }

    /// Запись байта в базовый модуль
    unsafe fn bm_write8(&self, offset: u16, val: u8) 
    {
        match self.bm {
            BmAccess::Io(base) => outb(base.wrapping_add(offset), val),
            BmAccess::Mmio(base) => {
                MMIOPtr::<u8>::new(base as usize + offset as usize).write(val)
            }
        }
    }
    /// Запись long в базовый модуль
    unsafe fn bm_write32(&self, offset: u16, val: u32) 
    {
        match self.bm {
            BmAccess::Io(base) => outl(base.wrapping_add(offset), val),
            BmAccess::Mmio(base) => {
                MMIOPtr::<u32>::new(base as usize + offset as usize).write(val)
            }
        }
    }

    /// Идентификация, возвращает Result<Номер, ошибка>
    pub unsafe fn identify(
        &self,
        drive: DriveSelect,
        buf: &mut [u16; 256],
    ) -> Result<DataId, AtaError> 
    {
        self.select_drive(drive);

        outb(self.iobase + reg::SECTOR_COUNT, 0);
        outb(self.iobase + reg::LBA_LO, 0);
        outb(self.iobase + reg::LBA_MID, 0);
        outb(self.iobase + reg::LBA_HI, 0);
        outb(self.iobase + reg::STATUS_CMD, CMD_ID);

        if self.read_status() == 0 {
            return Err(AtaError("no drive present"));
        }

        self.wait_not_busy()?;
        self.wait_drq_or_error()?;

        for word in buf.iter_mut() {
            *word = inw(self.iobase + reg::DATA);
        }

        let lba48_supp = (buf[83] & (1 << 10)) != 0;
        let udma_word = buf[88];
        let udma_supp_mods = (udma_word & 0x7F) as u8;
        let udma_active_mask = ((udma_word >> 8) & 0x7F) as u8;
        let udma_active_mode = if udma_active_mask == 0 {
            None
        } else {
            Some(7 - udma_active_mask.leading_zeros() as u8)
        };

        let sector_count_48 = (buf[100] as u64)
            | ((buf[101] as u64) << 16)
            | ((buf[102] as u64) << 32)
            | ((buf[103] as u64) << 48);

        Ok(DataId {
            lba48_supp,
            udma_supp_mods,
            udma_active_mode,
            sector_count_48,
        })
    }
    /// Выставляет уровень UltraDMA
    pub unsafe fn set_udma_mode(&self, drive: DriveSelect, mode: u8) -> Result<(), AtaError> 
    {
        self.select_drive(drive);
        self.wait_not_busy()?;

        outb(self.iobase + reg::ERROR_FEATURES, FEATURE_SET_TRANSFER_MODE);
        outb(self.iobase + reg::SECTOR_COUNT, udma_transfer_mode_byte(mode));
        outb(self.iobase + reg::STATUS_CMD, CMD_SET_FEATURES);

        self.wait_not_busy()?;

        if self.read_status() & ATA_SR_ERR != 0 {
            return Err(AtaError("SET FEATURES (transfer mode) rejected by drive"));
        }
        Ok(())
    }

    /// Подготавливает контроллер к тому, куда именно писать(или считать)
    unsafe fn program_prdt(&self, buf_phys: u32, bytelen: usize) -> Result<(), AtaError> 
    {
        if bytelen == 0 || bytelen > 65536 {
            return Err(AtaError("buffer size invalid for single PRD entry"));
        }

        // В спецификации PRDT размер 65536 кодируется как 0x0000
        let count_field = if bytelen == 65536 { 0 } else { bytelen as u32 & 0xFFFF };

        let entry = PrdEntry {
            phys_addr: buf_phys,
            byte_count_and_eot: count_field | (1 << 31), // 31-й бит — EOT (End of Table)
        };

        ptr::write_volatile(self.prdt.virt as *mut PrdEntry, entry);
        self.bm_write32(bm_reg::PRDT_ADDR, self.prdt.phys);
        Ok(())
    }

    /// Ждёт пока передача не закончится.
    unsafe fn poll_transfer_complete(&self) -> Result<(), AtaError> 
    {
        for _ in 0..5_000_000u32 
        {
            let bm_status = self.bm_read8(bm_reg::STATUS);
            let ata_status = self.read_alt_status();

            if ata_status & ATA_SR_ERR != 0 || bm_status & BM_STATUS_ERROR != 0 {
                return Err(AtaError("DMA transfer error"));
            }
            if bm_status & BM_STATUS_ACTIVE == 0 && ata_status & ATA_SR_BSY == 0 {
                self.bm_write8(bm_reg::STATUS, bm_status | BM_STATUS_IRQ | BM_STATUS_ERROR);
                return Ok(());
            }
            io_wait();
        }
        Err(AtaError("timeout waiting for DMA completion"))
    }

    /// Чтение UDMA секторов
    pub unsafe fn ata_read_sectors_udma(
        &self,
        drive: DriveSelect,
        lba: u64,
        sector_count: u16,
    ) -> Result<DMARegion, AtaError> 
    {
        let buf = dma_alloc(sector_count as usize * 512)?;
        match self.issue_dma_command(
            drive,
            lba,
            sector_count,
            buf.phys,
            buf.len,
            CMD_READ_DMA_EXT,
            BM_CMD_WRITE, // При чтении с диска DMA пишет в RAM
        ) {
            Ok(()) => Ok(buf),
            Err(e) => {
                dma_free(buf);
                Err(e)
            }
        }
    }

    /// Запись UDMA секторов
    pub unsafe fn ata_write_sectors_udma(
        &self,
        drive: DriveSelect,
        lba: u64,
        sector_count: u16,
        src: &DMARegion,
    ) -> Result<(), AtaError> 
    {
        self.issue_dma_command(
            drive,
            lba,
            sector_count,
            src.phys,
            src.len,
            CMD_WRITE_DMA_EXT,
            BM_CMD_READ, // При записи на диск DMA читает из RAM
        )
    }

    /// Главный командующий, говорит что сделать.
    unsafe fn issue_dma_command(
        &self,
        drive:        DriveSelect,
        lba:          u64,
        sector_count: u16,
        buf_phys:     u32,
        buf_len:      usize,
        ata_cmd:      u8,
        bm_dir:       u8,
    ) -> Result<(), AtaError> 
    {
        if sector_count == 0 {
            return Err(AtaError("sector_count must be > 0"));
        }
        let expected_len = sector_count as usize * 512;

        if buf_len < expected_len {
            return Err(AtaError("buffer smaller than sector count * 512"));
        }

        self.wait_not_busy()?;
        self.select_drive(drive);
        self.wait_not_busy()?;

        self.program_prdt(buf_phys, expected_len)?;

        let lba_bytes = lba.to_le_bytes();
        let sc_bytes = sector_count.to_le_bytes();

        // Запись LBA48 (сначала старшие байты, затем младшие)
        outb(self.iobase + reg::SECTOR_COUNT, sc_bytes[1]);
        outb(self.iobase + reg::LBA_LO, lba_bytes[3]);
        outb(self.iobase + reg::LBA_MID, lba_bytes[4]);
        outb(self.iobase + reg::LBA_HI, lba_bytes[5]);

        outb(self.iobase + reg::SECTOR_COUNT, sc_bytes[0]);
        outb(self.iobase + reg::LBA_LO, lba_bytes[0]);
        outb(self.iobase + reg::LBA_MID, lba_bytes[1]);
        outb(self.iobase + reg::LBA_HI, lba_bytes[2]);

        // Очищаем активный трансфер и сбрасываем статус
        self.bm_write8(bm_reg::COMMAND, BM_CMD_STOP);
        let stale = self.bm_read8(bm_reg::STATUS);
        self.bm_write8(bm_reg::STATUS, stale | BM_STATUS_IRQ | BM_STATUS_ERROR);

        // Посылаем команду ATA
        outb(self.iobase + reg::STATUS_CMD, ata_cmd);
        io_wait();

        // Запускаем Bus Master DMA
        self.bm_write8(bm_reg::COMMAND, bm_dir | BM_CMD_START);

        self.poll_transfer_complete()?;

        self.bm_write8(bm_reg::COMMAND, BM_CMD_STOP);

        Ok(())
    }
}

pub struct AtaController {
    pub primary:   AtaChan,
    pub secondary: AtaChan,
}

/// Инициализация ATA драйвера в UltraDMA режиме
pub unsafe fn init_ata_udma() -> Result<AtaController, AtaError> 
{
    let ide = ata_pci_find_ide().ok_or(AtaError("no IDE controller found on PCI bus"))?;

    let primary = AtaChan::new(ChanId::Primary, &ide)?;
    let secondary = AtaChan::new(ChanId::Secondary, &ide)?;

    Ok(AtaController { primary, secondary })
}

/// Выставляет максимально возможный уровень UltraDMA(от этого зависит скорость диска)
pub unsafe fn negotiate_best_udma(
    channel: &AtaChan,
    drive: DriveSelect,
) -> Result<DataId, AtaError> 
{
    let mut buf = [0u16; 256];
    let id = channel.identify(drive, &mut buf)?;

    let max_supported_mode: u8 = 5;
    let mut best_mode: Option<u8> = None;
    for m in (0..=max_supported_mode).rev() {
        if id.udma_supp_mods & (1 << m) != 0 {
            best_mode = Some(m);
            break;
        }
    }

    let mode = best_mode.ok_or(AtaError("drive reports no supported UDMA mode"))?;
    channel.set_udma_mode(drive, mode)?;

    Ok(id)
}