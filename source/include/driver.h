#ifndef __realix_driver__
#define __realix_driver__

#include "../libc/klibc/stdint.h"
#include "../libc/klibc/stddef.h"

#define __driver_magic_LE 0x53484954
#define __driver_magic_BE 0x54494853
#define __rcom_magic_LE 0x52434F4D
#define __rcom_magic_BE 0x4D4F4352

#define rx_packed __attribute__((packed))

struct kernel_io_interfaces {
    void (*put_pixel)(int x, int y, uint32_t color);
    void (*draw_char)(char c, int x, int y, uint32_t fg, uint32_t bg, int t);
    int (*get_char)(void);
    uint32_t (*get_pixel)(uint16_t x, uint16_t y);
    void (*draw_line)(int16_t x0, uint16_t y0, uint16_t x1, uint16_t y1, uint32_t color);
    void (*draw_rect)(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color);
    void (*draw_circle)(uint16_t cx, uint16_t cy, uint16_t r, uint32_t color);
    void (*draw_string)(uint16_t x, uint16_t y, const char *s, uint32_t fg, uint32_t bg, int transparent);
    void (*draw_stringn)(uint16_t x, uint16_t y, const char *s, size_t n, uint32_t fg, uint32_t bg, int transparent);
    void (*fill_rect)(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color);
    void (*fill_circle)(uint16_t cx, uint16_t cy, uint16_t r, uint32_t color);
    uint16_t (*get_width)(void);
    uint16_t (*get_height)(void);
    uint8_t (*is_graphics_initialized)(void);
    void (*clear)(uint32_t color);
    void (*clear_black)(void);
    void* (*kmalloc)(uint32_t size);
    void* (*memset)(void *s, int c, size_t n);
    void (*kfree)(void *ptr);
    void* (*resolve_symbol)(const char *name);
};

struct rx_packed rcom_header {
    char     magic[4];            // 0x00: Сигнатура самого формата
    uint32_t magic_check;        // 0x04: Сигнатура драйвера
    uint32_t exec_offset;       // 0x08: Смещение от начала файла до точки входа
    uint32_t component_size;   // 0x0C: Размер чистого исполняемого кода в байтах
    char     name[16];        // 0x10: Имя компонента в ASCII 
};

#define REALIX_COMPONENT(comp_name, init_func) \
    __attribute__((section(".header"))) \
    struct rcom_header my_driver = { \
        .magic = { 'R', 'C', 'O', 'M' }, \
        .magic_check = __driver_magic_LE, \
        .exec_offset = 32, \
        .component_size = 0, \
        .name = comp_name \
    }; \
    int driver_init(struct kernel_io_interfaces *io) { return init_func(io); }

#endif /*__realix_driver__*/