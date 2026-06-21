/* The Realix VFS Close operations
* © 2026 Alexander Silaev.
*/
#define __RVFS_INTERNAL_FUNCTIONS__
#include "vfs.h"
#include "../libc/string/string.h"

int vfs_close(struct vfs_file *file) 
{
    if (!file) return -1;

    if (file->node && file->node->oprs && file->node->oprs->close) 
        file->node->oprs->close(file);

    if (file->priv_file_data) 
        kfree(file->priv_file_data);

    kfree(file);
    return 0;
}
