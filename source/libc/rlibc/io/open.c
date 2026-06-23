#define __RLIBC_INTERNAL_FS__
#include "io.h"

/* internal functions */
int open(const char *path) {
    return syscall(SYS_OPEN, (uint32_t)path, 0, 0);
}

/* пока статически, ибо malloc-а для user space нету*/
#define MAX_USER_FILES 8
static FILE user_files_pool[MAX_USER_FILES];
static int pool_initialized = 0;

FILE *fopen(const char *path, const char *mode)
{
    if (!path || !mode) return NULL;

    if (!pool_initialized)
    {
        for (int i = 0; i < MAX_USER_FILES; i++) user_files_pool[i].fd = -1;
        pool_initialized = 1;
    }

    FILE *file = NULL;
    for (int i = 0; i < MAX_USER_FILES; i++)
    {
        if (user_files_pool[i].fd == -1)
        {
            file = &user_files_pool[i];
            break;
        }
    }
    if (!file) return NULL;

    int fd = -1;

    if (mode[0] == 'r')
    {
        fd = open(path);
        if (fd < 0) return NULL;
    }else if (mode[0] == 'w')
    {
        fd = open(path);
        if (fd < 0)
            if (create(path) == 0) fd = open(path);

        if (fd < 0) return NULL;
    }
    
    file->fd = fd;
    file->buffer_pos = 0;
    file->buffer_size = 0;
    file->eof = 0;

    return file;
}