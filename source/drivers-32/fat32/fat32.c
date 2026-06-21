#include "fat32.h"
#include "../../libc/string/string.h"

static uint32_t cluster_to_lba(struct fat32_volume *vol, uint32_t cluster)
{
    return vol->clusters_start_lba + ((cluster - 2) * vol->sectors_per_cluster);
}

static uint32_t fat32_get_next_cluster(struct fat32_volume *vol, uint32_t current_cluster)
{
    uint16_t sector_buf[256];

    uint32_t fat_offset = current_cluster * 4;
    uint32_t fat_sector = vol->fat_start_lba + (fat_offset / 512);
    uint32_t entry_offset = (fat_offset % 512) / 4;

    if (ata_read_sector(vol->ch, vol->drive, fat_sector, sector_buf) != 0) return 0x0FFFFFF7;
    uint32_t next_cluster = ((uint32_t*)sector_buf)[entry_offset] & 0x0FFFFFFF;
    return next_cluster;
}

static int fat32_set_cluster(struct fat32_volume *vol, uint32_t cluster, uint32_t value)
{
    uint16_t sector_buf[256];
    uint32_t fat_offset = cluster * 4;
    uint32_t fat_sector = vol->fat_start_lba + (fat_offset / 512);
    uint32_t entry_offset = (fat_offset % 512) / 4;

    if (ata_read_sector(vol->ch, vol->drive, fat_sector, sector_buf) != 0) return -1;

    uint32_t old_val = sector_buf[entry_offset];
    sector_buf[entry_offset] = (old_val & 0xF0000000) | (value & 0x0FFFFFFF);

    if (ata_write_sector(vol->ch, vol->drive, fat_sector, sector_buf) != 0) return -1;
    
    return 0;
}

static uint32_t fat32_find_free_cluster(struct fat32_volume *vol) {
    uint16_t sector_buffer[256];
    for (uint32_t s = 0; s < 256; s++) 
    { 
        uint32_t fat_sector = vol->fat_start_lba + s;
        if (ata_read_sector(vol->ch, vol->drive, fat_sector, sector_buffer) != 0) return 0;

        uint32_t *entries = (uint32_t*)sector_buffer;
        for (int i = 0; i < 128; i++) 
        {
            uint32_t cluster_num = (s * 128) + i;
            if (cluster_num < 2) continue;
            
            if ((entries[i] & 0x0FFFFFFF) == 0) return cluster_num;
        }
    }
    return 0;
}

static int fat32_update_dir_entry(struct fat32_volume *vol, struct fat32_dir_entry *entry) 
{
    uint16_t sector_buffer[256];
    uint32_t current_cluster = vol->root_cluster;

    while (current_cluster < 0x0FFFFFF8) 
    {
        for (uint32_t s = 0; s < vol->sectors_per_cluster; s++) 
        {
            uint32_t sector_lba = vol->clusters_start_lba + ((current_cluster - 2) * vol->sectors_per_cluster) + s;
            if (ata_read_sector(vol->ch, vol->drive, sector_lba, sector_buffer) != 0) return -1;

            struct fat32_dir_entry *entries = (struct fat32_dir_entry*)sector_buffer;
            for (int i = 0; i < 16; i++) {
                if (entries[i].name == 0x00) return -2;
                
                if (memcmp(entries[i].name, entry->name, 11) == 0) 
                {
                    entries[i].file_size = entry->file_size;
                    entries[i].cluster_num_high = entry->cluster_num_high;
                    entries[i].cluster_num_low = entry->cluster_num_low;
                    
                    return ata_write_sector(vol->ch, vol->drive, sector_lba, sector_buffer);
                }
            }
        }
        current_cluster = fat32_get_next_cluster(vol, current_cluster);
    }
    return -2;
}

static void fat32_format_name(const char *fat_name, char *dest) 
{
    int p = 0;
    for (int i = 0; i < 8; i++) {
        if (fat_name[i] != ' ') {
            dest[p++] = fat_name[i];
        }
    }
    /*если есть расширение, добавляем точку*/
    if (fat_name[8] != ' ') {
        dest[p++] = '.';
        for (int i = 8; i < 11; i++) {
            if (fat_name[i] != ' ') {
                dest[p++] = fat_name[i];
            }
        }
    }
    dest[p] = '\0';
}

/*перевод обычного пути ("test.txt") в 8.3 ("TEST    TXT")*/
static void to_fat_name(const char *src, char *dest) {
    for (int i = 0; i < 11; i++) dest[i] = ' ';
    int i = 0;
    while (src[i] != '\0' && src[i] != '.' && i < 8) 
    {
        char c = src[i];
        if (c >= 'a' && c <= 'z') c -= 32;
        dest[i] = c;
        i++;
    }
    while (src[i] != '\0' && src[i] != '.') i++;
    if (src[i] == '.') 
    {
        i++;
        int j = 0;
        while (src[i] != '\0' && j < 3) 
        {
            char c = src[i];
            if (c >= 'a' && c <= 'z') c -= 32;
            dest[8 + j] = c;
            i++;
            j++;
        }
    }
}

int fat32_init_volume(struct fat32_volume *vol, struct ata_channel *ch, uint8_t dev_idx)
{
    uint16_t sector_buf[256];

    vol->ch = ch;
    vol->drive = dev_idx;
    vol->start_lba = 0;

    /*чтение MBR*/
    if (ata_read_sector(ch, dev_idx, 0, sector_buf) != 0) return -1;

    struct mbr_layout *mbr = (struct mbr_layout*)sector_buf;
    if (mbr->sign != MBR_SIGN) return -2;

    uint32_t part_lba = 0;
    for(int i = 0; i < 4; i++)
    {
        if (mbr->partitions[i].sizein_sectors > 0) 
        {
            part_lba = mbr->partitions[i].starting_lba;
            break;
        }
    }

    vol->start_lba = part_lba;

    /*чтение BPB*/
    if (ata_read_sector(ch, dev_idx, vol->start_lba, sector_buf) != 0) return -3;

    struct fat32_bpb *bpb = (struct fat32_bpb*)sector_buf;

    /*проверка на FAT32*/
    if (memcmp(bpb->fs_type, "FAT32   ", 8) != 0)
    {
        if (sector_buf[255] != MBR_SIGN) return -4;
    }

    vol->sectors_per_cluster = bpb->sectors_per_cluster;
    vol->root_cluster = bpb->root_cluster_num;

    vol->fat_start_lba = vol->start_lba + bpb->res_sector_count;

    vol->clusters_start_lba = vol->fat_start_lba + (bpb->num_fats * bpb->fat_size_32);

    return 0;
}

int fat32_find_file(struct fat32_volume *vol, const char *filename, struct fat32_dir_entry *out_entry) 
{
    uint16_t sector_buffer[256];
    uint32_t current_cluster = vol->root_cluster;

    
    while (current_cluster < 0x0FFFFFF8) {
        for (uint32_t s = 0; s < vol->sectors_per_cluster; s++) {
            uint32_t sector_lba = cluster_to_lba(vol, current_cluster) + s;
            
            if (ata_read_sector(vol->ch, vol->drive, sector_lba, sector_buffer) != 0) return -1;

            struct fat32_dir_entry *entries = (struct fat32_dir_entry*)sector_buffer;
            
            for (int i = 0; i < 16; i++) {
                if (entries[i].name[0] == 0x00) return -2;
                
                if ((uint8_t)entries[i].name[0] == 0xE5) continue;
                
                if (entries[i].attr == 0x0F) continue;

                if (memcmp(entries[i].name, filename, 11) == 0) 
                {
                    *out_entry = entries[i];
                    return 0; 
                }
            }
        }
        
        current_cluster = fat32_get_next_cluster(vol, current_cluster);
        if (current_cluster == 0) return -1;
    }

    return -2; 
}

int fat32_read(struct fat32_volume *vol, struct fat32_dir_entry *entry, uint8_t *buf) 
{
    uint16_t sector_buffer[256];
    
    uint32_t current_cluster = ((uint32_t)entry->cluster_num_high << 16) | entry->cluster_num_low;
    uint32_t bytes_left = entry->file_size;
    uint32_t buf_offset = 0;

    while (current_cluster < 0x0FFFFFF8 && bytes_left > 0) {
        
        for (uint32_t s = 0; s < vol->sectors_per_cluster && bytes_left > 0; s++) {
            uint32_t sector_lba = cluster_to_lba(vol, current_cluster) + s;
            
            if (ata_read_sector(vol->ch, vol->drive, sector_lba, sector_buffer) != 0) return -1;

            uint32_t bytes_to_copy = (bytes_left > 512) ? 512 : bytes_left;
            
            for (uint32_t i = 0; i < bytes_to_copy; i++) buf[buf_offset + i] = ((uint8_t*)sector_buffer)[i];

            buf_offset += bytes_to_copy;
            bytes_left -= bytes_to_copy;
        }

        current_cluster = fat32_get_next_cluster(vol, current_cluster);
        if (current_cluster == 0) return -1;
    }

    return 0;
}

int fat32_write(struct fat32_volume *vol, struct fat32_dir_entry *entry, const uint8_t *buf, uint32_t size) 
{
    uint16_t sector_buffer[256];
    
    uint32_t current_cluster = ((uint32_t)entry->cluster_num_high << 16) | entry->cluster_num_low;
    
    if (current_cluster == 0) {
        current_cluster = fat32_find_free_cluster(vol);
        if (current_cluster == 0) return -1;
        
        fat32_set_cluster(vol, current_cluster, 0x0FFFFFFF);
        entry->cluster_num_low = current_cluster & 0xFFFF;
        entry->cluster_num_high = (current_cluster >> 16) & 0xFFFF;
    }

    uint32_t bytes_written = 0;
    uint32_t cluster_offset = 0;
    uint32_t bytes_per_cluster = vol->sectors_per_cluster * 512;

    while (bytes_written < size) 
    {
        uint32_t sector_in_cluster = (cluster_offset) / 512;
        uint32_t byte_in_sector = (cluster_offset) % 512;
        uint32_t sector_lba = vol->clusters_start_lba + ((current_cluster - 2) * vol->sectors_per_cluster) + sector_in_cluster;

        if (byte_in_sector > 0 || (size - bytes_written) < 512) 
            ata_read_sector(vol->ch, vol->drive, sector_lba, sector_buffer);

        uint32_t chunk = 512 - byte_in_sector;
        if (chunk > (size - bytes_written)) chunk = size - bytes_written;

        for (uint32_t i = 0; i < chunk; i++) 
            ((uint8_t*)sector_buffer)[byte_in_sector + i] = buf[bytes_written + i];

        if (ata_write_sector(vol->ch, vol->drive, sector_lba, sector_buffer) != 0) return -1;

        bytes_written += chunk;
        cluster_offset += chunk;

        if (cluster_offset >= bytes_per_cluster && bytes_written < size) 
        {
            uint32_t next_cluster = fat32_get_next_cluster(vol, current_cluster);
            
            if (next_cluster >= 0x0FFFFFF8) 
            {
                next_cluster = fat32_find_free_cluster(vol);
                if (next_cluster == 0) return -1; 
                
                fat32_set_cluster(vol, current_cluster, next_cluster);
                fat32_set_cluster(vol, next_cluster, 0x0FFFFFFF);
            }
            
            current_cluster = next_cluster;
            cluster_offset = 0;
        }
    }

    if (size > entry->file_size) entry->file_size = size;
    
    fat32_update_dir_entry(vol, entry);

    return bytes_written;
}

int fat32_open(struct vfs_file *file, const char *path) 
{
    struct fat32_volume *vol = (struct fat32_volume *)file->node->priv_data;
    struct fat32_dir_entry entry;
    
    if (fat32_find_file(vol, path, &entry) != 0) return -1;

    uint32_t first_cluster = ((uint32_t)entry.cluster_num_high << 16) | entry.cluster_num_low;

    file->offset = 0;
    file->size = entry.file_size;

    struct fat32_file_info *finfo = kmalloc(sizeof(struct fat32_file_info));
    if (!finfo) return -1;

    finfo->first_cluster = first_cluster;
    finfo->current_cluster = first_cluster;
    finfo->current_cluster_idx = 0;

    file->priv_file_data = finfo;
    return 0;
}

int fat32_close(struct vfs_file *file) 
{
    if (file->priv_file_data) 
    {
        kfree(file->priv_file_data);
        file->priv_file_data = NULL;
    }
    return 0;
}

int fat32_vfs_read(struct vfs_file *file, uint8_t *buf, uint32_t size) 
{
    struct fat32_volume *vol = (struct fat32_volume *)file->node->priv_data;
    struct fat32_file_info *finfo = (struct fat32_file_info *)file->priv_file_data;

    if (file->offset >= file->size) return 0;
    if (file->offset + size > file->size) size = file->size - file->offset;

    struct fat32_dir_entry fake_entry;
    fake_entry.cluster_num_high = (finfo->first_cluster >> 16) & 0xFFFF;
    fake_entry.cluster_num_low = finfo->first_cluster & 0xFFFF;
    fake_entry.file_size = file->size;

    uint8_t *full_buf = kmalloc(file->size);
    if (!full_buf) return -1;

    if (fat32_read(vol, &fake_entry, full_buf) != 0) 
    {
        kfree(full_buf);
        return -1;
    }

    for (uint32_t i = 0; i < size; i++) 
        buf[i] = full_buf[file->offset + i];
    

    kfree(full_buf);
    file->offset += size;
    return size;
}

int fat32_vfs_write(struct vfs_file *file, const uint8_t *buf, uint32_t size) 
{
    struct fat32_volume *vol = (struct fat32_volume *)file->node->priv_data;
    struct fat32_file_info *finfo = (struct fat32_file_info *)file->priv_file_data;

    struct fat32_dir_entry entry;
    entry.cluster_num_high = (finfo->first_cluster >> 16) & 0xFFFF;
    entry.cluster_num_low = finfo->first_cluster & 0xFFFF;
    entry.file_size = file->size;

    int written = fat32_write(vol, &entry, buf, size);
    if (written > 0) 
    {
        file->offset += written;
        if (file->offset > file->size) file->size = file->offset;
    }
    return written;
}

int fat32_readdir(struct vfs_file *file, struct vfs_dirent *dir, uint32_t index) 
{
    struct fat32_volume *vol = (struct fat32_volume *)file->node->priv_data;
    struct fat32_file_info *finfo = (struct fat32_file_info *)file->priv_file_data;
    
    uint16_t sector_buffer[256];
    uint32_t current_cluster = finfo->first_cluster; 
    if (current_cluster == 0) current_cluster = vol->root_cluster;

    uint32_t current_valid_index = 0;

    while (current_cluster < 0x0FFFFFF8) 
    {
        for (uint32_t s = 0; s < vol->sectors_per_cluster; s++) 
        {
            uint32_t sector_lba = cluster_to_lba(vol, current_cluster) + s;
            if (ata_read_sector(vol->ch, vol->drive, sector_lba, (uint16_t*)sector_buffer) != 0) return -1;

            struct fat32_dir_entry *entries = (struct fat32_dir_entry*)sector_buffer;
            for (int i = 0; i < 16; i++) 
            {
                if (entries[i].name[0] == 0x00) return -1; 
                if ((uint8_t)entries[i].name[0] == 0xE5) continue; 
                if (entries[i].attr == 0x0F) continue; 
                if (entries[i].attr & 0x08) continue; 

                if (current_valid_index == index) {
                    dir->size = entries[i].file_size;
                    dir->is_dir = (entries[i].attr & 0x10) ? 1 : 0;
                    fat32_format_name(entries[i].name, dir->name);
                    return 0;
                }
                current_valid_index++;
            }
        }
        current_cluster = fat32_get_next_cluster(vol, current_cluster);
    }
    return -1;
}

int fat32_create(struct vfs_node *node, const char *path) 
{
    struct fat32_volume *vol = (struct fat32_volume *)node->priv_data;
    char fat_name[11];
    
    if (path[0] == '/') path++;
    fat32_format_name(path, fat_name);

    uint32_t current_cluster = vol->root_cluster;
    uint16_t sector_buf[256];

    while (current_cluster >= 2 && current_cluster < 0x0FFFFFF8) 
    {
        uint32_t lba = cluster_to_lba(vol, current_cluster);

        for (uint32_t s = 0; s < vol->sectors_per_cluster; s++) 
        {
            if (ata_read_sector(vol->ch, vol->drive, lba + s, sector_buf) != 0) return -1;
            
            struct fat32_dir_entry *entries = (struct fat32_dir_entry *)sector_buf;

            for (int i = 0; i < 16; i++) 
            { // 512 байт / 32 байта = 16 записей на сектор
                // 0x00 - свободная запись и все за ней, 0xE5 - старая удаленная запись
                if (entries[i].name[0] == 0x00 || (uint8_t)entries[i].name[0] == 0xE5) {
                    
                    for(int k=0; k<11; k++) entries[i].name[k] = fat_name[k];
                    entries[i].attr = 0x00; // Обычный файл
                    entries[i].nt_res = 0;
                    entries[i].cluster_num_high = 0; // Кластер пока не выделен (размер 0)
                    entries[i].cluster_num_low = 0;
                    entries[i].file_size = 0;
                    
                    if (ata_write_sector(vol->ch, vol->drive, lba + s, sector_buf) != 0) return -1;
                    return 0;
                }
            }
        }
        current_cluster = fat32_get_next_cluster(vol, current_cluster);
    }
    return -1; 
}

struct vfs_node_ops fat32_node_ops = {
    .open    = fat32_open,
    .read    = fat32_vfs_read, 
    .write   = fat32_vfs_write, 
    .close   = fat32_close,
    .create  = fat32_create,
    .readdir = fat32_readdir
};