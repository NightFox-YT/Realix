/* strlen.c/string length
    © 2026 Alexander Silaev
    The Realix libc
*/
#include "string.h"

size_t StrLen(const char *str)
{
    const char *s = str;
    while (*s) 
    {
        s++;
    }
    return s - str;
}