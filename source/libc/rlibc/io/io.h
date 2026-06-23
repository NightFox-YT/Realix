#ifndef __realix_io_h__
#define __realix_io_h__

#include "../stddef.h"
#include "../stdint.h"
#include "../stdarg.h"
#include "../../vfs/vfs.h"
#include "../../include/sysenter.h"

#define FILE_BUF_SIZE 512

typedef struct {
	int fd;
	char buffer[FILE_BUF_SIZE];
	int buffer_pos;
	int buffer_size;
	int mode;
	int eof;
} FILE;

extern FILE *stdin;
extern FILE *stdout;
extern FILE *stderr;

FILE *fopen(const char *path, const char *mode);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream);
int fclose(FILE *stream);

int vsnprintf(char *str, size_t maxlen, const char *format, va_list ap);
int vprintf(const char *format, va_list ap);
int snprintf (char *str, size_t size, const char *format, ...);
int sprintf(char *str, const char *format, ...);
int printf(const char *format, ...);

#ifdef __RLIBC_INTERNAL_FS__
int open(const char *path);
int read(int fd, void *buf, uint32_t size);
int write(int fd, const void *buf, uint32_t size);
int close(int fd);
int create(const char *path);
int readdir(int fd, struct vfs_dirent *dir, uint32_t index);
#endif
#endif /*__realix_io_h__*/