/*
файл содержит реализацию ВСЕХ функций с названием print
(C) 2026 Alexander Silaev
The realix libc, IO module.
*/

#include "io.h"
#include "../../include/sysenter.h"
#include "../stdlib.h"

int vsnprintf(char *str, size_t size, const char *format, va_list ap)
{
    if (!str || size == 0) return 0;
    size_t written = 0;
    const char *p = format;
    size_t limit = size - 1;

    while (*p != '\0') 
    {
        if (*p == '%') 
        {
            p++;
            
            while (*p == '0' || (*p >= '1' && *p <= '9') || *p == '.' || *p == '-') 
            {
                p++;
            }
            
            switch (*p) {
                case 'c': {
                    char c = (char)va_arg(ap, int);
                    if (written < limit) str[written++] = c;
                    break;
                }
                case 's': {
                    char *s = va_arg(ap, char *);
                    if (!s) s = "(null)";
                    while (*s && written < limit) str[written++] = *s++;
                    break;
                }
                case 'd':
                case 'i': {
                    int n = va_arg(ap, int);
                    char num_buf[32];
                    if (n < 0) {
                        if (written < limit) str[written++] = '-';
                        uint32_t abs_n = (uint32_t)(-n);
                        xtoa(abs_n, num_buf, 10, 0);
                    } else {
                        xtoa((uint32_t)n, num_buf, 10, 0);
                    }
                    char *nb = num_buf;
                    while (*nb && written < limit) str[written++] = *nb++;
                    break;
                }
                case 'u': {
                    uint32_t u = va_arg(ap, uint32_t);
                    char num_buf[32];
                    xtoa(u, num_buf, 10, 0);
                    char *nb = num_buf;
                    while (*nb && written < limit) str[written++] = *nb++;
                    break;
                }
                case 'x':
                case 'X': {
                    uint32_t x = va_arg(ap, uint32_t);
                    char num_buf[32];
                    xtoa(x, num_buf, 16, (*p == 'X'));
                    char *nb = num_buf;
                    while (*nb && written < limit) str[written++] = *nb++;
                    break;
                }
                case '%': {
                    if (written < limit) str[written++] = '%';
                    break;
                }
                default:
                    if (written < limit) str[written++] = *p;
                    break;
            }
            if (*p != '\0') p++; 
        } else {
            if (written < limit) {
                str[written++] = *p;
            }
            p++; 
        }
    }
    str[written] = '\0';
    return (int)written;
}


int snprintf (char *str, size_t size, const char *format, ...)
{
	va_list ap;
	int rc;

	va_start(ap, format);
	rc = vsnprintf(str, size, format, ap);
	va_end(ap);

	return rc;
}

#define PRINTF_BUFFER_SIZE 256

int vprintf(const char *format, va_list ap)
{
    char buf[PRINTF_BUFFER_SIZE];
    
    int written = vsnprintf(buf, PRINTF_BUFFER_SIZE, format, ap);
    
    if (written > 0) sys_puts(buf);
    
    return written;
}

int printf(const char *format, ...)
{
    va_list ap;
    int rc;

    va_start(ap, format);
    rc = vprintf(format, ap);
    va_end(ap);

    return rc;
}

int sprintf(char *str, const char *format, ...)
{
	va_list ap;
	int rc;

	va_start(ap, format);
	rc = vsnprintf(str, (size_t)-1, format, ap);
	va_end(ap);

	return rc;
}