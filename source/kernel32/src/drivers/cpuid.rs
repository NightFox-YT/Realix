// © Realix > Driver: CPUID
// ================
// ❗️ Только то, что реально можно узнать без сторонних драйверов: вендор
// и "brand string" процессора через инструкцию CPUID. Ничего про GPU -
// это требует сканирования шины PCI, которого в kernel32 пока нет (см.
// commands::cliff::my_pc, которое честно об этом сообщает, а не выдумывает)

use core::arch::asm;

unsafe fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let mut eax = leaf;
    let mut ebx: u32 = 0;
    let mut ecx: u32 = 0;
    let mut edx: u32 = 0;
    unsafe {
        asm!(
            "cpuid",
            inout("eax") eax,
            out("ebx") ebx,
            inout("ecx") ecx,
            out("edx") edx,
        );
    }
    (eax, ebx, ecx, edx)
}

/// Строка вендора (12 ASCII-символов, напр. "GenuineIntel"/"AuthenticAMD")
/// - CPUID лист 0 отдаёт её как ebx:edx:ecx (именно в этом порядке)
pub fn vendor_string() -> [u8; 12] {
    let (_, ebx, ecx, edx) = unsafe { cpuid(0) };
    let mut buf = [0u8; 12];
    buf[0..4].copy_from_slice(&ebx.to_le_bytes());
    buf[4..8].copy_from_slice(&edx.to_le_bytes());
    buf[8..12].copy_from_slice(&ecx.to_le_bytes());
    buf
}

/// "Brand string" (модель CPU, до 48 символов, обычно с хвостом из
/// пробелов) - листы 0x80000002-0x80000004, доступны только если
/// максимальный расширенный лист (0x80000000) >= 0x80000004
pub fn brand_string() -> Option<[u8; 48]> {
    let (max_extended, _, _, _) = unsafe { cpuid(0x8000_0000) };
    if max_extended < 0x8000_0004 {
        return None;
    }

    let mut buf = [0u8; 48];
    for (i, leaf) in (0x8000_0002u32..=0x8000_0004u32).enumerate() {
        let (eax, ebx, ecx, edx) = unsafe { cpuid(leaf) };
        let offset = i * 16;
        buf[offset..offset + 4].copy_from_slice(&eax.to_le_bytes());
        buf[offset + 4..offset + 8].copy_from_slice(&ebx.to_le_bytes());
        buf[offset + 8..offset + 12].copy_from_slice(&ecx.to_le_bytes());
        buf[offset + 12..offset + 16].copy_from_slice(&edx.to_le_bytes());
    }
    Some(buf)
}
