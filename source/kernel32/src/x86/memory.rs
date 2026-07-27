// © Realix > x86: Memory (E820 Map)
// (27.07.26) v0.1
// ================
// ❗️ Раскладка должна совпадать с bios-api/memory/high.asm

/// Максимум записей карты (Значение берёт E820_MAX_ENTRIES в high.asm)
pub const E820_MAX_ENTRIES: usize = 64;

/// Структура записи карты памяти E820 (24 байта)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct E820Entry {
    pub address: u64,     // Начало региона
    pub size: u64,        // Длина региона (байт)
    pub seg_type: u32,    // Тип региона (1 - свободный)
    pub attributes: u32,  // Расширенные атрибуты ACPI 3.X
}

/// Структура карты памяти E820
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct E820Map {
    pub entry_count: u16,
    pub map: [E820Entry; E820_MAX_ENTRIES],
}
