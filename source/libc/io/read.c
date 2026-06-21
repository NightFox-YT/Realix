#define __RLIBC_INTERNAL_FS__
#include "io.h"

/* internal functions */
int read(int fd, void *buf, uint32_t size) 
{
    return syscall(SYS_READ, (uint32_t)fd, (uint32_t)buf, (uint32_t)size);
}

int readdir(int fd, struct vfs_dirent *dir, uint32_t index) 
{
    return syscall(SYS_READDIR, (uint32_t)fd, (uint32_t)dir, (uint32_t)index);
}