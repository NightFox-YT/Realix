/*
	Realix fat32 driver, 
	The realix drivers pack.
*/
#ifndef __realix_fat32__
#define __realix_fat32__

#include "../../libc/stdint.h"
#include "../../include/io.h"
#include "../../vfs/vfs.h"
#include "../ata/ata.h"

struct gcc_packed mbr_partition {
	uint8_t 	boot_ind;
	uint8_t 	starting_chs[3];
	uint8_t 	os_type;
	uint8_t 	ending_chs[3];
	uint32_t 	starting_lba;
	uint32_t 	sizein_sectors;
};

struct gcc_packed mbr_layout {
	uint8_t		         boot_code[446];
	struct mbr_partition partitions[4];
	uint16_t 			 sign;
};

struct gcc_packed fat32_bpb {
	uint8_t  jmp_boot[3];
    char     oem_name[8];
    uint16_t bytes_per_sector;   
    uint8_t  sectors_per_cluster; // Количество секторов в одном кластере
    uint16_t res_sector_count; // Количество резервных секторов (до таблицы FAT)
    uint8_t  num_fats;            // Количество таблиц FAT (обычно 2)
    uint16_t root_entry_count;    // Для FAT32 всегда 0
    uint16_t total_sectors_16;    // Для FAT32 всегда 0
    uint8_t  media_type;
    uint16_t fat_size_16;         // Для FAT32 всегда 0
    uint16_t sectors_per_track;
    uint16_t num_heads;
    uint32_t hidden_sectors;      // Равно starting_lba из MBR
    uint32_t total_sectors_32;    // Полный размер тома в секторах
    uint32_t fat_size_32;         // Размер одной таблицы FAT в секторах
    uint16_t ext_flags;
    uint16_t fs_version;
    uint32_t root_cluster_num;    // Номер первого кластера корневого каталога (обычно 2)
    uint16_t fs_info_sector;      // Сектор со свободной памятью (обычно 1)
    uint16_t backup_boot_sector;
    uint8_t  reserved[12];
    uint8_t  drive_number;
    uint8_t  reserved1;
    uint8_t  boot_sign;      // Должна быть 0x29
    uint32_t volume_id;
    char     volume_label[11];
    char     fs_type[8];          // Строка "FAT32   "
};

struct gcc_packed fat32_dir_entry {
	char     name[11];            // 8 символов имя + 3 расширение (8.3 формат)
    uint8_t  attr;                // Атрибуты (0x10 - папка, 0x20 - архивный и т.д.)
    uint8_t  nt_res;
    uint8_t  crt_time_tenth;
    uint16_t crt_time;
    uint16_t crt_date;
    uint16_t lst_acc_date;
    uint16_t cluster_num_high;    // Старшие 16 бит номера первого кластера
    uint16_t wrt_time;
    uint16_t wrt_date;
    uint16_t cluster_num_low;     // Младшие 16 бит номера первого кластера
    uint32_t file_size;       
};

struct fat32_volume {
	struct ata_channel *ch;
    uint8_t  drive;
    uint32_t start_lba;          
    uint32_t sectors_per_cluster;
    uint32_t reserved_sectors;
    uint32_t fat_size;           
    uint32_t num_fats;
    uint32_t root_cluster;
    uint32_t fat_start_lba;      
    uint32_t clusters_start_lba;  
};

struct fat32_fs_info {
    struct ata_channel *chan;    // Ссылка на канал ATA
    uint8_t  drive;              // Master или Slave
    uint32_t partition_start_lba;// Стартовый сектор раздела из MBR
    
    uint32_t bytes_per_sector;   // Обычно 512
    uint32_t sectors_per_cluster;// Сколько секторов в кластере (1, 2, 4, 8...)
    uint32_t reserved_sectors;   // Сектора до первой таблицы FAT
    uint32_t num_fats;   
    uint32_t fat_size_sectors;   // Размер одной таблицы FAT в секторах
    uint32_t root_cluster;       // Первый кластер корневой директории
};

struct fat32_file_info {
    uint32_t first_cluster;      // С какого кластера начинается файл
    uint32_t current_cluster;    // На каком кластере мы сейчас находимся
    uint32_t current_cluster_idx;// Индекс текущего кластера от начала
};

#define MBR_SIGN 0xAA55

int fat32_init_volume(struct fat32_volume *vol, struct ata_channel *ch, uint8_t dev_idx);
int fat32_find_file(struct fat32_volume *vol, const char *filename, struct fat32_dir_entry *out_entry);
int fat32_read(struct fat32_volume *vol, struct fat32_dir_entry *entry, uint8_t *buf);
int fat32_write(struct fat32_volume *vol, struct fat32_dir_entry *entry, const uint8_t *buf, uint32_t size);
int fat32_close(struct vfs_file *file);
int fat32_open(struct vfs_file *file, const char *path);
int fat32_readdir(struct vfs_file *file, struct vfs_dirent *dir, uint32_t index);
int fat32_create(struct vfs_node *node, const char *path);
int fat32_vfs_read(struct vfs_file *file, uint8_t *buf, uint32_t size);
int fat32_vfs_write(struct vfs_file *file, const uint8_t *buf, uint32_t size);

extern struct vfs_node_ops fat32_node_ops;

#endif /*__realix_fat32__*/