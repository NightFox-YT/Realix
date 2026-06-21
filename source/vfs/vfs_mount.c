/* The Realix VFS Mount operations
* © 2026 Alexander Silaev.
*/
#define __RVFS_INTERNAL_FUNCTIONS__
#include "vfs.h"
#include "../libc/string/string.h"

int vfs_mount(const char *path, struct vfs_node_ops *ops, void *priv_data)
{
    char mount_name[VFS_MAX_NAME_LEN];
    if (vfs_get_first_part(path, mount_name) != 0) return -1;
    if (vfs_find_mount(path) != 0) return -2;

    struct vfs_node *new_node = (struct vfs_node*)kmalloc(sizeof(struct vfs_node));
    if (!new_node) return -3;

    strcpy(new_node->name, mount_name);
    new_node->type = VFS_TYPE_DIR;
    new_node->oprs = ops;
    new_node->priv_data = priv_data;
    new_node->next = 0;

    if (mount_list_head == 0) mount_list_head = new_node;
    else {
        struct vfs_node *cur = mount_list_head;
        while (cur->next != 0) cur = cur->next;
        cur->next = new_node;
    }
    return 0;
}
