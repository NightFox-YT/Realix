/* The Realix VFS Create operations
* © 2026 Alexander Silaev.
*/
#define __RVFS_INTERNAL_FUNCTIONS__
#include "vfs.h"
#include "../libc/string/string.h"

int vfs_create(const char *path) 
{
    struct vfs_node *node = vfs_find_mount(path);
    if (!node || !node->oprs || !node->oprs->create) return -1;

    const char *pure_path = vfs_get_pure_path(path);
    return node->oprs->create(node, pure_path);
}