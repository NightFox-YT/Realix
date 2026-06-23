#include "../../include/memory.h"
#include "../../libc/klibc/string/string.h"

static uint32_t page_pool_start = 0;
static uint32_t page_pool_end = 0;

void init_kernel_page_allocator(void)
{
	pcinfo_t *info = PCINFO_PTR;

	for (uint16_t i = 0; i < info->mmap_count; i++)
	{
		if (info->mmap[i].type == 1 && info->mmap->base_addr >= 0x100000)
		{
			page_pool_start = (uint32_t)info->mmap[i].base_addr;
			page_pool_end = page_pool_start + (uint32_t)info->mmap[i].len;

			page_pool_start = (page_pool_start + 4095) & ~4095;
			page_pool_end = page_pool_end & ~4095;
			break;
		}
	}

	if (page_pool_start == 0)
	{
		page_pool_start = 0x500000;
		page_pool_end = 0x2000000;
	}
}

void *rlxalloc_page(void)
{
	if (page_pool_start >= page_pool_end) return (void*)0; // если память переполнена
	
	void *page = (void*)page_pool_start;
	page_pool_start += LARRY_SIZE;
	return page;	
}