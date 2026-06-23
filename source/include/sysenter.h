#ifndef __realix_sysenter__
#define __realix_sysenter__

#include "../libc/klibc/stdint.h"
#include "../libc/klibc/stddef.h"
#include "../vfs/vfs.h"

#define IA32_SYSENTER_CS 0x174
#define IA32_SYSENTER_ESP 0x175
#define IA32_SYSENTER_EIP 0x176

#define FONT_WIDTH 8
#define FONT_HEIGHT 16

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

/*-----------EXTERN AREA------------------*/
extern char keyboard_getc(void);

extern void rxbdph_put_pixel (uint16_t x, uint16_t y, uint32_t color);
extern uint32_t rxbdph_get_pixel (uint16_t x, uint16_t y);

extern void rxbdph_draw_line (uint16_t x0, uint16_t y0, uint16_t x1, uint16_t y1, uint32_t color);
extern void rxbdph_draw_rect (uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color);
extern void rxbdph_fill_rect (uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color);
extern void rxbdph_draw_circle (uint16_t cx, uint16_t cy, uint16_t r, uint32_t color);
extern void rxbdph_fill_circle (uint16_t cx, uint16_t cy, uint16_t r, uint32_t color);

extern void rxbdph_clear (uint32_t color);
extern void rxbdph_clear_black (void);

extern void rxbdph_draw_char (uint16_t x, uint16_t y, char c, uint32_t fg, uint32_t bg, int transparent);
extern void rxbdph_draw_string (uint16_t x, uint16_t y, const char *str, uint32_t fg, uint32_t bg, int transparent);
extern void rxbdph_draw_stringn (uint16_t x, uint16_t y, const char *str, size_t n, uint32_t fg, uint32_t bg, int transparent);

extern uint16_t rxbdph_get_width (void);
extern uint16_t rxbdph_get_height (void);
extern uint8_t rxbdph_is_initialized (void);

#define RGB(r, g, b) (((uint32_t)(r) << 16) | ((uint32_t)(g) << 8) | (uint32_t)(b))
#define VGA_BLACK RGB(0, 0, 0)
#define VGA_WHITE RGB(255, 255, 255)
#define VGA_RED RGB(200, 50, 50)
#define VGA_GREEN RGB(50, 180, 50)
#define VGA_BLUE RGB(50, 100, 220)
#define VGA_YELLOW RGB(220, 200, 50)
#define VGA_CYAN RGB(50, 200, 220)
#define VGA_MAGENTA RGB(200, 50, 180)
#define VGA_GRAY RGB(128, 128, 128)
#define VGA_DGRAY RGB(64, 64, 64)
#define VGA_LGRAY RGB(192, 192, 192)

/*-----------------END-------------------------*/

int sys_create_impl(const char *path);
int sys_readdir_impl(int fd, struct vfs_dirent *dir, uint32_t index);
int sys_close_impl(int fd);
int sys_write_impl(int fd, const uint8_t *buf, uint32_t size);
int sys_read_impl(int fd, uint8_t *buf, uint32_t size);
int sys_open_impl(const char *path);
uint32_t sys_gets_impl(int fd, void *buf, uint32_t count);
uint32_t sys_puts_impl(const char *str);

#endif