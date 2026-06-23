#ifndef __realix_init_ramfs__
#define __realix_init_ramfs__

#include "../include/driver.h"

struct rx_packed ramfs_header {
    char name[32];    // Имя файла
    uint32_t size;   // размер в байтах
};

void ramfs_init(void *ramfs_base);
int ramfs_load_driver(void *rcom_addr);
void *ramfs_find_file(const char *filename);
#endif /*__realix_init_ramfs__*/