#ifndef __realix_gdt__
#define __realix_gdt__

#include "../libc/stdint.h"

/* the GDT structure */
struct gdt_entry {
  uint16_t limit_low; /* little 16 bit of segment's limit */
  uint16_t base_low; /* little 16 bit of basic address */
  uint8_t base_middle; /* next 8 bit basic address */
  uint8_t access; /* access flags, for Ring3(user) and Ring0(kernel) */
  uint8_t granularity; /* granularity and Big 4 bit of limit */
  uint8_t base_high; /* little 8 bits basic address*/
} __attribute__((packed));

/* pointer's structure for LGDT inst. */
struct gdt_ptr {
  uint16_t limit;
  uint32_t base;
} __attribute__((packed));

#define SEL_NULL  0x00
#define SEL_KCODE 0x08
#define SEL_KDATA 0x10
#define SEL_UCODE 0x18
#define SEL_UDATA 0x20
#define SEL_TSS   0x28

#define ACC_PRESENT 0x80
#define ACC_RING0   0x00
#define ACC_RING3   0x60
#define ACC_CODE    0x1A
#define ACC_DATA    0x12
#define ACC_RW      0x00
#define ACC_TSS     0x09

#define GRAN_4K 0x80
#define GRAN_32BIT 0x40

void set_gdt_gate(struct gdt_entry *entry, uint32_t base, uint32_t limit, uint8_t access, uint8_t gran);
void gdt_init(void);
void init_tss(void);
void set_tss_esp0(uint32_t esp0);

#endif