/* mem.c
    © 2026 Alexander Silaev
    The Realix libc
*/
#include "string.h"

void *memset(void *s, int c, size_t n)
{
    void *p = s;
    /*Дублируем байт 'c' во все 4 байта 32-битного числа*/
    unsigned int dword_value = (unsigned char)c;
    dword_value |= dword_value << 8;
    dword_value |= dword_value << 16;

    size_t dwords = n >> 2;
    size_t bytes = n & 3;

    /*
        +D - указатель значения
        +с - счетчик итераций
        "a" - значение для заполнения
    */
   /*ass*/__asm__ volatile(
        "rep stosl\n\t"
        "movl %3, %%ecx\n\t"
        "rep stosb"
        : "+D"(p), "+c"(dwords)
        : "a"(dword_value), "r"(bytes)
        : "memory"
   );

   return s;
}

/* memcpy/memory copy */
void *memcpy(void *destination, const void *source, size_t n)
{
    void *d = destination;
    const void *s = source;
    size_t dwords = n >> 2;
    size_t bytes = n & 3;

    /*
        +D - куда копировать
        +S - откуда копировать
        +с - сколько блоков копировать
    */

    __asm__ volatile(
        "rep movsl\n\t"
        "movl %3, %%ecx\n\t"
        "rep movsb"
        : "+D"(d), "+S"(s), "+c"(dwords)
        : "r"(bytes)
        : "memory"
    );

    return destination;
}

/* memmove/memory move (Безопасен при пересечении буферов) */
void *memmove(void *dest, const void *src, size_t n)
{
    unsigned char *d = dest;
    const unsigned char *s = src;

    if (d < s)
    {
        while (n--)
        {
            *d++ = *s++;
        }
    }
    else if (d > s)
    {
        d += n;
        s += n;
        while (n--)
        {
            *--d = *--s;
        }
    }
    return dest;
}

/* memcmp/memory compare */
int memcmp(const void *s1, const void *s2, size_t n)
{
    const unsigned char *p1 = s1;
    const unsigned char *p2 = s2;
    while (n--)
    {
        if (*p1 != *p2)
        {
            return *p1 - *p2;
        }
        p1++;
        p2++;
    }
    return 0;
}