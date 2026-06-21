#ifndef __realix_slab__
#define __realix_slab__

#ifdef __cplusplus
extern "C" {
#endif

#include "../libc/stdint.h"
#include "../libc/stddef.h"
#include "../libc/stdbool.h"
#define KMALLOC_MAGIC   0xA110CA7E
/* Freelist node внутри свободного объекта */
typedef struct slab_obj {
    struct slab_obj *next;
} slab_obj_t;

/* Slab — одна страница памяти */
typedef struct slab {
    struct slab     *next;
    struct slab     *prev;
    slab_obj_t      *freelist;  /* список свободных объектов */
    uint16_t         inuse;     /* сколько объектов занято */
    uint16_t         capacity;  /* всего объектов в слабе */
} slab_t;

typedef struct kmem_cache {
    uint32_t         obj_size;      /* размер одного объекта (выровнен) */
    uint32_t         obj_per_slab;  /* объектов на страницу */
    slab_t          *slabs_partial; /* есть свободные слоты */
    slab_t          *slabs_full;    /* полностью заняты */
    slab_t          *slabs_free;    /* полностью свободны */
    const char      *name;
} kmem_cache_t;

kmem_cache_t *kmem_cache_create(const char *name, uint32_t obj_size);
void         *kmem_cache_alloc(kmem_cache_t *cache);
void          kmem_cache_free(kmem_cache_t *cache, void *obj);
void          kmem_cache_destroy(kmem_cache_t *cache);

void  kmalloc_init(void);
void *kmalloc(uint32_t size);
void  kfree(void *ptr);

#ifdef __cplusplus
}
#endif

#endif /*__realix_slab__*/