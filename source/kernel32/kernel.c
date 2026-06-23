#include "../include/io.h"
#include "../drivers-32/serial/com.h"
#include "../drivers-32/pci/pci.h"
#include "../include/sysenter.h"
#include "../include/gdt.h"
#include "../include/idt.h"
#include "../include/slab.h"
#include "../drivers-32/rxbdph/graphics.h"
#include "../vfs/vfs.h"
#include "../init/ramfs.h" 
#include "../libc/klibc/string/string.h"

extern void _jump_to_userspace(uint32_t user_eip, uint32_t user_esp);
extern void temporary_user_stub(void);

void temporary_user_stub(void) 
{
    while(1) {
        __asm__ volatile("hlt");
    }
}

struct ata_channel *ata_primary_ptr = NULL;
struct ata_channel *ata_secondary_ptr = NULL;
void (*keyboard_callback)(void) = NULL;

static void kernel_put_pixel_adapter(int x, int y, uint32_t color) {(void)x; (void)y; (void)color;}
static void kernel_draw_char_adapter(char c, int x, int y, uint32_t fg, uint32_t bg, int t) {(void)x; (void)c; (void)y; (void)t; (void)bg; (void)fg;}
static int kernel_get_char_adapter(void) {return 0;}
static uint32_t kernel_get_pixel_adapter(uint16_t x, uint16_t y) { (void)x; (void)y; return 0; }
static void kernel_draw_line_adapter(int16_t x0, uint16_t y0, uint16_t x1, uint16_t y1, uint32_t color) { (void)x0; (void)y0; (void)x1; (void)y1; (void)color; }
static void kernel_draw_rect_adapter(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color) { (void)x; (void)y; (void)w; (void)h; (void)color; }
static void kernel_fill_rect_adapter(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color) { (void)x; (void)y; (void)w; (void)h; (void)color; }
static void kernel_draw_circle_adapter(uint16_t cx, uint16_t cy, uint16_t r, uint32_t color) { (void)cx; (void)cy; (void)r; (void)color; }
static void kernel_fill_circle_adapter(uint16_t cx, uint16_t cy, uint16_t r, uint32_t color) { (void)cx; (void)cy; (void)r; (void)color; }
static void kernel_draw_string_adapter(uint16_t x, uint16_t y, const char *s, uint32_t fg, uint32_t bg, int t) { (void)x; (void)y; (void)s; (void)fg; (void)bg; (void)t; }
static void kernel_draw_stringn_adapter(uint16_t x, uint16_t y, const char *s, size_t n, uint32_t fg, uint32_t bg, int t) { (void)x; (void)y; (void)s; (void)n; (void)fg; (void)t; (void)bg; }
static uint16_t kernel_get_width(void) { return 1024; }
static uint16_t kernel_get_height(void) { return 768; }
static uint8_t kernel_graphics_is_initialized(void) { return 0; }
static void kernel_graphics_clear(uint32_t color) { (void)color; }
static void kernel_graphics_clear_black(void) { }

struct kernel_io_interfaces g_kernel_io = {
    .put_pixel = kernel_put_pixel_adapter,
    .draw_char = kernel_draw_char_adapter,
    .get_char  = kernel_get_char_adapter,
    .get_pixel = kernel_get_pixel_adapter,
    .draw_line = kernel_draw_line_adapter,
    .draw_rect = kernel_draw_rect_adapter,
    .draw_circle = kernel_draw_circle_adapter,
    .draw_string = kernel_draw_string_adapter,
    .draw_stringn = kernel_draw_stringn_adapter,
    .fill_rect = kernel_fill_rect_adapter,
    .fill_circle = kernel_fill_circle_adapter,
    .get_height = kernel_get_height,
    .get_width = kernel_get_width,
    .is_graphics_initialized = kernel_graphics_is_initialized,
    .clear = kernel_graphics_clear,
    .clear_black = kernel_graphics_clear_black,
    .kmalloc = kmalloc,
    .memset = memset,
    .kfree = kfree
};

void kernel_exec(void *ramfs_addr)
{
    gdt_init();
    idt_init(); 
    kmalloc_init();
    
    if (init_serial() == 0) {
        serial_print("Realix: Info: Initialized Serial Logging.\n");
    }

    if (ramfs_addr == NULL) {
        serial_print("Realix: Critical: Ramfs addr is null, stopping system.\n");
        while(1) __asm__ volatile("hlt");
    }
    ramfs_init(ramfs_addr);

    serial_print("Realix: Info: Loading modules...\n");

    void *kbd_module = ramfs_find_file("keyboard.rcom");
    if (kbd_module) {
        ramfs_load_driver(kbd_module);
    }

    //pci_init(); 

    void *ata_module = ramfs_find_file("ata.rcom");
    if (ata_module) {
        ramfs_load_driver(ata_module);
    }

    void *fat_module = ramfs_find_file("fat32.rcom");
    if (fat_module) {
        ramfs_load_driver(fat_module);
    }

    if (g_kernel_io.is_graphics_initialized() == 0) 
    {
        serial_print("Realix: Info: Initializing Display Driver\n");
        g_kernel_io.clear(VGA_LGRAY);
        g_kernel_io.fill_rect(50, 50, 300, 200, VGA_BLUE);
        g_kernel_io.draw_rect(50, 50, 300, 200, VGA_BLACK); 
        g_kernel_io.fill_circle(200, 150, 40, VGA_RED);
        g_kernel_io.draw_line(0, 0, 1024, 768, VGA_YELLOW);
    } 

    uint32_t *sysenter_stack = kmalloc(4096);
    uint32_t sysenter_stack_top = (uint32_t)sysenter_stack + 4096;
    sysenter_init(sysenter_stack_top);
    set_tss_esp0(sysenter_stack_top);
    
    serial_print("Realix: Info: Allocating user stack and dropping to Ring 3...\n");

    uint32_t *user_stack = kmalloc(4096);
    uint32_t user_stack_top = (uint32_t)user_stack + 4096;

    _jump_to_userspace((uint32_t)temporary_user_stub, user_stack_top);

    while(1) {
        __asm__ volatile("hlt");
    }   
}