#define __KLIBC_INTERNAL_FS__
#include "io.h"

/* internal functions */
int close(int fd) 
{
    return sys_close_impl(fd);
}

int kfclose(FILE *stream) 
{
    if (!stream || stream->fd == -1) return -1;

    int res = close(stream->fd);
    
    stream->fd = -1; 
    
    return res;
}