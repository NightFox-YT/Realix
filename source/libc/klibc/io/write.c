#define __RLIBC_INTERNAL_FS__
#include "io.h"

/* internal functions */
int kwrite(int fd, const void *buf, uint32_t size) 
{
    return sys_write_impl(fd, buf, size);
}