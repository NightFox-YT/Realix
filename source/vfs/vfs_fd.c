/* The Realix VFS File Descriptors
* © 2026 Alexander Silaev.
*/
#define __RVFS_INTERNAL_FUNCTIONS__
#include "vfs.h"
#include "../libc/klibc/string/string.h"

struct vfs_file *global_fd_table[MAX_OPEN_FILES_PER_PROCESS];

int alloc_fd(struct vfs_file *file) 
{
    for (int i = 0; i < MAX_OPEN_FILES_PER_PROCESS; i++) 
    {
        if (global_fd_table[i] == NULL) 
        {
            global_fd_table[i] = file;
            return i; 
        }
    }
    return -1; 
}

struct vfs_file* get_file_by_fd(int fd) 
{
    if (fd < 0 || fd >= MAX_OPEN_FILES_PER_PROCESS) return NULL;
    return global_fd_table[fd];
}

void free_fd(int fd)
{
    if (fd >= 0 && fd < MAX_OPEN_FILES_PER_PROCESS) 
        global_fd_table[fd] = NULL;
}