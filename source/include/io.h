#ifndef __realix_io__
#define __realix_io__

#include <stdint.h>

static inline void outb(uint16_t port, uint8_t val)
{
    __asm__ volatile("outb %b0, %w1" : : "a"(val), "Nd"(port));
}

static inline uint8_t inb(uint16_t port)
{
    uint8_t ret;
    __asm__ volatile("inb %w1, %b0" : "=a"(ret) : "Nd"(port));
    return ret;
}

static inline void outl(uint16_t port, uint32_t val)
{
    __asm__ volatile("outl %%eax, %%dx" : : "a"(val), "d"(port));
}

static inline uint32_t inl(uint16_t port)
{
    uint32_t ret;
    __asm__ volatile("inl %%dx, %%eax" : "=a"(ret) : "d"(port));
    return ret;
}

static inline uint16_t inw(uint16_t port) 
{
    uint16_t result;
    __asm__ volatile ("inw %w1, %w0" : "=a"(result) : "Nd"(port));
    return result;
}

static inline void outw(uint16_t port, uint16_t value) 
{
    __asm__ volatile ("outw %w0, %w1" : : "a"(value), "Nd"(port));
}

static inline void io_wait(void) { outb(0x80, 0); }

#endif /*__realix_io__*/
