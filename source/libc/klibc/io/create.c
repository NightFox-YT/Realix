#define __RLIBC_INTERNAL_FS__
#include "io.h"

/* internal functions */
int kcreate(const char *path)
{
    return sys_create_impl(path);
}