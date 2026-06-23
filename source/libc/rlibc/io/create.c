#define __RLIBC_INTERNAL_FS__
#include "io.h"

/* internal functions */
int create(const char *path)
{
    return syscall(SYS_CREATE, (uint32_t)path, 0, 0);
}