// © Realix > FAT12: Filesystem (kernel32)
// (07.09.26) v0.12
// ================
// ❗️ Зависимости: drivers::floppy
// ❗️ Только чтение. Портировано с bios-api/fat12/*.asm - геометрия и разметка
//    полностью совместимы (BPB читается напрямую с диска, а не хардкодится),
//    но нет BIOS - каждый сектор идёт через drivers::floppy::read_sector

// Подключение функций
use crate::drivers::floppy::{self, SECTOR_SIZE};

// Маркеры кластеров FAT12 (Совпадают с shared/config.asm)
const CHAIN_END: u16 = 0x0FF8;
const BAD_CLUSTER: u16 = 0x0FF7;

// Предел длины цепочки кластеров (Архитектурный потолок FAT12, как в file_load.asm)
const MAX_CHAIN_LENGTH: u32 = 4084;

// Максимум секторов FAT, которые кэшируем в памяти (С запасом на 1.44 МБ, где FAT = 9 секторов)
const MAX_FAT_SECTORS: usize = 12;

/// Параметры BPB (Bios Parameter Block), считанные из загрузочного сектора
#[derive(Clone, Copy)]
#[allow(dead_code)] // Хранится целиком по смыслу BPB, часть полей пока не читается
pub struct Bpb {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_count: u8,
    dir_entries: u16,
    sectors_per_fat: u16,
    root_dir_lba: u32,
    root_dir_size: u32, // В секторах
    data_lba: u32,
}

/// Запись корневого каталога (Уже разобранная)
#[derive(Clone, Copy)]
pub struct DirEntry {
    pub name: [u8; 11], // "Сырое" имя в формате 8.3 (Заглавные буквы, паддинг пробелами)
    pub first_cluster: u16,
    pub size: u32,
}

// Кэш таблицы FAT (Заполняется `load_fat`, один раз на операцию чтения файла)
static mut FAT_CACHE: [u8; MAX_FAT_SECTORS * SECTOR_SIZE] = [0; MAX_FAT_SECTORS * SECTOR_SIZE];

/// Чтение и разбор BPB из загрузочного сектора (LBA 0)
/// Вывод:
///  - None - ошибка чтения диска
pub fn read_bpb() -> Option<Bpb> {
    let mut sector0: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
    if !floppy::read_sector(0, &mut sector0) {
        return None;
    }

    let bytes_per_sector: u16 = u16::from_le_bytes([sector0[11], sector0[12]]);
    let sectors_per_cluster: u8 = sector0[13];
    let reserved_sectors: u16 = u16::from_le_bytes([sector0[14], sector0[15]]);
    let fat_count: u8 = sector0[16];
    let dir_entries: u16 = u16::from_le_bytes([sector0[17], sector0[18]]);
    let sectors_per_fat: u16 = u16::from_le_bytes([sector0[22], sector0[23]]);

    // LBA корневого каталога = sectors_per_fat * fat_count + reserved_sectors
    let root_dir_lba: u32 =
        sectors_per_fat as u32 * fat_count as u32 + reserved_sectors as u32;

    // Размер корневого каталога в секторах, с округлением вверх
    let root_dir_bytes: u32 = dir_entries as u32 * 32;
    let root_dir_size: u32 =
        (root_dir_bytes + bytes_per_sector as u32 - 1) / bytes_per_sector as u32;

    let data_lba: u32 = root_dir_lba + root_dir_size;

    Some(Bpb {
        bytes_per_sector,
        sectors_per_cluster,
        reserved_sectors,
        fat_count,
        dir_entries,
        sectors_per_fat,
        root_dir_lba,
        root_dir_size,
        data_lba,
    })
}

/// Перебор всех непустых записей корневого каталога
/// Параметры:
///  - visit: вызывается для каждой записи; false останавливает перебор раньше
/// Вывод:
///  - false - ошибка чтения диска
fn for_each_root_entry<F: FnMut(&DirEntry) -> bool>(bpb: &Bpb, mut visit: F) -> bool {
    let mut sector_buf: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];

    for i in 0..bpb.root_dir_size {
        if !floppy::read_sector(bpb.root_dir_lba + i, &mut sector_buf) {
            return false;
        }

        for entry_off in (0..SECTOR_SIZE).step_by(32) {
            let raw: &[u8] = &sector_buf[entry_off..entry_off + 32];
            let first_byte: u8 = raw[0];

            // 0x00 - конец каталога (Дальше записей нет)
            if first_byte == 0x00 {
                return true;
            }
            // 0xE5 - удалённый файл, 0x0F - Long File Name, бит 0x08 - метка тома
            if first_byte == 0xE5 || raw[11] == 0x0F || raw[11] & 0x08 != 0 {
                continue;
            }

            let mut name: [u8; 11] = [0; 11];
            name.copy_from_slice(&raw[0..11]);

            let entry = DirEntry {
                name,
                first_cluster: u16::from_le_bytes([raw[26], raw[27]]),
                size: u32::from_le_bytes([raw[28], raw[29], raw[30], raw[31]]),
            };

            if !visit(&entry) {
                return true;
            }
        }
    }
    true
}

/// Список всех файлов корневого каталога
/// Параметры:
///  - visit: вызывается для каждой найденной записи
/// Вывод:
///  - false - ошибка чтения диска
pub fn list_files<F: FnMut(&DirEntry)>(bpb: &Bpb, mut visit: F) -> bool {
    for_each_root_entry(bpb, |entry| {
        visit(entry);
        true
    })
}

/// Поиск файла по имени в формате 8.3 (11 байт, см. `format_83`)
pub fn find_file(bpb: &Bpb, name83: &[u8; 11]) -> Option<DirEntry> {
    let mut found: Option<DirEntry> = None;

    for_each_root_entry(bpb, |entry| {
        if &entry.name == name83 {
            found = Some(*entry);
            false
        } else {
            true
        }
    });

    found
}

/// Конвертация имени файла из пользовательского ввода в формат FAT 8.3
/// Параметры:
///  - input: имя вида "kernel16.bin" (Регистр не важен)
/// Вывод:
///  - None - некорректное имя (Пустое, длиннее 8.3 или несколько точек)
pub fn format_83(input: &str) -> Option<[u8; 11]> {
    let input: &str = input.trim();
    if input.is_empty() {
        return None;
    }

    let (name_part, ext_part) = match input.find('.') {
        Some(idx) => (&input[..idx], &input[idx + 1..]),
        None => (input, ""),
    };

    if name_part.is_empty() || name_part.len() > 8 || ext_part.len() > 3 {
        return None;
    }
    if ext_part.contains('.') || !input.is_ascii() {
        return None;
    }

    let mut result: [u8; 11] = [b' '; 11];
    for (i, b) in name_part.bytes().enumerate() {
        result[i] = b.to_ascii_uppercase();
    }
    for (i, b) in ext_part.bytes().enumerate() {
        result[8 + i] = b.to_ascii_uppercase();
    }

    Some(result)
}

/// Загрузка таблицы FAT в кэш (Ограничено `MAX_FAT_SECTORS`)
fn load_fat(bpb: &Bpb) -> bool {
    let count: usize = (bpb.sectors_per_fat as usize).min(MAX_FAT_SECTORS);
    let mut sector_buf: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];

    for i in 0..count {
        if !floppy::read_sector(bpb.reserved_sectors as u32 + i as u32, &mut sector_buf) {
            return false;
        }
        unsafe {
            FAT_CACHE[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE].copy_from_slice(&sector_buf);
        }
    }
    true
}

/// Чтение 12-битной записи таблицы FAT по номеру кластера (Из уже загруженного кэша)
fn fat_entry(cluster: u16) -> u16 {
    let offset: usize = (cluster as usize * 3) / 2;

    unsafe {
        let raw: u16 = u16::from_le_bytes([FAT_CACHE[offset], FAT_CACHE[offset + 1]]);
        if cluster & 1 == 0 {
            raw & 0x0FFF // Чётный кластер - младшие 12 бит
        } else {
            raw >> 4 // Нечётный кластер - старшие 12 бит
        }
    }
}

/// Чтение файла по цепочке кластеров FAT12 в буфер назначения
/// Параметры:
///  - entry: запись каталога (Из `find_file`)
///  - dest: буфер назначения (Данные, не влезающие в него, отбрасываются)
/// Вывод:
///  - None - ошибка чтения диска или повреждённая/зацикленная цепочка
///  - Some(n) - успех, n байт записано в `dest`
pub fn read_file(bpb: &Bpb, entry: &DirEntry, dest: &mut [u8]) -> Option<usize> {
    // Пустой файл: цепочки кластеров нет (0 - пусто, 1 - резерв)
    if entry.first_cluster < 2 {
        return Some(0);
    }
    if !load_fat(bpb) {
        return None;
    }

    let mut cluster: u16 = entry.first_cluster;
    let mut written: usize = 0;
    let mut chain_length: u32 = 0;
    let mut sector_buf: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];

    loop {
        let cluster_lba: u32 =
            bpb.data_lba + (cluster as u32 - 2) * bpb.sectors_per_cluster as u32;

        for s in 0..bpb.sectors_per_cluster as u32 {
            if !floppy::read_sector(cluster_lba + s, &mut sector_buf) {
                return None;
            }

            let remaining: usize =
                (entry.size as usize).saturating_sub(written).min(dest.len() - written);
            let take: usize = remaining.min(SECTOR_SIZE);

            if take > 0 {
                dest[written..written + take].copy_from_slice(&sector_buf[..take]);
                written += take;
            }
        }

        let next: u16 = fat_entry(cluster);

        // Конец цепочки
        if next >= CHAIN_END {
            break;
        }
        // Дефектный кластер или служебный (0/1) - цепочка повреждена
        if next == BAD_CLUSTER || next < 2 {
            return None;
        }

        chain_length += 1;
        if chain_length > MAX_CHAIN_LENGTH {
            return None; // Защита от зацикленной FAT-цепочки
        }

        cluster = next;
    }

    Some(written)
}
