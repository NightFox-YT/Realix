// © Realix > Kernel32: Error Registry
// (19.09.26) v0.13
// ================
// ❗️ Единый реестр ошибок проекта (Rust-сторона).
//    Числа ДОЛЖНЫ совпадать с source/shared/errors.asm (ERR_*).
//    При добавлении нового кода — обновить оба файла и docs/errors.md.

/// Централизованный реестр кодов ошибок.
///
/// Нумерация сквозная по проекту (E1..EN) и совпадает с asm-версией.
/// Значение `self as u32` — это и есть код `ERR_*`.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    // --- Файловая система (FAT12) -----------------------------------------
    FileNotFound     = 1,   // E1
    BadCluster       = 2,   // E2
    GuardOverlap     = 3,   // E3
    BrokenChain      = 4,   // E4

    // --- Загрузчик / ранняя инициализация ---------------------------------
    LowerMemory      = 5,   // E5
    MemoryMap        = 6,   // E6
    DiskInit         = 7,   // E7

    // --- Питание -----------------------------------------------------------
    ShutdownFailed   = 8,   // E8

    // --- RTC ---------------------------------------------------------------
    RtcTime          = 9,   // E9
    RtcDate          = 10,  // E10

    // --- .RLX-загрузчик приложений ----------------------------------------
    RlxBadName       = 11,  // E11
    RlxMagic         = 12,  // E12
    RlxMode          = 13,  // E13
    RlxHeader        = 14,  // E14

    // --- Сеть --------------------------------------------------------------
    NicNotFound      = 15,  // E15
    NicResetTimeout  = 16,  // E16

    // --- Память ------------------------------------------------------------
    OutOfMemory      = 17,  // E17
    MapOutOfRange    = 18,  // E18

    // --- Ядро / syscall ----------------------------------------------------
    InvalidPcInfo    = 19,  // E19
    InvalidSyscall   = 20,  // E20
    InvalidArgument  = 21,  // E21
    PitFreqRange     = 22,  // E22
    UnsupportedVideo = 23,  // E23
    IoPort           = 24,  // E24
}

impl Error {
    /// Код ошибки в том виде, в котором он фигурирует в asm-реестре.
    #[inline]
    pub const fn code(self) -> u32 {
        self as u32
    }

    /// Человекочитаемое описание — то же, что и в asm-строке,
    /// но БЕЗ префикса `'[!] EN: '`.
    pub const fn message(self) -> &'static str {
        match self {
            Error::FileNotFound     => "File not found",
            Error::BadCluster       => "Bad cluster found",
            Error::GuardOverlap     => "Destination overlaps a protected region",
            Error::BrokenChain      => "Broken FAT chain",
            Error::LowerMemory      => "Get lower memory failed (int 12h)",
            Error::MemoryMap        => "Get memory map failed (int 15h)",
            Error::DiskInit         => "Disk init failed",
            Error::ShutdownFailed   => "PC shutdown failed (no APM)",
            Error::RtcTime          => "Failed to read RTC time",
            Error::RtcDate          => "Failed to read RTC date",
            Error::RlxBadName       => "Invalid filename format for FAT12",
            Error::RlxMagic         => "Invalid .RLX header magic",
            Error::RlxMode          => "Not a 16-bit .RLX binary",
            Error::RlxHeader        => "Corrupted .RLX header",
            Error::NicNotFound      => "RTL8139 not found on PCI bus",
            Error::NicResetTimeout  => "NIC reset timeout",
            Error::OutOfMemory      => "Out of memory",
            Error::MapOutOfRange    => "Address out of range",
            Error::InvalidPcInfo    => "Invalid PCINFO address",
            Error::InvalidSyscall   => "Unknown syscall",
            Error::InvalidArgument  => "Invalid argument",
            Error::PitFreqRange     => "PIT frequency out of range",
            Error::UnsupportedVideo => "Unsupported video mode",
            Error::IoPort           => "I/O port operation failed",
        }
    }

    /// Восстановление кода ошибки из числа (для syscall-возвратов,
    /// IRQ-статусов, диагностических снимков).
    /// Возвращает `None`, если код вне реестра.
    pub const fn from_code(code: u32) -> Option<Error> {
        Some(match code {
            1  => Error::FileNotFound,
            2  => Error::BadCluster,
            3  => Error::GuardOverlap,
            4  => Error::BrokenChain,
            5  => Error::LowerMemory,
            6  => Error::MemoryMap,
            7  => Error::DiskInit,
            8  => Error::ShutdownFailed,
            9  => Error::RtcTime,
            10 => Error::RtcDate,
            11 => Error::RlxBadName,
            12 => Error::RlxMagic,
            13 => Error::RlxMode,
            14 => Error::RlxHeader,
            15 => Error::NicNotFound,
            16 => Error::NicResetTimeout,
            17 => Error::OutOfMemory,
            18 => Error::MapOutOfRange,
            19 => Error::InvalidPcInfo,
            20 => Error::InvalidSyscall,
            21 => Error::InvalidArgument,
            22 => Error::PitFreqRange,
            23 => Error::UnsupportedVideo,
            24 => Error::IoPort,
            _  => return None,
        })
    }
}

/// Верхняя граница реестра — совпадает с `ERR_COUNT` в `errors.asm`.
pub const ERR_COUNT: u32 = 24;