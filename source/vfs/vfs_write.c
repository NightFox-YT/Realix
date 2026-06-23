/* The Realix VFS Write operations
* © 2026 Alexander Silaev.
*/
#define __RVFS_INTERNAL_FUNCTIONS__
#include "vfs.h"
#include "../libc/klibc/string/string.h"

int vfs_write(struct vfs_file *file, const uint8_t *buf, uint32_t size) 
{
    if (!file || !file->node || !file->node->oprs || !file->node->oprs->write) return -1;

    int bytes_written = file->node->oprs->write(file, buf, size);
    if (bytes_written > 0) 
    {
        file->offset += bytes_written;
        if (file->offset > file->size) 
            file->size = file->offset;
    }
    return bytes_written;
}