/* strcpy.c/string copy
    © 2026 Alexander Silaev
    The Realix libc
*/

char *StrCopy(char *dest, const char *src)
{
    char *d = dest;
    while ((*d++ = *src++));
    return dest;
}