/* string.h
    © 2026 Alexander Silaev
    The Realix libc
*/

#ifndef __realix_string_h__
#define __realix_string_h__

#include "../stddef.h"

#ifdef __cplusplus
extern "C" {
#endif
/* Сравнение */
int StrCompare(const char *stro, const char *strt);
int MemCompare(const void *s1, const void *s2, size_t n);

/* Копирование и заполнение */
void *MemCopy(void *dest, const void *src, size_t n);
void *MemMove(void *destination, const void *source, size_t n);
void *MemSet(void *s, int c, size_t n);

/* Работа со строками */
size_t StrLen(const char *str);
char *StrCopy(char *dest, const char *src);
char *StrBoundedCopy(char *dest, const char *src, size_t n);

#ifdef __cplusplus
}
#endif

#endif /* __realix_string_h__ */