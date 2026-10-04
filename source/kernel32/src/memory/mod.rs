// © Realix > Memory mechanics Passport
// (25.06.26) v0.11
// ================
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
pub mod pmm;
pub mod frame_allocator;
pub mod vmm;
pub mod dma;
/// Получение количества свободной памяти в байтах
pub fn get_free_memory() -> usize {
    FREE_FRAMES.load(Relaxed) * FRAME_PAGE_SIZE
}

// Состояние аллокатора
pub static mut PMM_POOL_START: usize = 0;
pub static mut PMM_POOL_END: usize = 0;
pub const PMM_PAGE_SIZE: usize = 4096;

// Размер фрейма и границы адресного пространства
pub const FRAME_PAGE_SIZE: usize = 4096;                         // 4 КБ
pub const FRAME_MAX_PHYS_MEMORY: usize = 128 * 1024 * 1024;  // ! 2 ГБ для теста
pub const FRAME_COUNT: usize = FRAME_MAX_PHYS_MEMORY / FRAME_PAGE_SIZE;

// Битмап: 1 байт на 8 фреймов
pub const FRAME_BITMAP_SIZE: usize = FRAME_COUNT / 8;

// Первый 1 МБ резервируется целиком:
// IVT+BDA, загрузчик, kernel32, стек ядра, видеопамять и BIOS ROM
pub const RESERVED_LOW_MEMORY: usize = 1024 * 1024;

/// Тип свободного региона в карте памяти E820
pub const E820_TYPE_FREE: u32 = 1;

// Статический битмап фреймов: бит 0 - свободен, бит 1 - занят
// (Изначально всё занято: свобода выдаётся по карте памяти E820)
pub static mut FRAME_BITMAP: [u8; FRAME_BITMAP_SIZE] = [0xFF; FRAME_BITMAP_SIZE];
pub static TOTAL_FRAMES: AtomicUsize = AtomicUsize::new(0);
pub static FREE_FRAMES: AtomicUsize = AtomicUsize::new(0);

pub const DMA_MAX_ADDRESS: usize = 16 * 1024 * 1024; // 16 MB
pub const DMA_MAX_FRAME: usize = DMA_MAX_ADDRESS / FRAME_PAGE_SIZE;

/// Доступ к битмапу через raw-указатель
pub fn bitmap() -> &'static mut [u8; FRAME_BITMAP_SIZE] {
    unsafe { &mut *(&raw mut FRAME_BITMAP) }
}

/// Получение статистики: (всего фреймов, свободно фреймов)
pub fn get_stats() -> (usize, usize) {
    (TOTAL_FRAMES.load(Relaxed), FREE_FRAMES.load(Relaxed))
}

/// Получение количества всей полезной памяти в байтах
pub fn get_total_memory() -> usize {
    TOTAL_FRAMES.load(Relaxed) * FRAME_PAGE_SIZE
}

// > Вспомогательные функции (номер фрейма, без валидации)
pub fn is_bit_free(frame: usize) -> bool {
    (bitmap()[frame / 8] >> (frame % 8)) & 1 == 0
}

pub fn mark_frame_used(frame: usize) {
    bitmap()[frame / 8] |= 1 << (frame % 8);
}

pub fn mark_frame_free(frame: usize) {
    bitmap()[frame / 8] &= !(1 << (frame % 8));
}

/// Подсчёт свободных фреймов по битмапу
pub fn count_free_frames() -> usize {
    bitmap().iter().map(|byte| byte.count_zeros() as usize).sum()
}

/// Проверка, свободен ли фрейм по физическому адресу
pub fn is_frame_free(addr: usize) -> bool {
    // Проверка: адрес выходит за предел адресного пространства
    if addr >= FRAME_MAX_PHYS_MEMORY {
        return false;
    }
    is_bit_free(addr / FRAME_PAGE_SIZE)
}
use core::marker::PhantomData;
use core::ptr;
#[repr(transparent)]
pub struct MMIOPtr<T> {
    addr: usize,
    _phantom: PhantomData<*mut T>,
}

impl<T: Copy> MMIOPtr<T>
{
    pub const fn new(addr: usize) -> Self
    {
        unsafe
        {
            Self { addr, _phantom: PhantomData }
        }
    }

    #[inline(always)]
    pub fn read(&self) -> T
    {
        unsafe { ptr::read_volatile(self.addr as *const T) }
    }

    #[inline(always)]
    pub fn write(&self, val: T)
    {
        unsafe { ptr::write_volatile(self.addr as *mut T, val) }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut T
    {
        self.addr as *mut T
    }
}
