/* The RXBDPH an Realix Basic Display Place Holder, basic VGA 24-bit color mode driver
 * for Realix.
 * © 2026 Alexander Silaev
 * SPDX-License-Identifier: GPL-3-or-later
 */

#ifndef __realix_rxbdph__
#define __realix_rxbdph__

#include "../../libc/klibc/stdint.h"
#include "../../libc/klibc/stddef.h"
#include "../../libc/klibc/stdbool.h"

#if defined (__cplusplus)
extern "C" {
#endif /* __cplusplus */

/* === VBE Mode Info STRUCTURE === */
struct vbe_mode_info {
    uint16_t attributes;
    uint8_t win_a;
    uint8_t win_b;
    uint16_t granularity;
    uint16_t win_size;
    uint16_t segment_a;
    uint16_t segment_b;
    uint32_t win_func_ptr;
    uint16_t pitch;
    uint16_t width;
    uint16_t height;
    uint8_t w_char;
    uint8_t y_char;
    uint8_t planes;
    uint8_t bpp;
    uint8_t banks;
    uint8_t memory_model;
    uint8_t bank_size;
    uint8_t image_pages;
    uint8_t reserved0;

    uint8_t red_mask;
    uint8_t red_pos;
    uint8_t green_mask;
    uint8_t green_pos;
    uint8_t blu_mask;
    uint8_t blu_pos;
    uint8_t res_mask;
    uint8_t res_pos;
    uint8_t direct_color_attributes;

    uint32_t framebuffer;
    uint32_t off_screen_mem_off;
    uint16_t off_screen_mem_size;
    uint8_t reserved1[206];
} __attribute ((packed));

/* === Framebuffer Context === */
typedef struct {
    uint32_t *fb;
    uint16_t width;
    uint16_t height;
    uint16_t bpp;
    uint16_t pitch;
    uint8_t initialized;
} fb_ctx_t;

/* === 24-bit VGA RGB color, as 0x00RRGGBB === */
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

#define FONT_WIDTH 8
#define FONT_HEIGHT 16

/* === FUNCTIONS === */
int rxbdph_init (struct kernel_io_interfaces *io);
int rxbdph_init_hardware(uint16_t width, uint16_t height);

void rxbdph_put_pixel (uint16_t x, uint16_t y, uint32_t color);
uint32_t rxbdph_get_pixel (uint16_t x, uint16_t y);

void rxbdph_draw_line (uint16_t x0, uint16_t y0, uint16_t x1, uint16_t y1, uint32_t color);
void rxbdph_draw_rect (uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color);
void rxbdph_fill_rect (uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color);
void rxbdph_draw_circle (uint16_t cx, uint16_t cy, uint16_t r, uint32_t color);
void rxbdph_fill_circle (uint16_t cx, uint16_t cy, uint16_t r, uint32_t color);

void rxbdph_clear (uint32_t color);
void rxbdph_clear_black (void);

void rxbdph_draw_char (uint16_t x, uint16_t y, char c, uint32_t fg, uint32_t bg, int transparent);
void rxbdph_draw_string (uint16_t x, uint16_t y, const char *str, uint32_t fg, uint32_t bg, int transparent);
void rxbdph_draw_stringn (uint16_t x, uint16_t y, const char *str, size_t n, uint32_t fg, uint32_t bg, int transparent);

uint16_t rxbdph_get_width (void);
uint16_t rxbdph_get_height (void);
uint8_t rxbdph_is_initialized (void);
   /* === END === */
#if defined (__cplusplus)
}
#endif /* __cplusplus */
#endif /* __realix_rxbdph__ */
