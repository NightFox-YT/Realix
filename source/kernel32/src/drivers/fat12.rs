// © Realix > Driver: FAT12
// (15.08.26) v0.1
// ================
// ❗️ Зависимости: x86::realmode (чтение/запись секторов через BIOS-thunk)
// ❗️ Нет динамического аллокатора - все буферы статические фиксированного
//    размера, рассчитанные на стандартную дискету 1.44 МБ (mformat -f 1440):
//    224 корневых записи (14 секторов), FAT12 обычно ~9 секторов на таблицу.
// ❗️ init() должен быть вызван один раз перед любой другой функцией модуля.

use crate::x86::realmode;

const SECTOR_SIZE: usize = 512;
const BOOT_SECTOR_LBA: u32 = 0;
const DIRENT_SIZE: usize = 32;

// Смещения полей 32-байтной записи каталога FAT12
const DIRENT_NAME: usize = 0; // 8 байт имени + 3 байта расширения (offset 8)
const DIRENT_ATTR: usize = 11;
const DIRENT_FIRST_CLUSTER: usize = 26;
const DIRENT_FILESIZE: usize = 28;

const DIRENT_FREE: u8 = 0x00; // Конец каталога
const DIRENT_DELETED: u8 = 0xE5;
const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_LFN: u8 = 0x0F;

// Маркеры кластеров FAT12 (совпадают с source/shared/config.asm)
const CHAIN_END: u16 = 0x0FF8;
const BAD_CLUSTER: u16 = 0x0FF7;

// Буферы (запас с округлением вверх под стандартную дискету 1.44 МБ)
const ROOT_DIR_BUF_SECTORS: usize = 16; // >= 224*32/512 = 14 секторов
const FAT_BUF_SECTORS: usize = 16; // >= обычных 9 секторов на FAT12-таблицу

// Максимальный поддерживаемый размер кластера (в секторах) - используется как
// промежуточный буфер при чтении/записи файлов, чтобы никогда не писать за
// пределы предоставленного вызывающим кода среза (последний сектор кластера
// почти всегда неполный по отношению к оставшемуся размеру файла)
const CLUSTER_SCRATCH_SECTORS: usize = 8;

static mut BOOT_SECTOR: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
static mut ROOT_DIR_BUF: [u8; ROOT_DIR_BUF_SECTORS * SECTOR_SIZE] =
    [0; ROOT_DIR_BUF_SECTORS * SECTOR_SIZE];
static mut FAT_BUF: [u8; FAT_BUF_SECTORS * SECTOR_SIZE] = [0; FAT_BUF_SECTORS * SECTOR_SIZE];
static mut CLUSTER_SCRATCH: [u8; CLUSTER_SCRATCH_SECTORS * SECTOR_SIZE] =
    [0; CLUSTER_SCRATCH_SECTORS * SECTOR_SIZE];

#[derive(Clone, Copy)]
struct Fat12Info {
    sectors_per_cluster: u8,
    fat_count: u8,
    dir_entries: u16,
    sectors_per_fat: u16,
    fat1_lba: u32,
    root_dir_lba: u32,
    root_dir_sectors: u32,
    data_lba: u32,
    total_clusters: u16,
}

static mut FS_INFO: Option<Fat12Info> = None;

/// Разбор имени файла из пользовательского ввода в формат 8.3 (11 байт)
/// Параметры:
///  - input: имя (например "notes.txt" или "NOTES.TXT")
/// Вывод:
///  - Some(11-байтный 8.3, заглавные, дополнено пробелами), либо None если
///    имя некорректно (пустое базовое имя, слишком длинное имя/расширение,
///    более одной точки)
pub fn parse_name_83(input: &str) -> Option<[u8; 11]> {
    let (base, ext) = match input.split_once('.') {
        Some((b, e)) => (b, e),
        None => (input, ""),
    };

    if base.is_empty() || base.len() > 8 || ext.len() > 3 || ext.contains('.') {
        return None;
    }

    let mut result = [b' '; 11];
    for (i, byte) in base.bytes().enumerate() {
        result[i] = byte.to_ascii_uppercase();
    }
    for (i, byte) in ext.bytes().enumerate() {
        result[8 + i] = byte.to_ascii_uppercase();
    }

    Some(result)
}

/// Одна запись каталога, разобранная в удобный вид
pub struct DirEntry {
    pub name: [u8; 11], // 8.3 формат (заглавные, дополнено пробелами)
    pub first_cluster: u16,
    pub size: u32,
}

/// Инициализация по требованию: если уже инициализировано, ничего не делает
/// (безопасно вызывать перед каждой командой файловой системы). Так диск
/// трогается только при первом реальном обращении к файлам, а не при каждой
/// загрузке ОС - неисправность здесь ломает только файловые команды, а не
/// весь boot
fn ensure_init() -> bool {
    if unsafe { matches!(FS_INFO, Some(_)) } {
        return true;
    }
    init()
}

/// Инициализация: читает загрузочный сектор, разбирает BPB, читает FAT
/// и корневой каталог в память. Идемпотентна - безопасно вызывать повторно
/// (например, чтобы обновить состояние диска), но обычно достаточно
/// ensure_init(), вызываемой автоматически другими функциями модуля
/// Вывод: true при успехе
pub fn init() -> bool {
    unsafe {
        if !realmode::read_sectors(BOOT_SECTOR_LBA, 1, boot_sector_addr()) {
            return false;
        }

        let bytes_per_sector = u16::from_le_bytes([BOOT_SECTOR[11], BOOT_SECTOR[12]]);
        let sectors_per_cluster = BOOT_SECTOR[13];
        let reserved_sectors = u16::from_le_bytes([BOOT_SECTOR[14], BOOT_SECTOR[15]]);
        let fat_count = BOOT_SECTOR[16];
        let dir_entries = u16::from_le_bytes([BOOT_SECTOR[17], BOOT_SECTOR[18]]);
        let total_sectors_16 = u16::from_le_bytes([BOOT_SECTOR[19], BOOT_SECTOR[20]]);
        let sectors_per_fat = u16::from_le_bytes([BOOT_SECTOR[22], BOOT_SECTOR[23]]);
        let total_sectors_32 = u32::from_le_bytes([
            BOOT_SECTOR[32], BOOT_SECTOR[33], BOOT_SECTOR[34], BOOT_SECTOR[35],
        ]);

        if bytes_per_sector as usize != SECTOR_SIZE || sectors_per_cluster == 0 {
            return false; // Не поддерживается (весь код рассчитан на 512 байт/сектор)
        }

        // FAT1 - первая копия таблицы, сразу после зарезервированных секторов
        // (не root_dir_lba - sectors_per_fat: при fat_count=2 это была бы FAT2)
        let fat1_lba = reserved_sectors as u32;
        let root_dir_lba = fat1_lba + (sectors_per_fat as u32 * fat_count as u32);
        let root_dir_bytes = dir_entries as u32 * DIRENT_SIZE as u32;
        let root_dir_sectors = root_dir_bytes.div_ceil(bytes_per_sector as u32);
        let data_lba = root_dir_lba + root_dir_sectors;

        let total_sectors = if total_sectors_16 != 0 { total_sectors_16 as u32 } else { total_sectors_32 };
        let total_clusters = if total_sectors > data_lba {
            ((total_sectors - data_lba) / sectors_per_cluster as u32).min(u16::MAX as u32) as u16
        } else {
            0
        };

        if root_dir_sectors as usize > ROOT_DIR_BUF_SECTORS
            || sectors_per_fat as usize > FAT_BUF_SECTORS
        {
            return false; // Слишком большой том для статических буферов
        }

        let info = Fat12Info {
            sectors_per_cluster,
            fat_count,
            dir_entries,
            sectors_per_fat,
            fat1_lba,
            root_dir_lba,
            root_dir_sectors,
            data_lba,
            total_clusters,
        };
        FS_INFO = Some(info);

        reload_fat(&info) && reload_root_dir(&info)
    }
}

fn reload_fat(info: &Fat12Info) -> bool {
    realmode::read_sectors(info.fat1_lba, info.sectors_per_fat as u8, fat_buf_addr())
}

fn reload_root_dir(info: &Fat12Info) -> bool {
    realmode::read_sectors(info.root_dir_lba, info.root_dir_sectors as u8, root_dir_buf_addr())
}

/// Записывает FAT_BUF на диск (во все копии FAT, если их несколько)
fn flush_fat(info: &Fat12Info) -> bool {
    for copy in 0..info.fat_count as u32 {
        let lba = info.fat1_lba + copy * info.sectors_per_fat as u32;
        if !realmode::write_sectors(lba, info.sectors_per_fat as u8, fat_buf_addr()) {
            return false;
        }
    }
    true
}

/// Записывает ROOT_DIR_BUF на диск
fn flush_root_dir(info: &Fat12Info) -> bool {
    realmode::write_sectors(info.root_dir_lba, info.root_dir_sectors as u8, root_dir_buf_addr())
}

fn boot_sector_addr() -> u32 {
    &raw const BOOT_SECTOR as u32
}

fn root_dir_buf_addr() -> u32 {
    &raw const ROOT_DIR_BUF as u32
}

fn fat_buf_addr() -> u32 {
    &raw const FAT_BUF as u32
}

/// Возвращает запись каталога по индексу (0-based), либо None если
/// достигнут конец каталога (свободная запись) или индекс за пределами
pub fn dir_entry(index: usize) -> Option<DirEntry> {
    ensure_init();
    unsafe {
        let info = FS_INFO?;
        if index >= info.dir_entries as usize {
            return None;
        }

        let offset = index * DIRENT_SIZE;
        let first_byte = ROOT_DIR_BUF[offset + DIRENT_NAME];

        if first_byte == DIRENT_FREE {
            return None;
        }
        if first_byte == DIRENT_DELETED {
            return Some(DirEntry { name: [0; 11], first_cluster: 0, size: 0 }); // Пропускается вызывающим
        }

        let attr = ROOT_DIR_BUF[offset + DIRENT_ATTR];
        if attr == ATTR_LFN || attr & ATTR_VOLUME_ID != 0 || attr & ATTR_DIRECTORY != 0 {
            return Some(DirEntry { name: [0; 11], first_cluster: 0, size: 0 }); // Пропускается вызывающим
        }

        let mut name = [0u8; 11];
        name.copy_from_slice(&ROOT_DIR_BUF[offset..offset + 11]);

        let first_cluster = u16::from_le_bytes([
            ROOT_DIR_BUF[offset + DIRENT_FIRST_CLUSTER],
            ROOT_DIR_BUF[offset + DIRENT_FIRST_CLUSTER + 1],
        ]);
        let size = u32::from_le_bytes([
            ROOT_DIR_BUF[offset + DIRENT_FILESIZE],
            ROOT_DIR_BUF[offset + DIRENT_FILESIZE + 1],
            ROOT_DIR_BUF[offset + DIRENT_FILESIZE + 2],
            ROOT_DIR_BUF[offset + DIRENT_FILESIZE + 3],
        ]);

        Some(DirEntry { name, first_cluster, size })
    }
}

/// Проверка, является ли запись реальным файлом (не пропуском/заглушкой)
pub fn is_real_entry(entry: &DirEntry) -> bool {
    entry.name != [0u8; 11]
}

/// Максимальное число записей в корневом каталоге (для перебора в командах)
pub fn dir_entries_capacity() -> u16 {
    ensure_init();
    unsafe { FS_INFO.map(|i| i.dir_entries).unwrap_or(0) }
}

/// Значение записи FAT12 для кластера (12-битная упаковка)
/// Параметры:
///  - cluster: номер кластера
/// Вывод:
///  - значение записи (следующий кластер в цепочке, либо CHAIN_END/BAD_CLUSTER)
fn fat_entry(cluster: u16) -> u16 {
    unsafe {
        let byte_offset = (cluster as usize * 3) / 2;
        let word = u16::from_le_bytes([FAT_BUF[byte_offset], FAT_BUF[byte_offset + 1]]);

        if cluster & 1 == 0 {
            word & 0x0FFF
        } else {
            word >> 4
        }
    }
}

/// Устанавливает значение записи FAT12 для кластера (12-битная упаковка).
/// Меняет только нужные 12 бит из общего 16-битного слова, сохраняя нибл
/// соседнего кластера, упакованный в тот же байт (см. fat_entry)
fn fat_set_entry(cluster: u16, value: u16) {
    unsafe {
        let byte_offset = (cluster as usize * 3) / 2;
        let word = u16::from_le_bytes([FAT_BUF[byte_offset], FAT_BUF[byte_offset + 1]]);

        let new_word = if cluster & 1 == 0 {
            (word & 0xF000) | (value & 0x0FFF)
        } else {
            (word & 0x000F) | (value << 4)
        };

        let bytes = new_word.to_le_bytes();
        FAT_BUF[byte_offset] = bytes[0];
        FAT_BUF[byte_offset + 1] = bytes[1];
    }
}

/// Ищет и резервирует (CHAIN_END) первый свободный кластер (значение 0)
fn alloc_cluster(info: &Fat12Info) -> Option<u16> {
    for cluster in 2..(2u32 + info.total_clusters as u32) {
        let cluster = cluster as u16;
        if fat_entry(cluster) == 0 {
            fat_set_entry(cluster, CHAIN_END);
            return Some(cluster);
        }
    }
    None
}

/// Освобождает всю цепочку кластеров, начиная с first_cluster (значение 0
/// в каждом звене), без записи на диск - вызывающий должен сделать flush_fat
fn free_chain(mut cluster: u16) {
    while cluster >= 2 && cluster < CHAIN_END && cluster != BAD_CLUSTER {
        let next = fat_entry(cluster);
        fat_set_entry(cluster, 0);
        cluster = next;
    }
}

/// Ищет индекс записи каталога по имени (8.3). Возвращает None, если не найдена
fn find_entry_index(info: &Fat12Info, name_83: &[u8; 11]) -> Option<usize> {
    for i in 0..info.dir_entries as usize {
        let offset = i * DIRENT_SIZE;
        let first_byte = unsafe { ROOT_DIR_BUF[offset + DIRENT_NAME] };

        if first_byte == DIRENT_FREE {
            return None; // Конец каталога
        }
        if let Some(entry) = dir_entry(i) {
            if is_real_entry(&entry) && entry.name == *name_83 {
                return Some(i);
            }
        }
    }
    None
}

/// Ищет индекс свободной записи каталога (для создания нового файла):
/// первая запись с байтом DIRENT_FREE или DIRENT_DELETED
fn find_free_slot(info: &Fat12Info) -> Option<usize> {
    for i in 0..info.dir_entries as usize {
        let offset = i * DIRENT_SIZE;
        let first_byte = unsafe { ROOT_DIR_BUF[offset + DIRENT_NAME] };
        if first_byte == DIRENT_FREE || first_byte == DIRENT_DELETED {
            return Some(i);
        }
    }
    None
}

/// Записывает поля одной записи каталога по индексу (кроме имени, если None)
fn write_dir_entry(index: usize, name_83: Option<&[u8; 11]>, first_cluster: u16, size: u32) {
    unsafe {
        let offset = index * DIRENT_SIZE;

        if let Some(name) = name_83 {
            ROOT_DIR_BUF[offset..offset + 11].copy_from_slice(name);
            ROOT_DIR_BUF[offset + DIRENT_ATTR] = 0; // Обычный файл (атрибуты не используются)
        }

        let cluster_bytes = first_cluster.to_le_bytes();
        ROOT_DIR_BUF[offset + DIRENT_FIRST_CLUSTER] = cluster_bytes[0];
        ROOT_DIR_BUF[offset + DIRENT_FIRST_CLUSTER + 1] = cluster_bytes[1];

        let size_bytes = size.to_le_bytes();
        ROOT_DIR_BUF[offset + DIRENT_FILESIZE..offset + DIRENT_FILESIZE + 4]
            .copy_from_slice(&size_bytes);
    }
}

/// Записывает текстовое содержимое в файл, заменяя предыдущее (если было);
/// создаёт файл, если он ещё не существует
/// Параметры:
///  - name_83: имя файла (8.3)
///  - data: новое содержимое
/// Вывод:
///  - true при успехе
pub fn write_file(name_83: &[u8; 11], data: &[u8]) -> bool {
    ensure_init();
    let info = match unsafe { FS_INFO } {
        Some(i) => i,
        None => return false,
    };

    // Существующий файл перезаписывается целиком; отсутствующий - создаётся
    let (index, old_cluster) = match find_entry_index(&info, name_83) {
        Some(i) => {
            let cluster = unsafe {
                u16::from_le_bytes([
                    ROOT_DIR_BUF[i * DIRENT_SIZE + DIRENT_FIRST_CLUSTER],
                    ROOT_DIR_BUF[i * DIRENT_SIZE + DIRENT_FIRST_CLUSTER + 1],
                ])
            };
            (i, cluster)
        }
        None => {
            let slot = match find_free_slot(&info) {
                Some(s) => s,
                None => return false, // Каталог полон
            };
            write_dir_entry(slot, Some(name_83), 0, 0);
            (slot, 0)
        }
    };

    if old_cluster >= 2 {
        free_chain(old_cluster);
    }

    if info.sectors_per_cluster as usize > CLUSTER_SCRATCH_SECTORS {
        return false;
    }

    let bytes_per_cluster = info.sectors_per_cluster as usize * SECTOR_SIZE;
    let mut first_cluster: u16 = 0;
    let mut prev_cluster: u16 = 0;
    let mut written: usize = 0;

    while written < data.len() {
        let cluster = match alloc_cluster(&info) {
            Some(c) => c,
            None => {
                // Диск заполнен - освобождаем уже выделенное для этой записи
                if first_cluster >= 2 {
                    free_chain(first_cluster);
                }
                let _ = flush_fat(&info);
                return false;
            }
        };

        if first_cluster == 0 {
            first_cluster = cluster;
        } else {
            fat_set_entry(prev_cluster, cluster);
        }
        prev_cluster = cluster;

        // chunk_len <= bytes_per_cluster всегда (см. .min() выше), поэтому
        // срез chunk_len..bytes_per_cluster корректен (пуст, если кластер заполнен целиком)
        let chunk_len = (data.len() - written).min(bytes_per_cluster);
        unsafe {
            CLUSTER_SCRATCH[..chunk_len].copy_from_slice(&data[written..written + chunk_len]);
            CLUSTER_SCRATCH[chunk_len..bytes_per_cluster].fill(0);
        }

        let lba = info.data_lba + (cluster as u32 - 2) * info.sectors_per_cluster as u32;
        let sectors = bytes_per_cluster.div_ceil(SECTOR_SIZE) as u8;
        if !realmode::write_sectors(lba, sectors, &raw const CLUSTER_SCRATCH as u32) {
            return false;
        }

        written += chunk_len;
    }

    write_dir_entry(index, None, first_cluster, data.len() as u32);
    flush_fat(&info) && flush_root_dir(&info)
}

/// Переименовывает файл
/// Вывод: true при успехе (false: не найден, либо новое имя уже занято)
pub fn rename_file(old_name_83: &[u8; 11], new_name_83: &[u8; 11]) -> bool {
    ensure_init();
    let info = match unsafe { FS_INFO } {
        Some(i) => i,
        None => return false,
    };

    let index = match find_entry_index(&info, old_name_83) {
        Some(i) => i,
        None => return false,
    };

    if find_entry_index(&info, new_name_83).is_some() {
        return false; // Новое имя уже занято
    }

    unsafe {
        ROOT_DIR_BUF[index * DIRENT_SIZE..index * DIRENT_SIZE + 11].copy_from_slice(new_name_83);
    }
    flush_root_dir(&info)
}

/// Удаляет файл (освобождает кластеры и помечает запись каталога удалённой)
/// Вывод: true при успехе (false: не найден)
pub fn delete_file(name_83: &[u8; 11]) -> bool {
    ensure_init();
    let info = match unsafe { FS_INFO } {
        Some(i) => i,
        None => return false,
    };

    let index = match find_entry_index(&info, name_83) {
        Some(i) => i,
        None => return false,
    };

    let cluster = unsafe {
        u16::from_le_bytes([
            ROOT_DIR_BUF[index * DIRENT_SIZE + DIRENT_FIRST_CLUSTER],
            ROOT_DIR_BUF[index * DIRENT_SIZE + DIRENT_FIRST_CLUSTER + 1],
        ])
    };
    if cluster >= 2 {
        free_chain(cluster);
    }

    unsafe {
        ROOT_DIR_BUF[index * DIRENT_SIZE + DIRENT_NAME] = DIRENT_DELETED;
    }

    flush_fat(&info) && flush_root_dir(&info)
}

/// Читает содержимое файла по имени (8.3, как хранится в каталоге) в буфер
/// Параметры:
///  - name_83: имя в формате 8.3 (11 байт, заглавные, дополнено пробелами)
///  - dest: буфер назначения
/// Вывод:
///  - Some(размер файла в байтах), если найден и уместился в dest; None иначе
pub fn read_file(name_83: &[u8; 11], dest: &mut [u8]) -> Option<u32> {
    ensure_init();
    let info = unsafe { FS_INFO? };
    let mut cluster: u16 = 0;
    let mut size: u32 = 0;
    let mut found = false;

    for i in 0..info.dir_entries {
        if let Some(entry) = dir_entry(i as usize) {
            if is_real_entry(&entry) && entry.name == *name_83 {
                cluster = entry.first_cluster;
                size = entry.size;
                found = true;
                break;
            }
        } else if unsafe { ROOT_DIR_BUF[i as usize * DIRENT_SIZE] } == DIRENT_FREE {
            break;
        }
    }

    if !found {
        return None;
    }
    if size as usize > dest.len() {
        return None;
    }
    if cluster < 2 {
        return Some(0); // Пустой файл
    }

    if info.sectors_per_cluster as usize > CLUSTER_SCRATCH_SECTORS {
        return None; // Кластер больше, чем поддерживает промежуточный буфер
    }

    let bytes_per_cluster = info.sectors_per_cluster as u32 * SECTOR_SIZE as u32;
    let mut written: u32 = 0;

    loop {
        if cluster >= CHAIN_END || cluster == BAD_CLUSTER || cluster < 2 {
            break;
        }

        let lba = info.data_lba + (cluster as u32 - 2) * info.sectors_per_cluster as u32;
        let remaining = size - written;
        let to_read = remaining.min(bytes_per_cluster);
        let sectors = to_read.div_ceil(SECTOR_SIZE as u32).max(1);

        // Читаем целыми секторами в промежуточный буфер (безопасно даже если
        // последний сектор кластера не заполнен целиком), затем копируем
        // только реально нужные байты в dest - иначе округление вверх до
        // сектора могло бы записать за пределы dest
        unsafe {
            if !realmode::read_sectors(lba, sectors as u8, &raw mut CLUSTER_SCRATCH as u32) {
                return None;
            }
            dest[written as usize..(written + to_read) as usize]
                .copy_from_slice(&CLUSTER_SCRATCH[..to_read as usize]);
        }

        written += to_read;
        if written >= size {
            break;
        }

        cluster = fat_entry(cluster);
    }

    Some(size)
}
