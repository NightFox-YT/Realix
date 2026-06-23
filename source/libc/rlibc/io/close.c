#define __RLIBC_INTERNAL_FS__
#include "io.h"

/* internal functions */
int close(int fd) 
{
    return syscall(SYS_CLOSE, (uint32_t)fd, 0, 0);
}

int fclose(FILE *stream) 
{
    if (!stream || stream->fd == -1) return -1;

    int res = close(stream->fd);
    
    stream->fd = -1; 
    
    return res;
}