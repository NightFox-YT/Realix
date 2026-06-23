#include "ramfs.h"
#include "../libc/klibc/stdint.h"
#include "../libc/klibc/string/string.h"

static void *ramfs_start_addr = 0;

void ramfs_init(void *ramfs_base)
{
    ramfs_start_addr = ramfs_base;
}

void *ramfs_find_file(const char *filename)
{
    uint8_t *ptr = (uint8_t*)ramfs_start_addr;

    if (!ptr) return NULL;

    while (1)
    {
        struct ramfs_header *head = (struct ramfs_header*)ptr;

        if (head->name[0] == '\0') break;

        ptr += sizeof(struct ramfs_header);

        if (strcmp(head->name, filename) == 0)
            return (void*)ptr;

        ptr += head->size;
    }

    return NULL;
}

int ramfs_load_driver(void *rcom_addr)
{
    if (!rcom_addr) return -1;

    struct rcom_header *header = (struct rcom_header*)rcom_addr;

    uint32_t *magic_ptr = (uint32_t*)header->magic;
    if (*magic_ptr != __rcom_magic_LE)
        return -242;

    if (header->magic_check != __driver_magic_LE) return -342;

    uint32_t init_entry_addr = (uint32_t)rcom_addr + header->exec_offset;

    int (*driver_init)(void) = (int (*)(void))init_entry_addr;

    int status = driver_init();

    return status;
}