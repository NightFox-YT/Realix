// © Realix > exec16 Subsystem Bridge Utility in C (.RLX 32-bit)
// Executes in 32-bit Protected Mode and invokes 16-bit Kernel Subsystem
// ======================================================================

#define SYS_PRINT_STRING 1
#define SYS_PUTCHAR      2
#define SYS_EXIT         3
#define SYS_CLEAR        5
#define SYS_GET_SYSINFO  6
#define SYS_EXEC_16      7

static inline void rlx_print(const char *str) {
    __asm__ __volatile__ (
        "int $0x80"
        :
        : "a"(SYS_PRINT_STRING), "S"(str)
        : "memory"
    );
}

static inline void rlx_exec16(const char *filename) {
    __asm__ __volatile__ (
        "int $0x80"
        :
        : "a"(SYS_EXEC_16), "S"(filename)
        : "memory"
    );
}

static inline void rlx_exit(void) {
    __asm__ __volatile__ (
        "int $0x80"
        :
        : "a"(SYS_EXIT)
        : "memory"
    );
}

int main(void) {
    rlx_print("\n");
    rlx_print("   === Realix exec16 Subsystem Bridge (32-bit C Utility) ===\n");
    rlx_print("   [exec16] Initializing 16-bit Real Mode Kernel Subsystem Bridge...\n");
    rlx_print("   [exec16] Sending SYS_EXEC_16 command via INT 0x80 System Call...\n\n");

    rlx_exec16("rlxfetch16.rlx");

    rlx_print("   [exec16] 16-bit Kernel Subsystem bridge execution completed.\n\n");

    rlx_exit();
    return 0;
}
