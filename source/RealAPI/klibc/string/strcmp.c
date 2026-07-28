/* strcmp/string compare.
    © 2026 Alexander Silaev
    The Realix libc.
*/
#include "string.h"

int StrCompare(const char *stro, const char *strt) // args: string one and string two
{
    while (*stro && (*stro == *strt))
    {
        stro++;
        strt++;
    }
    return *(const unsigned char*)stro - *(const unsigned char*)strt;
}