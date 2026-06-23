#include "../../include/sysenter.h"
#include "../../include/slab.h"
#include "../../init/ramfs.h"
#include "../../drivers-32/serial/com.h"
#include "../../libc/klibc/string/string.h" 

extern struct kernel_io_interfaces g_kernel_io;

/* Minimal process table for future userspace */
#define MAX_PROCS 64

typedef struct {
    uint32_t pid;
    uint32_t state;          /* 0=free, 1=runnable, 2=running, 3=blocked, 4=zombie */
    uint32_t esp0;           /* Kernel stack top */
    uint32_t esp3;           /* User stack pointer */
    uint32_t eip;            /* User entry point */
    uint32_t cr3;            /* Page directory (future) */
    uint32_t brk;            /* Program break */
    uint32_t parent;
    uint32_t exit_code;
} proc_t;

static proc_t procs[MAX_PROCS];
static uint32_t next_pid = 1;
static uint32_t current_pid = 0;     /* 0 = kernel idle */
static int g_sysenter_ready = 0;

/* Kernel stack pool for userspace processes */
#define KSTACK_SIZE 8192
#define KSTACK_PAGES ((MAX_PROCS * KSTACK_SIZE + 4095) / 4096)

static uint8_t *kstack_pool = NULL;

/*=== CPU feature check ===*/

int cpu_has_sysenter(void)
{
    uint32_t eax, ebx, ecx, edx;
    __asm__ volatile ("cpuid"
        : "=a"(eax), "=b"(ebx), "=c"(ecx), "=d"(edx)
        : "a"(1));
    return (edx & (1 << 11)) ? 1 : 0;   /* SEP bit */
}

/*=== Init ===*/

static uint8_t static_kstack_pool[KSTACK_PAGES * 4096] __attribute__((aligned(4096)));

void sysenter_init(uint32_t kernel_esp0)
{
    if (!cpu_has_sysenter()) {
        /* Panic or fallback to int 0x80 */
        return;
    }

   // kmalloc_init();
    
    /* Allocate kernel stack pool */
    kstack_pool = static_kstack_pool;
    
    memset(procs, 0, sizeof(procs));
    memset(kstack_pool, 0, KSTACK_PAGES * 4096);
    
    /* Set up MSRs via assembly */
    extern void _sysenter_init(uint32_t);
    _sysenter_init(kernel_esp0);
    
    g_sysenter_ready = 1;
}

int sysenter_ready(void)
{
    return g_sysenter_ready;
}

/*=== Process management (stub for future) ===*/

static uint32_t alloc_pid(void)
{
    if (next_pid >= MAX_PROCS) return 0;
    return next_pid++;
}

static proc_t *current_proc(void)
{
    if (current_pid == 0 || current_pid >= MAX_PROCS) return NULL;
    return &procs[current_pid];
}

static uint16_t g_term_x = 10;
static uint16_t g_term_y = 30;

/*=== Syscall implementations ===*/

uint32_t sys_puts_impl(const char *str)
{
    if (!str || !g_kernel_io.is_graphics_initialized()) return -1;
    uint16_t screen_width = g_kernel_io.get_width();
    uint16_t screen_height = g_kernel_io.get_height();

    while (*str)
    {
        char c = *str;

        if (c == '\n')
        {
            g_term_x = 10;
            g_term_y += FONT_HEIGHT;

            if (g_term_y + FONT_HEIGHT <= screen_height)
            {
                for (uint16_t px = 0; px < screen_height; px++)
                {
                    for (uint16_t py = 0; py < FONT_HEIGHT; py++) 
                        g_kernel_io.put_pixel(px, g_term_y + py, VGA_BLACK);
                }
            }
        } else if (c == '\r')
        {
            g_term_x = 10;
        } else
        {
            g_kernel_io.draw_char(g_term_x, g_term_y, c, VGA_WHITE, VGA_BLACK, 1);
            g_term_x += FONT_WIDTH;

            if (g_term_x + FONT_WIDTH > screen_width)
            {
                g_term_x = 10;
                g_term_y += FONT_HEIGHT;
            }
        }
        
        if (g_term_y + FONT_HEIGHT > screen_height)
        {
            g_kernel_io.clear_black();
            g_term_x = 10;
            g_term_y = 10;
        }

        str++;
    }
    return 0;
}

uint32_t sys_getc_impl(void)
{
    return g_kernel_io.get_char();
}

uint32_t sys_gets_impl(int fd, void *buf, uint32_t count)
{
    if (fd != 0 || !buf || count == 0) return (uint32_t)-1;

    char *buffer = (char *)buf;
    uint32_t index = 0;

    while (index < (count - 1)) 
    {
        char ch = (char)sys_getc_impl();

        /* Если нажали Enter */
        if (ch == '\n' || ch == '\r') 
        {
            sys_puts_impl("\n");
            break;
        }
        /* Если нажали Backspace */
        else if (ch == '\b') 
        {
            if (index > 0) 
            {
                index--;
                /* Сдвигаем растровый курсор назад */
                if (g_term_x >= 10 + FONT_WIDTH) {
                    g_term_x -= FONT_WIDTH;
                }
                g_kernel_io.draw_char(g_term_x, g_term_y, ' ', VGA_WHITE, VGA_BLACK, 0);
            }
        }
        /* Обычный печатный символ */
        else if (ch >= 32 && ch <= 126) 
        {
            buffer[index++] = ch;
            
            char echo_str[2] = {ch, '\0'};
            sys_puts_impl(echo_str);
        }
    }

    buffer[index] = '\0'; /* Замыкаем строку нуля-терминатором */
    return index;         /* Возвращаем длину прочитанной строки */
}

int sys_open_impl(const char *path)
{
    if (!path) return -1;

    struct vfs_file *file = vfs_open(path);
    if (!file) return -1;

    int fd = alloc_fd(file);
    if (fd == -1)
    {
        vfs_close(file);
        return -1;
    }
    return fd;
}

int sys_read_impl(int fd, uint8_t *buf, uint32_t size) 
{
    if (!buf || size == 0) return 0;
    
    struct vfs_file *file = get_file_by_fd(fd);
    if (!file) return -1;

    return vfs_read(file, buf, size);
}

int sys_write_impl(int fd, const uint8_t *buf, uint32_t size) 
{
    if (!buf || size == 0) return 0;

    struct vfs_file *file = get_file_by_fd(fd);
    if (!file) return -1;

    return vfs_write(file, buf, size);
}

int sys_close_impl(int fd) 
{
    struct vfs_file *file = get_file_by_fd(fd);
    if (!file) return -1;

    int res = vfs_close(file);
    free_fd(fd);
    return res;
}

int sys_readdir_impl(int fd, struct vfs_dirent *dir, uint32_t index) 
{
    if (!dir) return -1;

    struct vfs_file *file = get_file_by_fd(fd);
    if (!file) return -1;

    return vfs_readdir(file, dir, index);
}

int sys_create_impl(const char *path) 
{
    if (!path) return -1;

    return vfs_create(path);
}

static uint32_t sys_exit_impl(int code)
{
    proc_t *p = current_proc();
    if (p) {
        p->state = 4;       /* zombie */
        p->exit_code = (uint32_t)code;
        current_pid = 0;    /* back to kernel */
    }
    
    /* If no userspace procs left, halt */

    __asm__ volatile ("cli; hlt");
    return 0;   /* never reached */
}

/*=== Main dispatcher ===*/

uint32_t sysenter_dispatch(uint32_t num, uint32_t arg1, uint32_t arg2, uint32_t arg3)
{
    if (num >= SYS_MAX) return (uint32_t)-1;
    
    switch (num) {
        case SYS_EXIT:          return sys_exit_impl((int)arg1);
        case SYS_PUTS:          return sys_puts_impl((const char *)arg1);
        case SYS_GETC:          return sys_getc_impl();
        case SYS_GETS:          return sys_gets_impl((int)arg1, (void *)arg2, arg3);
        case SYS_READ:          return sys_read_impl((int)arg1, (uint8_t*)arg2, (uint32_t)arg3);
        case SYS_WRITE:         return sys_write_impl((int)arg1, (const uint8_t*)arg2, (uint32_t)arg3);
        //case SYS_MMAP:          return sys_mmap_impl((void *)arg1, arg2, (int)arg3);
        //case SYS_MUNMAP:        /* TODO */ return (u32)-1;
        //case SYS_YIELD:         return sys_yield_impl();
        //case SYS_GETPID:        return sys_getpid_impl();
        //case SYS_FORK:          /* TODO */ return (u32)-1;
        //case SYS_EXEC:          /* TODO */ return (u32)-1;
        //case SYS_WAIT:          /* TODO */ return (u32)-1;
        //case SYS_KILL:          /* TODO */ return (u32)-1;
        case SYS_OPEN:          return sys_open_impl((const char*)arg1);
        case SYS_CLOSE:         return sys_close_impl((int)arg1);
   //     case SYS_STAT:          /* TODO */ return (u32)-1;
     //   case SYS_BRK:           return sys_brk_impl((void *)arg1);
      //  case SYS_GETTIMEOFDAY:  /* TODO */ return (u32)-1;
       // case SYS_NANOSLEEP:     /* TODO */ return (u32)-1;
        case SYS_READDIR:       return sys_readdir_impl((int)arg1, (struct vfs_dirent*)arg2, (uint32_t)arg3); 
        case SYS_CREATE:        return sys_create_impl((const char*)arg1);
        default:                return (uint32_t)-1;
   }
}