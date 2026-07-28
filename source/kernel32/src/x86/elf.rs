/* ELF32 parser for RealixOS
    Copyright(C) 2026 Alexander Silaev <thebinaryblob@gmail.com>.
    Licensed under GNU GPLv3
*/
#![allow(non_camel_case_types)]

use core::arch::asm;

#[repr(C)]
pub struct vma_t {
    virt_start: u32,
    virt_end: u32,
    flags: u32,
    next: *mut vma_t,
}

#[repr(C)]
pub struct vmm_t {
    pub pgdir: *mut u32,
    vmas: *mut vma_t,
}

const VMA_READ: u32   = 1 << 0;
const VMA_WRITE: u32  = 1 << 1;
const VMA_EXEC: u32   = 1 << 2;
const VMA_USER: u32   = 1 << 4;

const PTE_PRESENT: u32 = 1 << 0;
const PTE_WRITE: u32   = 1 << 1;
const PTE_USER: u32    = 1 << 2;

extern "C" {
    static kernel_vmm: *mut vmm_t;

    fn vmm_mmap(vmm: *mut vmm_t, virt: u32, size: u32, flags: u32) -> u32;
    fn vmm_virt_to_phys(vmm: *mut vmm_t, virt: u32) -> u32;
    fn vmm_map_page(vmm: *mut vmm_t, virt: u32, phys: u32, flags: u32) -> bool;
    fn vmm_unmap_page(vmm: *mut vmm_t, virt: u32);
}

const KERNEL_TMP_WINDOW: u32 = 0xDE000000;

pub const EI_MAG0: usize = 0;
pub const EI_MAG1: usize = 1;
pub const EI_MAG2: usize = 2;
pub const EI_MAG3: usize = 3;
pub const EI_CLASS: usize = 4;
pub const EI_DATA: usize = 5;
pub const ELFCLASS32: u8 = 1;
pub const ELFDATA2LSB: u8 = 1; // Little Endian

pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNA: u32 = 2;
pub const PT_NTER: u32 = 3;
pub const PT_NOTE: u32 = 4;

pub const PF_X: u32 = 1; // Executable
pub const PF_W: u32 = 2; // Writable
pub const PF_R: u32 = 4; // Readable

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct ELF32_Ehead {
    pub e_ident:   [u8; 16],
    pub e_type:    u16,
    pub e_mach:    u16,
    pub e_vers:    u16,
    pub e_entry:   u32,
    pub e_phoff:   u32,
    pub e_shoff:   u32,
    pub e_flag:    u32,
    pub e_phentsz: u16,
    pub e_phnum:   u16,
    pub e_shentsz: u16,
    pub e_shnum:   u16,
    pub e_shstndx: u16, 
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct ELF32_Phead {
    pub p_type: u32,
    pub p_ofst: u32,
    pub p_vadr: u32, // мне впадлу что-то тут писать, старайтесь сами понять что я тут понаписал, но по стандарту System V ABI!!
    pub p_padr: u32,
    pub p_flsz: u32,
    pub p_mmsz: u32,
    pub p_flag: u32,
    pub p_algn: u32, // да мне приходится уродовать названия чтобы красива была, да и всё равно какая разница я же этот код написал и буду писать.
}

#[derive(Debug, PartialEq, Eq)]
pub enum ELFParseException {
    InvalidMagic,
    Not32Bit,
    NotLittleEndian,
    NotX86,
    BufferTooSmall,
    MemMapFailed,
}

pub struct ELF32Parser;

impl ELF32Parser {
    pub fn validate(elf_bytes: &[u8]) -> Result<&ELF32_Ehead, ELFParseException> 
    {
        if elf_bytes.len() < core::mem::size_of::<ELF32_Ehead>() 
        {
            return Err(ELFParseException::BufferTooSmall);
        }

        let header = unsafe { &*(elf_bytes.as_ptr() as *const ELF32_Ehead)};

        if header.e_ident[EI_MAG0] != 0x7F
            || header.e_ident[EI_MAG1] != b'E'
            || header.e_ident[EI_MAG2] != b'L'
            || header.e_ident[EI_MAG3] != b'F'
        {
            return Err(ELFParseException::InvalidMagic);
        }

        if header.e_ident[EI_CLASS] != ELFCLASS32 
        {
            return Err(ELFParseException::Not32Bit);
        }

        if header.e_ident[EI_DATA] != ELFDATA2LSB
        {
            return Err(ELFParseException::NotLittleEndian);
        }

        if header.e_mach != 3 
        {
            return Err(ELFParseException::NotX86);
        }

        Ok(header)
    }

    /* load_elf

        парам пам пам
        elf_bytes - байты бинарника
        map_memory - функция аллокатора

        ретёрн:
        адрес точки входа(e_entry)
    */
    pub unsafe fn load_elf<F>(elf_bytes: &[u8], mut memmap: F) -> Result<u32, ELFParseException>
    where
        F: FnMut(u32/*vadr*/, u32/*mmsz*/, u32/*flag(s)*/) -> Option<*mut u8>,
    {
        let header = Self::validate(elf_bytes)?;

        let phoff = header.e_phoff as usize;
        let phnum = header.e_phnum as usize;
        let phentsz = header.e_phentsz as usize;

        if phoff + (phnum * phentsz) > elf_bytes.len()
        {
            return Err(ELFParseException::BufferTooSmall);
        }

        for i in 0..phnum
        {
            let cur_phoff = phoff + (i * phentsz);
            let ph = &*(elf_bytes.as_ptr().add(cur_phoff) as *const ELF32_Phead);

            if ph.p_type == PT_LOAD
            {
                if ph.p_mmsz == 0 {
                    continue;
                }

                let destp = match memmap(ph.p_vadr, ph.p_mmsz, ph.p_flag)
                {
                    Some(ptr) => ptr,
                    None => return Err(ELFParseException::MemMapFailed),
                };

                let pageoff = (ph.p_vadr & 4095) as usize;
                let actoff = destp.add(pageoff);

                if ph.p_flsz > 0 
                {
                    let scoff = ph.p_ofst as usize;
                    let scptr = elf_bytes.as_ptr().add(scoff);

                    core::ptr::copy_nonoverlapping/*nonoverlarping*/(scptr, destp, ph.p_flsz as usize);
                }

                if ph.p_mmsz > ph.p_flsz {
                    let bsss = (ph.p_mmsz - ph.p_flsz) as usize;
                    let bss = destp.add(ph.p_flsz as usize);

                    core::ptr::write_bytes(bss, 0, bsss);
                }
            }
        }

        Ok(header.e_entry)
    }
}

pub unsafe fn load_elf_to_vmm(elf_bytes: &[u8], dest_vmm: *mut vmm_t) -> Result<u32, ELFParseException>
{
    let entry = ELF32Parser::load_elf(elf_bytes, |vadr, memsz, elf_flag| {

        let mut vma_flag = VMA_USER;
        if (elf_flag & PF_R) != 0 { vma_flag |= VMA_READ; }
        if (elf_flag & PF_W) != 0 { vma_flag |= VMA_WRITE; }
        if (elf_flag & PF_X) != 0 { vma_flag |= VMA_EXEC; }

        let allocated_vadr = vmm_mmap(dest_vmm, vadr, memsz, vma_flag);
        if allocated_vadr == 0 
        {
            return None;
        }

        let page_count = (memsz + 4095) / 4096;
        for i in 0..page_count {
            let offset = i * 4096;
            let tarphys = vmm_virt_to_phys(dest_vmm, allocated_vadr + offset);
            vmm_map_page(kernel_vmm, KERNEL_TMP_WINDOW + offset, tarphys, PTE_PRESENT | PTE_WRITE);
        }

        Some(KERNEL_TMP_WINDOW as *mut u8)
    })?;

    let header = ELF32Parser::validate(elf_bytes).unwrap();
    let phoff = header.e_phoff as usize;
    for i in 0..(header.e_phnum as usize) {
        let ph = &*(elf_bytes.as_ptr().add(phoff + i * header.e_phentsz as usize) as *const ELF32_Phead);
        if ph.p_type == PT_LOAD && ph.p_mmsz > 0 {
            let page_count = (ph.p_mmsz + 4095) / 4096;
            for p in 0..page_count {
                vmm_unmap_page(kernel_vmm, KERNEL_TMP_WINDOW + p * 4096);
            }
        }
    }

    flush_tlb();

    Ok(entry)
}

#[inline(always)]
unsafe fn flush_tlb()
{
    let cr3: u32;
    asm!("mov {0}, cr3", "mov cr3, {0}", out(reg) cr3, options(nostack, preserves_flags));
}