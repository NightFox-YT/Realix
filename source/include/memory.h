#ifndef __realix_memory__
#define __realix_memory__

#ifdef __cplusplus
extern "C" {
#endif

#include "../../libc/stdint.h"

#define LARRY_SIZE 4096
#define PCINFO_ADDR 0x4500

typedef struct __attribute__((packed))
{
	uint64_t base_addr;
	uint64_t len;
	uint32_t type; // 1 = свободная память, 2 - занято железом
	uint32_t ext; 
} e820_t;

typedef struct __attribute__((packed)) {
    uint16_t low_memory_kb;     // [PCINFO_ADDR]
    uint8_t  boot_drive_num;    // [PCINFO_ADDR + 2]
    uint16_t mmap_count;        // [PCINFO_ADDR + 3]
    e820_t mmap[0];       		// [PCINFO_ADDR + 5]
} pcinfo_t;

#define PCINFO_PTR ((pcinfo_t*)PCINFO_ADDR)

void init_kernel_page_allocator(void);
void *rlxalloc_page(void);

#ifdef __cplusplus
}
#endif

#endif /*__realix_memory__*/