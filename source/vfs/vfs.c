/* The Realix VFS main
* © 2026 Alexander Silaev.
*/
#define __RVFS_INTERNAL_FUNCTIONS__
#include "vfs.h"
#include "../libc/klibc/string/string.h"

struct vfs_node *mount_table[MAX_MOUNTS];
struct vfs_node *mount_list_head = 0;

int vfs_get_first_part(const char *path, char *output)
{
    if (path[0] != '/') return -1;

    int p = 1;
    int o = 0;

    while (path[p] != '\0' && path[p] != '/' && o < (VFS_MAX_NAME_LEN - 1))
        output[o++] = path[p++];
    output[o] = '\0';

    return (o > 0) ? 0 : -1; 
}

const char* vfs_get_pure_path(const char *path)
{
    if (path[0] != '/') return path;

    int p = 1;

    while (path[p] != '\0' && path[p] != '/') p++;
    if (path[p] == '/') return &path[p+1];
    return &path[p];
}

struct vfs_node* vfs_find_mount(const char *path)
{
    char mount_name[VFS_MAX_NAME_LEN];
    if (vfs_get_first_part(path, mount_name) != 0) return 0;
    struct vfs_node *cur = mount_list_head;
    while (cur != 0) {
        if (strcmp(cur->name, mount_name) == 0) return cur;
        cur = cur->next;
    }
    return 0;
}

void vfs_init(void){ mount_list_head = 0; }