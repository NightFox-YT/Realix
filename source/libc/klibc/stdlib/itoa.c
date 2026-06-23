#include "io.h"
void itoa(int32_t value, char *str, int base) 
{
    char *rc, *ptr;
    char buf[32];
    ptr = buf;
    
    if (value == 0) {
        *ptr++ = '0';
        *ptr = '\0';
        str[0] = '0'; str[1] = '\0';
        return;
    }

    uint32_t uvalue = value;
    if (value < 0 && base == 10) {
        *str++ = '-';
        uvalue = -value;
    }

    while (uvalue > 0) {
        int rem = uvalue % base;
        *ptr++ = (rem > 9) ? (rem - 10) + 'A' : rem + '0';
        uvalue /= base;
    }

    rc = str;
    while (ptr > buf) {
        *str++ = *--ptr;
    }
    *str = '\0';
}