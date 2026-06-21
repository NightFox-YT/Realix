#ifndef __realix_sysenter__
#define __realix_sysenter__

#include "../libc/stdint.h"

#define IA32_SYSENTER_CS 0x174
#define IA32_SYSENTER_ESP 0x175
#define IA32_SYSENTER_EIP 0x176

typedef enum {
    SYS_EXIT=0,
    SYS_PUTS=1,
    SYS_GETC=2,
    SYS_READ=3,
    SYS_WRITE=4,
    SYS_MMAP=5,
    SYS_MUNMAP=6,
    SYS_YIELD=7,
    SYS_GETPID=8,
    SYS_FORK=9,
    SYS_EXEC=10,
    SYS_WAIT=11,
    SYS_KILL=12,
    SYS_OPEN=13,
    SYS_CLOSE=14,
    SYS_STAT=15,
    SYS_BRK=16,
    SYS_GETTIMEOFDAY=17,
    SYS_NANOSLEEP=18,
    SYS_MAX=19,
    SYS_GETS=20,
    SYS_READDIR=21,
    SYS_CREATE=22
} SysCount;

int cpu_has_sysenter(void);
void sysenter_init(uint32_t kernel_esp0);
void sysenter_handler(void);
uint32_t sysenter_dispatch(uint32_t num, uint32_t arg1, uint32_t arg2, uint32_t arg3);

static inline uint32_t __attribute__((always_inline)) syscall(uint32_t num, uint32_t a1, uint32_t a2, uint32_t a3)
{
    uint32_t ret;
    /*ass*/__asm__ volatile (
        "push %%ebp\n\t"
        "movl %%esp, %%ebp\n\t"
        "leal 1f, %%edx\n\t"    // EDX = адрес возврата для sysenter
        "movl %%ebp, %%ecx\n\t" // ECX = старый стек для sysenter
        "sysenter\n"
        "1:\n\t"
        "pop %%ebp"
        : "=a"(ret)                               
        : "0"(num), "b"(a1), "S"(a2), "D"(a3)     
        : "memory", "ecx", "edx"                  
    );
    return ret;
}

static inline uint32_t sys_exit(int code) {return syscall(SYS_EXIT, (uint32_t)code, 0, 0);}
static inline uint32_t sys_puts(const char *s) {return syscall(SYS_PUTS, (uint32_t)s, 0, 0);}
static inline uint32_t sys_getc(void) {return syscall(SYS_GETC, 0, 0, 0);}
static inline uint32_t sys_read(int fd, void *buf, uint32_t count) {return syscall(SYS_READ, (uint32_t)fd, (uint32_t)buf, count);}
static inline uint32_t sys_write(int fd, const void *buf, uint32_t count) {return syscall(SYS_WRITE, (uint32_t)fd, (uint32_t)buf, count);}
static inline uint32_t sys_yield(void) {return syscall(SYS_YIELD, 0, 0, 0);}
static inline uint32_t sys_getpid(void) {return syscall(SYS_GETPID, 0, 0, 0);}
static inline uint32_t sys_mmap(void *addr, uint32_t len, int prot) {return syscall(SYS_MMAP, (uint32_t)addr, len, (uint32_t)prot);}
static inline uint32_t sys_brk(void *addr) {return syscall(SYS_BRK, (uint32_t)addr, 0, 0);}


#endif