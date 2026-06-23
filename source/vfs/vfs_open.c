/* The Realix VFS Open operations
* © 2026 Alexander Silaev.
*/
#define __RVFS_INTERNAL_FUNCTIONS__
#include "vfs.h"
#include "../libc/klibc/string/string.h"

struct vfs_file* vfs_open(const char *path) 
{
    struct vfs_node *node = vfs_find_mount(path);
    
    if (!node || !node->oprs || !node->oprs->open) return 0;

    struct vfs_file *file = (struct vfs_file*)kmalloc(sizeof(struct vfs_file));
    if (!file) return 0;

    file->node = node;
    file->offset = 0;
    file->size = 0;
    file->priv_file_data = 0;

    const char *pure_path = vfs_get_pure_path(path);
    if (node->oprs->open(file, pure_path) != 0) 
    {
        kfree(file);
        return 0;
    }

    return file;
}