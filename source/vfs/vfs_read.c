/* The Realix VFS Read operations
* © 2026 Alexander Silaev.
*/
#define __RVFS_INTERNAL_FUNCTIONS__
#include "vfs.h"
#include "../libc/klibc/string/string.h"

int vfs_read(struct vfs_file *file, uint8_t *buf, uint32_t size) 
{
    if (!file || !file->node || !file->node->oprs || !file->node->oprs->read) return -1;

    //Защита от чтения за границами файла
    if (file->offset >= file->size) return 0;
    if (file->offset + size > file->size) 
        size = file->size - file->offset;
    

    int bytes_read = file->node->oprs->read(file, buf, size);
    if (bytes_read > 0) file->offset += bytes_read;
    
    return bytes_read;
}

int vfs_readdir(struct vfs_file *file, struct vfs_dirent *dir, uint32_t index) 
{
    if (!file || !file->node || !file->node->oprs || !file->node->oprs->readdir) return -1;
    return file->node->oprs->readdir(file, dir, index);
}