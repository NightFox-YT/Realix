/* memcmp.c
    © 2026 Alexander Silaev <thebinaryblob@gmail.com>
    The Realix libc
*/
#include "string.h"

/* memcmp/memory compare */
int MemCompare(const void *s1, const void *s2, size_t n)
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