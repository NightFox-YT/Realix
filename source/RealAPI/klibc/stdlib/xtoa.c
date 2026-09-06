#include "io.h"
void xtoa(uint32_t num, char *buf, uint32_t base, int uppercase) {
    char tmp[32];
    uint32_t i = 0;

    if (num == 0) {
        buf[0] = '0';
        buf[1] = '\0';
        return;
    }

    while (num > 0) {
        uint32_t rem = num % base;
        if (rem < 10) {
            tmp[i++] = rem + '0';
        } else {
            tmp[i++] = rem - 10 + (uppercase ? 'A' : 'a');
        }
        num /= base;
    }

    uint32_t j = 0;
    while (i > 0) {
        buf[j++] = tmp[--i];
    }
    buf[j] = '\0';
}