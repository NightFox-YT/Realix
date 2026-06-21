#ifndef __realix_vmm__
#define __realix_vmm__

#ifdef __cplusplus
extern "C" {
#endif

#include "../../libc/stdint.h"
#include "../../libc/stddef.h"
#include "../../libc/stdbool.h"

/*i386 page table флаги*/
#define PTE_PRESENT   (1 << 0)
#define PTE_WRITE     (1 << 1)
#define PTE_USER      (1 << 2)
#define PTE_ACCESSED  (1 << 5)
#define PTE_DIRTY     (1 << 6)

/*VMA флаги*/
#define VMA_READ      (1 << 0)
#define VMA_WRITE     (1 << 1)
#define VMA_EXEC      (1 << 2)
#define VMA_KERNEL    (1 << 3)  /* регион ядра */
#define VMA_USER      (1 << 4)
/* структуры */

/*Регион виртуального адресного пространства*/
typedef struct vma {
    uint32_t    virt_start;
    uint32_t    virt_end;   /* не включительно */
    uint32_t    flags;      /* VMA_* */
    struct vma *next;
} vma_t;

/* Адресное пространство (одно на процесс / для ядра одно глобальное) */
typedef struct vmm {
    uint32_t   *pgdir;      /* физический адрес page directory (4K-aligned) */
    vma_t      *vmas;       /* список VMA, отсортирован по virt_start */
} vmm_t;

/* Глобальное адресное пространство ядра */
extern vmm_t *kernel_vmm;

/* инициализация */
void vmm_init(void);

/* управление адресными пространствами */
vmm_t   *vmm_create(void);
void     vmm_destroy(vmm_t *vmm);
void     vmm_switch(vmm_t *vmm);   /* загружает pgdir в CR3 */

/* маппинг страниц */

/*
 * vmm_map_page — замапить одну физическую страницу на виртуальный адрес.
 * phys и virt должны быть выровнены по 4K.
 */
bool vmm_map_page(vmm_t *vmm, uint32_t virt, uint32_t phys, uint32_t flags);

/*
 * vmm_unmap_page — убрать маппинг одной страницы.
 */
void vmm_unmap_page(vmm_t *vmm, uint32_t virt);

/*
 * vmm_mmap — выделить регион [virt, virt+size) и замапить физические страницы.
 * size выравнивается вверх по 4K.
 * Если virt == 0 — ядро само выбирает адрес.
 * Возвращает виртуальный адрес начала региона или 0 при ошибке.
 */
uint32_t vmm_mmap(vmm_t *vmm, uint32_t virt, uint32_t size, uint32_t flags);

/*
 * vmm_munmap — освободить регион, убрать маппинг страниц.
 */
void vmm_munmap(vmm_t *vmm, uint32_t virt, uint32_t size);

/*
 * vmm_virt_to_phys — трансляция виртуального адреса в физический.
 * Возвращает 0 если страница не замаплена.
 */
uint32_t vmm_virt_to_phys(vmm_t *vmm, uint32_t virt);

#ifdef __cplusplus
}
#endif

#endif /* __realix_vmm__ */