#include "../include/gdt.h"
#include "../libc/klibc/string/string.h"

#define GDT_BASE_ADDRESS 0x00007000
#define TSS_BASE_ADDRESS 0x00007500

static struct gdt_entry * volatile gdt = (struct gdt_entry *)GDT_BASE_ADDRESS;
static struct gdt_ptr gptr;

struct tss_entry {
    uint32_t prev_tss;
    uint32_t esp0;
    uint32_t ss0;
    uint32_t esp1;
    uint32_t ss1;
    uint32_t esp2;
    uint32_t ss2;
    uint32_t cr3;
    uint32_t eip;
    uint32_t eflags;
    uint32_t eax, ecx, edx, ebx;
    uint32_t esp;
    uint32_t ebp;
    uint32_t esi;
    uint32_t edi;
    uint16_t es, reserved0;
    uint16_t cs, reserved1;
    uint16_t ss, reserved2;
    uint16_t ds, reserved3;
    uint16_t fs, reserved4;
    uint16_t gs, reserved5;
    uint16_t ldt, reserved6;
    uint16_t trap;
    uint16_t iomap_base;
} __attribute__((packed)); 

static struct tss_entry *tss = (struct tss_entry *)TSS_BASE_ADDRESS;

void set_gdt_gate(struct gdt_entry *entry, uint32_t base, uint32_t limit, uint8_t access, uint8_t gran)
{
    volatile struct gdt_entry *e = (volatile struct gdt_entry*)entry;
    entry->limit_low   = (uint16_t)(limit & 0xFFFF);
    entry->base_low    = (uint16_t)(base & 0xFFFF);
    entry->base_middle = (uint8_t)((base >> 16) & 0xFF);
    entry->access      = access;
    entry->granularity = (uint8_t)((limit >> 16) & 0x0F) | (gran & 0xF0);
    entry->base_high   = (uint8_t)((base >> 24) & 0xFF);
}

void set_tss_esp0(uint32_t esp0) {tss->esp0 = esp0;}

void init_tss(void)
{
    memset(tss, 0, sizeof(struct tss_entry));
    tss->ss0 = 0x10;
    tss->iomap_base = sizeof(struct tss_entry);

    uint32_t base = (uint32_t)tss;
    uint32_t limit = sizeof(struct tss_entry) - 1;
    
    set_gdt_gate(&gdt[5], base, limit, ACC_PRESENT | ACC_RING0 | ACC_TSS, 0x00);

    __asm__ volatile ("ltr %%ax" : : "a"(0x28));
}

void gdt_init(void)
{
    gptr.limit = (sizeof(struct gdt_entry) * 6) - 1;
    gptr.base = GDT_BASE_ADDRESS;

    memset(gdt, 0, sizeof(struct gdt_entry) * 6);

    set_gdt_gate(&gdt[0], 0, 0, 0, 0);
    set_gdt_gate(&gdt[1], 0, 0xFFFFFFFF, 0x9A, 0xCF); // Kernel Code
    set_gdt_gate(&gdt[2], 0, 0xFFFFFFFF, 0x92, 0xCF); // Kernel Data
    set_gdt_gate(&gdt[3], 0, 0xFFFFFFFF, 0xFA, 0xCF); // User Code
    set_gdt_gate(&gdt[4], 0, 0xFFFFFFFF, 0xF2, 0xCF); // User Data

    __asm__ volatile("lgdt %0" : : "m" (gptr));

    __asm__ volatile (
        "sub $6, %%esp\n\t"          // Выделяем место на стеке под псевдо-gptr
        "movw $47, (%%esp)\n\t"      // Limit: 6 дескрипторов * 8 байт - 1 = 47
        "movl $0x7000, 2(%%esp)\n\t" // Base: жесткий адрес 0x7000
        "lgdt (%%esp)\n\t"           // Загружаем GDT процессору
        "add $6, %%esp\n\t"          // Возвращаем стек назад
        
        // Сразу же прыгаем, пока компилятор не успел ничего вставить между ними
        "ljmp $0x08, $1f\n\t"
        "1:\n\t"
        "mov $0x10, %%ax\n\t"
        "mov %%ax, %%ds\n\t"
        "mov %%ax, %%es\n\t"
        "mov %%ax, %%fs\n\t"
        "mov %%ax, %%gs\n\t"
        "mov %%ax, %%ss\n\t"
        :
        :
        : "eax", "memory"
    );

    init_tss();
}