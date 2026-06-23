#include "../../include/slab.h"
#include "../../include/memory.h"
#include "../../libc/klibc/string/string.h"

#define ALIGN(x, a)     (((x) + (a) - 1) & ~((a) - 1))
#define ALIGN4(x)       ALIGN(x, 4)
#define LARRY_SIZE      4096

/*
 * Компоновка страницы-slab:
 *   [slab_t header | padding | obj0 | obj1 | ... | objN-1]
 * header живёт в начале страницы, объекты сразу за ним.
 */
static uint32_t slab_obj_offset(void)
{
    return ALIGN4(sizeof(slab_t));
}

static uint32_t calc_obj_per_slab(uint32_t obj_size)
{
    uint32_t usable = LARRY_SIZE - slab_obj_offset();
    return usable / obj_size;
}

/* Выделить новый slab и инициализировать freelist */
static slab_t *slab_alloc_new(kmem_cache_t *cache)
{
    void *page = rlxalloc_page();
    if (!page) return (slab_t *)0;

    slab_t *s = (slab_t *)page;
    s->next     = (slab_t *)0;
    s->prev     = (slab_t *)0;
    s->inuse    = 0;
    s->capacity = (uint16_t)cache->obj_per_slab;

    /* каждый свободный объект хранит указатель на следующий */
    uint8_t *base = (uint8_t *)page + slab_obj_offset();
    s->freelist = (slab_obj_t *)0;

    for (int32_t i = (int32_t)cache->obj_per_slab - 1; i >= 0; i--) {
        slab_obj_t *node = (slab_obj_t *)(base + i * cache->obj_size);
        node->next  = s->freelist;
        s->freelist = node;
    }

    return s;
}

static void list_remove(slab_t **head, slab_t *s)
{
    if (s->prev) s->prev->next = s->next;
    else         *head         = s->next;
    if (s->next) s->next->prev = s->prev;
    s->next = s->prev = (slab_t *)0;
}

static void list_push(slab_t **head, slab_t *s)
{
    s->prev = (slab_t *)0;
    s->next = *head;
    if (*head) (*head)->prev = s;
    *head = s;
}

kmem_cache_t *kmem_cache_create(const char *name, uint32_t obj_size)
{
    /* минимальный размер объекта — чтобы влез slab_obj_t */
    if (obj_size < sizeof(slab_obj_t))
        obj_size = sizeof(slab_obj_t);
    obj_size = ALIGN4(obj_size);

    kmem_cache_t *cache = (kmem_cache_t *)rlxalloc_page();
    if (!cache) return (kmem_cache_t *)0;

    cache->name         = name;
    cache->obj_size     = obj_size;
    cache->obj_per_slab = calc_obj_per_slab(obj_size);
    cache->slabs_partial = (slab_t *)0;
    cache->slabs_full    = (slab_t *)0;
    cache->slabs_free    = (slab_t *)0;

    return cache;
}

void *kmem_cache_alloc(kmem_cache_t *cache)
{
    slab_t *s = cache->slabs_partial;

    if (!s) {
        /* берём из свободных или выделяем новый */
        if (cache->slabs_free) {
            s = cache->slabs_free;
            list_remove(&cache->slabs_free, s);
        } else {
            s = slab_alloc_new(cache);
            if (!s) return (void *)0;
        }
        list_push(&cache->slabs_partial, s);
    }

    /* берём объект из freelist */
    slab_obj_t *obj = s->freelist;
    s->freelist = obj->next;
    s->inuse++;

    /* если slab заполнен — перемещаем в full */
    if (s->inuse == s->capacity) {
        list_remove(&cache->slabs_partial, s);
        list_push(&cache->slabs_full, s);
    }

    return (void *)obj;
}

void kmem_cache_free(kmem_cache_t *cache, void *obj)
{
    if (!obj) return;

    /* находим slab по адресу страницы (выравниваие вниз до 4K) */
    slab_t *s = (slab_t *)((uint32_t)obj & ~(LARRY_SIZE - 1));

    bool was_full = (s->inuse == s->capacity);

    slab_obj_t *node = (slab_obj_t *)obj;
    node->next  = s->freelist;
    s->freelist = node;
    s->inuse--;

    if (was_full) {
        list_remove(&cache->slabs_full, s);
        list_push(&cache->slabs_partial, s);
    } else if (s->inuse == 0) {
        list_remove(&cache->slabs_partial, s);
        list_push(&cache->slabs_free, s);
    }
}

void kmem_cache_destroy(kmem_cache_t *cache)
{
    /*Страницы физически не возвращаем,просто зануляем дескриптор чтобы не было dangling use.
     */
    memset(cache, 0, sizeof(kmem_cache_t));
}

/*
 * Кэши для степеней двойки: 16, 32, 64, 128, 256, 512, 1024, 2048.
 * Объекты > 2048 — напрямую через rlxalloc_page.
 */
#define KMALLOC_SIZES   8
static const uint32_t kmalloc_size_table[KMALLOC_SIZES] = {
    16, 32, 64, 128, 256, 512, 1024, 2048
};
static kmem_cache_t *kmalloc_caches[KMALLOC_SIZES];

/*
 * Перед каждым объектом kmalloc кладём маленький header,
 * чтобы kfree знал в какой кэш вернуть память.
 */
typedef struct {
    uint32_t magic;     /* 0xA110CA7E */
    uint32_t cache_idx; /* индекс в kmalloc_caches, 0xFF = большой объект */
} kmalloc_hdr_t;
#define HDR_SIZE        ALIGN4(sizeof(kmalloc_hdr_t))

void kmalloc_init(void)
{
    for (uint32_t i = 0; i < KMALLOC_SIZES; i++) {
        kmalloc_caches[i] = kmem_cache_create("kmalloc", kmalloc_size_table[i]);
    }
}

void *kmalloc(uint32_t size)
{
    uint32_t need = size + HDR_SIZE;

    /* ищем подходящий кэш */
    for (uint32_t i = 0; i < KMALLOC_SIZES; i++) {
        if (need <= kmalloc_size_table[i]) {
            void *raw = kmem_cache_alloc(kmalloc_caches[i]);
            if (!raw) return (void *)0;
            kmalloc_hdr_t *hdr = (kmalloc_hdr_t *)raw;
            hdr->magic     = KMALLOC_MAGIC;
            hdr->cache_idx = i;
            return (uint8_t *)raw + HDR_SIZE;
        }
    }

    /* большой объект — целая страница (или несколько, но пока одна) */
    void *page = rlxalloc_page();
    if (!page) return (void *)0;
    kmalloc_hdr_t *hdr = (kmalloc_hdr_t *)page;
    hdr->magic     = KMALLOC_MAGIC;
    hdr->cache_idx = 0xFF;
    return (uint8_t *)page + HDR_SIZE;
}

void kfree(void *ptr)
{
    if (!ptr) return;
    kmalloc_hdr_t *hdr = (kmalloc_hdr_t *)((uint8_t *)ptr - HDR_SIZE);
    if (hdr->magic != KMALLOC_MAGIC) return;
    hdr->magic = 0; 

    if (hdr->cache_idx == 0xFF) {
        /* большой объект — страницы не возвращаем (bump allocator) */
        return;
    }
    kmem_cache_free(kmalloc_caches[hdr->cache_idx], (void *)hdr);
}