// © Realix > RLXFetch 32-bit Live System Information Utility in C (.RLX 32-bit)
// =========================================================================

#define SYS_PRINT_STRING 1
#define SYS_PUTCHAR      2
#define SYS_EXIT         3
#define SYS_CLEAR        5
#define SYS_GET_SYSINFO  6

struct RealixSysInfo {
    char os_name[32];
    char os_version[16];
    char kernel_name[32];
    char cpu_vendor[16];
    unsigned int total_ram_mb;
    unsigned int uptime_sec;
    unsigned int mode;
};

static inline void rlx_print(const char *str) {
    __asm__ __volatile__ (
        "int $0x80"
        :
        : "a"(SYS_PRINT_STRING), "S"(str)
        : "memory"
    );
}

static inline void rlx_putchar(char c, unsigned int color) {
    __asm__ __volatile__ (
        "int $0x80"
        :
        : "a"(SYS_PUTCHAR), "d"((unsigned int)c), "b"(color)
        : "memory"
    );
}

static inline void rlx_clear(void) {
    __asm__ __volatile__ (
        "int $0x80"
        :
        : "a"(SYS_CLEAR)
        : "memory"
    );
}

static inline void rlx_get_sysinfo(struct RealixSysInfo *info) {
    __asm__ __volatile__ (
        "int $0x80"
        :
        : "a"(SYS_GET_SYSINFO), "D"(info)
        : "memory"
    );
}

static void print_uint(unsigned int val) {
    char buf[12];
    int i = 0;
    if (val == 0) {
        rlx_print("0");
        return;
    }
    while (val > 0) {
        buf[i++] = '0' + (val % 10);
        val /= 10;
    }
    for (int j = i - 1; j >= 0; j--) {
        char s[2] = {buf[j], 0};
        rlx_print(s);
    }
}

static void print_blue_r_logo(void) {
    const char *logo[] = {
        "   RRRRRRRRRRRRRRRRR   \n",
        "   RR             RR   \n",
        "   RR             RR   \n",
        "   RRRRRRRRRRRRRRRRR   \n",
        "   RR         RR       \n",
        "   RR          RR      \n",
        "   RR           RR     \n",
        0
    };

    for (int i = 0; logo[i] != 0; i++) {
        const char *p = logo[i];
        while (*p) {
            if (*p == 'R') {
                rlx_putchar('R', 1); // 1 = LightBlue
            } else {
                rlx_putchar(*p, 7); // 7 = LightGray
            }
            p++;
        }
    }
}

int main(void) {
    struct RealixSysInfo sys;
    rlx_get_sysinfo(&sys);

    rlx_clear();

    rlx_print("\n");
    rlx_print("   === RLXFetch 32-bit System Information (C Port) ===\n\n");

    print_blue_r_logo();

    rlx_print("\n");
    rlx_print("   User@realix-system\n");
    rlx_print("   ------------------\n");
    rlx_print("   OS:          "); rlx_print(sys.os_name); rlx_print(" "); rlx_print(sys.os_version); rlx_print("\n");
    rlx_print("   Kernel:      "); rlx_print(sys.kernel_name); rlx_print("\n");
    rlx_print("   CPU Vendor:  "); rlx_print(sys.cpu_vendor); rlx_print("\n");
    rlx_print("   RAM Memory:  "); print_uint(sys.total_ram_mb); rlx_print(" MB\n");
    rlx_print("   Live Uptime: "); print_uint(sys.uptime_sec); rlx_print(" seconds\n");
    rlx_print("   PATH:        /bin;/apps;/\n");
    rlx_print("   Privilege:   Ring 3 (User Space via INT 0x80 System API)\n");
    rlx_print("   Executable:  rlxfetch.rlx (Native C Port)\n\n");

    return 0;
}
