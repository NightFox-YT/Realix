/* string.h
    © 2026 Alexander Silaev
    The Realix libc
*/

#ifndef __realix_string_h__
#define __realix_string_h__

#include "../stddef.h"

/* Сравнение */
int strcmp(const char *stro, const char *strt);
int memcmp(const void *s1, const void *s2, size_t n);

/* Копирование и заполнение */
void *memcpy(void *dest, const void *src, size_t n);
void *memmove(void *destination, const void *source, size_t n);
void *memset(void *s, int c, size_t n);

/* Работа со строками */
size_t strlen(const char *str);
char *strcpy(char *dest, const char *src);
char *strncpy(char *dest, const char *src, size_t n);

#endif /* __realix_string_h__ */