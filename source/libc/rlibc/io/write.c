#define __RLIBC_INTERNAL_FS__
#include "io.h"

/* internal functions */
int write(int fd, const void *buf, uint32_t size) 
{
    return syscall(SYS_WRITE, (uint32_t)fd, (uint32_t)buf, (uint32_t)size);
}