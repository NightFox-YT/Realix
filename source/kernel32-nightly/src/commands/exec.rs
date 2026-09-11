// © Realix > Command: exec (.RLX loader, 32-bit)
// (07.09.26) v0.12
// ================
// ❗️ Зависимости: commands::disk, fs::fat12, x86::syscall
// ❗️ Приложение выполняется в Ring 0 (том же кольце, что и ядро) - в GDT уже есть
//    Ring3-дескрипторы (gdt.rs), но переключение колец для настоящей изоляции
//    вынесено за рамки этого прохода (см. README, "Upcoming Features")

// Подключение функций
use crate::commands::disk;
use crate::drivers::vga::{self, Color};
use crate::fs::fat12;
use crate::x86::syscall;

// Формат заголовка совпадает с shared/config.asm (RLX_MAGIC / RLX_MODE_32)
const RLX_MAGIC: u16 = 0xFC26;
const RLX_MODE_32: u16 = 0x32;
const HEADER_SIZE: usize = 8;

// ❗️ Физический адрес загрузки .RLX-приложения. ДОЛЖЕН совпадать с `org` в apps/*.asm
//    для 32-битных приложений (см. apps/hello32.asm) - как RLX16_APP_SEGMENT для 16-бит,
//    только тут это фиксированный физический адрес (Плоская модель памяти, а не сегмент).
//    Выбран заведомо далеко от kernel32 (~0x10000), стека (0x90000) и региона BIOS/VRAM
//    (0xA0000-0x100000): DMA-ограничения floppy-драйвера тут не действуют, т.к. сектор
//    сперва читается во внутренний буфер floppy.rs и копируется сюда обычным memcpy.
const RLX32_LOAD_ADDR: usize = 0x300000;
const APP_BUFFER_SIZE: usize = 1024 * 1024;

// Сигнатура точки входа приложения (Вызывается как обычная `call`, приложение
// обязано завершиться `ret`, попадая обратно сюда же)
// ❗️ "C" ABI => приложение ОБЯЗАНО сохранить и восстановить EBX/ESI/EDI/EBP
//    (callee-saved регистры), если использует их - иначе после возврата сюда
//    компилятор может прочитать испорченные значения (см. apps/hello32.asm, apps/rfetch.asm:
//    `push ebx/esi/edi/ebp` в начале app_entry, `pop` перед `ret`)
type EntryFn = unsafe extern "C" fn();

/// Команда запуска .RLX приложения по имени файла с диска: exec <имя файла>
pub fn run(args: &str) {
    let (bpb, entry) = match disk::resolve(args, "[?] Usage: exec <filename>\n") {
        Ok(v) => v,
        Err(msg) => { vga::print_line(msg, Color::LightGray); return; }
    };

    if entry.size as usize > APP_BUFFER_SIZE {
        vga::print_line("[!] File too large for the app buffer\n", Color::Red);
        return;
    }

    // ❗️ Пишем прямо по фиксированному физическому адресу (Плоская модель памяти,
    //    paging выключен => линейный адрес == физический). В отличие от 16-битного
    //    file_load, тут нет проверки пересечения с защищёнными регионами памяти -
    //    адрес заведомо выбран далеко от кода/стека ядра и BIOS/VRAM (см. константу выше)
    let dest: &mut [u8] = unsafe {
        core::slice::from_raw_parts_mut(RLX32_LOAD_ADDR as *mut u8, APP_BUFFER_SIZE)
    };
    let written: usize = match fat12::read_file(&bpb, &entry, dest) {
        Some(n) => n,
        None => {
            vga::print_line("[!] Disk read error or corrupted FAT chain\n", Color::Red);
            return;
        }
    };

    if written < HEADER_SIZE {
        vga::print_line("[! | RLX Loader] File too small to be a valid .RLX\n", Color::Red);
        return;
    }

    let magic: u16 = u16::from_le_bytes([dest[0], dest[1]]);
    let mode: u16 = u16::from_le_bytes([dest[2], dest[3]]);
    let entry_offset: u16 = u16::from_le_bytes([dest[4], dest[5]]);

    if magic != RLX_MAGIC {
        vga::print_line("[! | RLX Loader] Invalid .RLX header magic!\n", Color::Red);
        return;
    }
    if mode != RLX_MODE_32 {
        vga::print_line("[! | RLX Loader] Not a 32-bit .RLX binary!\n", Color::Red);
        return;
    }
    // Точка входа обязана попадать в реально загруженные данные файла
    if entry_offset as usize >= written {
        vga::print_line("[! | RLX Loader] Entry point outside loaded file!\n", Color::Red);
        return;
    }

    vga::print_line("[+ | RLX32 Loader] Executing 32-bit application...\n", Color::LightGray);
    unsafe { syscall::APP_RUNNING = true; }

    let entry_addr: usize = RLX32_LOAD_ADDR + entry_offset as usize;
    let entry_fn: EntryFn = unsafe { core::mem::transmute(entry_addr) };
    unsafe { entry_fn(); }

    vga::new_line_if_needed();
    vga::print_line("[+ | RLX32 Loader] Application finished.\n", Color::LightGray);
}
