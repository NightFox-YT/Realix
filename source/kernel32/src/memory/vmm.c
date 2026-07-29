#include "../../../include/vmm.h"
#include "../../../include/slab.h"
#include "../../../include/memory.h"
#include "../../../RealAPI/klibc/string/string.h"

#define LARRY_SIZE      4096
#define PAGE_MASK       (~(LARRY_SIZE - 1))
#define ALIGN_PAGE_UP(x)    (((x) + LARRY_SIZE - 1) & PAGE_MASK)

/* ── i386 page table helpers ────────────────────────────────── */

/*
 * i386 без PAE:
 *   CR3 → Page Directory (1024 × uint32_t)
 *   PDE[virt >> 22] → Page Table (1024 × uint32_t)
 *   PTE[(virt >> 12) & 0x3FF] → физическая страница
 */

#define PD_INDEX(v)  ((v) >> 22)
#define PT_INDEX(v)  (((v) >> 12) & 0x3FF)

static inline void tlb_flush_page(uint32_t virt)
{
    __asm__ volatile("invlpg (%0)" : : "r"(virt) : "memory");
}

static inline void load_cr3(uint32_t phys_pgdir)
{
    __asm__ volatile("mov %0, %%cr3" : : "r"(phys_pgdir) : "memory");
}

/*
 * Получить (или создать) page table для данного PD-индекса.
 * Возвращает виртуальный адрес page table.
 * Поскольку у нас identity mapping для ядра — virt == phys.
 */
static uint32_t *get_or_create_pt(vmm_t *vmm, uint32_t pd_idx, uint32_t flags)
{
    uint32_t pde = vmm->pgdir[pd_idx];

    if (pde & PTE_PRESENT) {
        /* PT уже есть — вернуть её адрес (identity mapped) */
        return (uint32_t *)(pde & PAGE_MASK);
    }

    /* выделяем новую PT */
    uint32_t *pt = (uint32_t *)rlxalloc_page();
    if (!pt) return (uint32_t *)0;
    memset(pt, 0, LARRY_SIZE);

    vmm->pgdir[pd_idx] = (uint32_t)pt | PTE_PRESENT | PTE_WRITE | (flags & PTE_USER);
    return pt;
}

/* ── vmm_t helpers ──────────────────────────────────────────── */

vmm_t *kernel_vmm = (vmm_t *)0;

//static vmem_cache_t_placeholder; /* slab-кэши для vmm_t и vma_t */
static kmem_cache_t *vmm_cache = (kmem_cache_t *)0;
static kmem_cache_t *vma_cache = (kmem_cache_t *)0;

void vmm_init(void)
{
    kmalloc_init();

    vmm_cache = kmem_cache_create("vmm_t", sizeof(vmm_t));
    vma_cache = kmem_cache_create("vma_t", sizeof(vma_t));

    /* создаём адресное пространство ядра */
    kernel_vmm = vmm_create();
}

vmm_t *vmm_create(void)
{
    vmm_t *vmm = (vmm_t *)kmem_cache_alloc(vmm_cache);
    if (!vmm) return (vmm_t *)0;

    /* выделяем page directory */
    uint32_t *pgdir = (uint32_t *)rlxalloc_page();
    if (!pgdir) {
        kmem_cache_free(vmm_cache, vmm);
        return (vmm_t *)0;
    }
    memset(pgdir, 0, LARRY_SIZE);

    vmm->pgdir = pgdir;
    vmm->vmas  = (vma_t *)0;

    /*
     * Identity-map первые 4MB ядра (0x00000000 – 0x003FFFFF).
     * Это покрывает сам код ядра и уже выделенные страницы.
     * Для реального ядра нужно маппить столько, сколько занято.
     */
    for (uint32_t phys = 0; phys < 0x400000; phys += LARRY_SIZE) {
        vmm_map_page(vmm, phys, phys, PTE_PRESENT | PTE_WRITE);
    }

    return vmm;
}

void vmm_destroy(vmm_t *vmm)
{
    if (!vmm) return;

    /* освобождаем VMA list */
    vma_t *v = vmm->vmas;
    while (v) {
        vma_t *next = v->next;
        kmem_cache_free(vma_cache, v);
        v = next;
    }

    /*
     * Page tables и физические страницы данных не возвращаем —
     * у нас bump allocator без free.
     * В реальном ядре здесь надо итерироваться по PD и освобождать PT.
     */

    kmem_cache_free(vmm_cache, vmm);
}

void vmm_switch(vmm_t *vmm)
{
    load_cr3((uint32_t)vmm->pgdir);
}

/* ── маппинг ────────────────────────────────────────────────── */

bool vmm_map_page(vmm_t *vmm, uint32_t virt, uint32_t phys, uint32_t flags)
{
    virt &= PAGE_MASK;
    phys &= PAGE_MASK;

    uint32_t *pt = get_or_create_pt(vmm, PD_INDEX(virt), flags);
    if (!pt) return false;

    pt[PT_INDEX(virt)] = phys | (flags & 0xFFF) | PTE_PRESENT;
    tlb_flush_page(virt);
    return true;
}

void vmm_unmap_page(vmm_t *vmm, uint32_t virt)
{
    virt &= PAGE_MASK;
    uint32_t pde = vmm->pgdir[PD_INDEX(virt)];
    if (!(pde & PTE_PRESENT)) return;

    uint32_t *pt = (uint32_t *)(pde & PAGE_MASK);
    pt[PT_INDEX(virt)] = 0;
    tlb_flush_page(virt);
}

uint32_t vmm_virt_to_phys(vmm_t *vmm, uint32_t virt)
{
    uint32_t pde = vmm->pgdir[PD_INDEX(virt)];
    if (!(pde & PTE_PRESENT)) return 0;

    uint32_t *pt = (uint32_t *)(pde & PAGE_MASK);
    uint32_t pte = pt[PT_INDEX(virt)];
    if (!(pte & PTE_PRESENT)) return 0;

    return (pte & PAGE_MASK) | (virt & ~PAGE_MASK);
}

/* ── VMA helpers ────────────────────────────────────────────── */

/* Найти свободный виртуальный адрес для региона size байт */
static uint32_t vma_find_free(vmm_t *vmm, uint32_t size)
{
    /* начинаем с 1MB (0x100000) — ниже карта устройств и BIOS */
    uint32_t candidate = 0x100000;
    vma_t *v = vmm->vmas;

    while (v) {
        if (candidate + size <= v->virt_start) break; /* нашли дырку */
        if (v->virt_end > candidate)
            candidate = v->virt_end;
        v = v->next;
    }

    return candidate;
}

/* Вставить VMA в список, отсортированный по virt_start */
static void vma_insert(vmm_t *vmm, vma_t *new_vma)
{
    vma_t **cur = &vmm->vmas;
    while (*cur && (*cur)->virt_start < new_vma->virt_start)
        cur = &(*cur)->next;
    new_vma->next = *cur;
    *cur = new_vma;
}

/* Убрать VMA из списка */
static void vma_remove(vmm_t *vmm, vma_t *target)
{
    vma_t **cur = &vmm->vmas;
    while (*cur && *cur != target)
        cur = &(*cur)->next;
    if (*cur) *cur = target->next;
}

/* ── публичное API VMM ──────────────────────────────────────── */

uint32_t vmm_mmap(vmm_t *vmm, uint32_t virt, uint32_t size, uint32_t flags)
{
    if (!size) return 0;
    size = ALIGN_PAGE_UP(size);

    if (!virt)
        virt = vma_find_free(vmm, size);

    /* создаём VMA */
    vma_t *vma = (vma_t *)kmem_cache_alloc(vma_cache);
    if (!vma) return 0;
    vma->virt_start = virt;
    vma->virt_end   = virt + size;
    vma->flags      = flags;
    vma->next       = (vma_t *)0;
    vma_insert(vmm, vma);

    /* конвертируем VMA_* флаги в PTE_* */
    uint32_t pte_flags = PTE_PRESENT;
    if (flags & VMA_WRITE) pte_flags |= PTE_WRITE;
    if (flags & VMA_USER)  pte_flags |= PTE_USER;

    /* выделяем физические страницы и маппим */
    for (uint32_t offset = 0; offset < size; offset += LARRY_SIZE) {
        void *phys = rlxalloc_page();
        if (!phys) {
            /* откат: анмапить уже замапленное */
            vmm_munmap(vmm, virt, offset);
            return 0;
        }
        if (!vmm_map_page(vmm, virt + offset, (uint32_t)phys, pte_flags)) {
            vmm_munmap(vmm, virt, offset);
            return 0;
        }
    }

    return virt;
}

void vmm_munmap(vmm_t *vmm, uint32_t virt, uint32_t size)
{
    if (!size) return;
    size = ALIGN_PAGE_UP(size);

    /* убираем маппинг страниц */
    for (uint32_t offset = 0; offset < size; offset += LARRY_SIZE)
        vmm_unmap_page(vmm, virt + offset);

    /* удаляем VMA (или обрезаем если частичный munmap) */
    vma_t *v = vmm->vmas;
    while (v) {
        vma_t *next = v->next;
        if (v->virt_start >= virt && v->virt_end <= virt + size) {
            /* VMA целиком внутри удаляемого диапазона */
            vma_remove(vmm, v);
            kmem_cache_free(vma_cache, v);
        }
        v = next;
    }
}