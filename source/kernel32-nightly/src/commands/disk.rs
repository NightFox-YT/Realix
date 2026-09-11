// © Realix > Commands: Disk helpers (общий пролог для ls/load/type/hexdump)
// (07.09.26) v0.12
// ================
// ❗️ Зависимости: fs::fat12

// Подключение функций
use crate::fs::fat12::{self, Bpb, DirEntry};

// Общий буфер назначения для load/type/hexdump (64 КБ - с запасом на все файлы образа)
pub const FILE_BUFFER_SIZE: usize = 64 * 1024;
pub static mut FILE_BUFFER: [u8; FILE_BUFFER_SIZE] = [0; FILE_BUFFER_SIZE];

/// Разбор имени файла из аргументов команды и поиск записи в корневом каталоге
/// Параметры:
///  - args: аргументы команды (после имени) - "<filename>"
/// Вывод:
///  - Err(msg) - сообщение об ошибке для печати ("[?] ..." / "[!] ...")
pub fn resolve(args: &str, usage: &'static str) -> Result<(Bpb, DirEntry), &'static str> {
    let name: &str = args.trim();
    if name.is_empty() {
        return Err(usage);
    }

    let name83: [u8; 11] = fat12::format_83(name).ok_or("[!] Invalid filename format (8.3)")?;
    let bpb: Bpb = fat12::read_bpb().ok_or("[!] Failed to read disk (boot sector)")?;
    let entry: DirEntry = fat12::find_file(&bpb, &name83).ok_or("[!] File not found")?;

    Ok((bpb, entry))
}

/// Загрузка содержимого файла в общий `FILE_BUFFER`
/// Вывод:
///  - Err(msg) - сообщение об ошибке
///  - Ok(n) - размер загруженных данных (Может быть обрезан по `FILE_BUFFER_SIZE`)
pub fn load_into_buffer(bpb: &Bpb, entry: &DirEntry) -> Result<usize, &'static str> {
    let dest: &mut [u8; FILE_BUFFER_SIZE] = unsafe { &mut *&raw mut FILE_BUFFER };

    fat12::read_file(bpb, entry, dest).ok_or("[!] Disk read error or corrupted FAT chain")
}
