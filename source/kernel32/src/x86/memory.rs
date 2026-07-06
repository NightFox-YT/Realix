// © Realix > Memory (Map)
// (04.07.26) v0.08
// ================

/// Структура записи карты памяти E820
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct E820Entry {
    pub address: u64,
    pub size: u64,
    pub seg_type: u32,
    pub attributes: u32,
}

/// Структура карты памяти E820 (64 - запас)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct E820Map {
    pub entry_count: u16,
    pub map: [E820Entry; 64],
}
