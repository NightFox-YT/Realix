#include "atomic.h"

/*atomic ADD*/
void atomic_add_32(volatile uint32_t *p, int32_t v)      { __atomic_fetch_add(p, v, __ATOMIC_SEQ_CST); }
void atomic_add_int(volatile unsigned int *p, int v)    { __atomic_fetch_add(p, v, __ATOMIC_SEQ_CST); }
void atomic_add_long(volatile unsigned long *p, long v)  { __atomic_fetch_add(p, v, __ATOMIC_SEQ_CST); }
void atomic_add_ptr(volatile void **p, ptrdiff_t v)     { __atomic_fetch_add((uintptr_t *)p, v, __ATOMIC_SEQ_CST); }
void atomic_add_64(volatile uint64_t *p, int64_t v)      { __atomic_fetch_add(p, v, __ATOMIC_SEQ_CST); }

/* new value */
uint32_t atomic_add_32_nv(volatile uint32_t *p, int32_t v)      { return __atomic_add_fetch(p, v, __ATOMIC_SEQ_CST); }
unsigned int atomic_add_int_nv(volatile unsigned int *p, int v)  { return __atomic_add_fetch(p, v, __ATOMIC_SEQ_CST); }
unsigned long atomic_add_long_nv(volatile unsigned long *p, long v){ return __atomic_add_fetch(p, v, __ATOMIC_SEQ_CST); }
void * atomic_add_ptr_nv(volatile void **p, ptrdiff_t v)         { return (void *)__atomic_add_fetch((uintptr_t *)p, v, __ATOMIC_SEQ_CST); }
uint64_t atomic_add_64_nv(volatile uint64_t *p, int64_t v)      { return __atomic_add_fetch(p, v, __ATOMIC_SEQ_CST); }

/*atomic AND*/
void atomic_and_32(volatile uint32_t *p, uint32_t v)      { __atomic_fetch_and(p, v, __ATOMIC_SEQ_CST); }
void atomic_and_uint(volatile unsigned int *p, unsigned int v) { __atomic_fetch_and(p, v, __ATOMIC_SEQ_CST); }
void atomic_and_ulong(volatile unsigned long *p, unsigned long v) { __atomic_fetch_and(p, v, __ATOMIC_SEQ_CST); }
void atomic_and_64(volatile uint64_t *p, uint64_t v)      { __atomic_fetch_and(p, v, __ATOMIC_SEQ_CST); }

uint32_t atomic_and_32_nv(volatile uint32_t *p, uint32_t v)      { return __atomic_and_fetch(p, v, __ATOMIC_SEQ_CST); }
unsigned int atomic_and_uint_nv(volatile unsigned int *p, unsigned int v) { return __atomic_and_fetch(p, v, __ATOMIC_SEQ_CST); }
unsigned long atomic_and_ulong_nv(volatile unsigned long *p, unsigned long v) { return __atomic_and_fetch(p, v, __ATOMIC_SEQ_CST); }
uint64_t atomic_and_64_nv(volatile uint64_t *p, uint64_t v)      { return __atomic_and_fetch(p, v, __ATOMIC_SEQ_CST); }

/*atomic OR*/
void atomic_or_32(volatile uint32_t *p, uint32_t v)       { __atomic_fetch_or(p, v, __ATOMIC_SEQ_CST); }
void atomic_or_uint(volatile unsigned int *p, unsigned int v) { __atomic_fetch_or(p, v, __ATOMIC_SEQ_CST); }
void atomic_or_ulong(volatile unsigned long *p, unsigned long v) { __atomic_fetch_or(p, v, __ATOMIC_SEQ_CST); }
void atomic_or_64(volatile uint64_t *p, uint64_t v)       { __atomic_fetch_or(p, v, __ATOMIC_SEQ_CST); }

uint32_t atomic_or_32_nv(volatile uint32_t *p, uint32_t v)       { return __atomic_or_fetch(p, v, __ATOMIC_SEQ_CST); }
unsigned int atomic_or_uint_nv(volatile unsigned int *p, unsigned int v) { return __atomic_or_fetch(p, v, __ATOMIC_SEQ_CST); }
unsigned long atomic_or_ulong_nv(volatile unsigned long *p, unsigned long v) { return __atomic_or_fetch(p, v, __ATOMIC_SEQ_CST); }
uint64_t atomic_or_64_nv(volatile uint64_t *p, uint64_t v)       { return __atomic_or_fetch(p, v, __ATOMIC_SEQ_CST); }

/*Compare AND Swap*/
uint32_t atomic_cas_32(volatile uint32_t *p, uint32_t old, uint32_t new) 
{
    __atomic_compare_exchange_n(p, &old, new, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return old;
}
unsigned int atomic_cas_uint(volatile unsigned int *p, unsigned int old, unsigned int new) 
{
    __atomic_compare_exchange_n(p, &old, new, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return old;
}
unsigned long atomic_cas_ulong(volatile unsigned long *p, unsigned long old, unsigned long new) 
{
    __atomic_compare_exchange_n(p, &old, new, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return old;
}
void * atomic_cas_ptr(volatile void **p, void *old, void *new) 
{
    __atomic_compare_exchange_n((uintptr_t *)p, (uintptr_t *)&old, (uintptr_t)new, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return old;
}
uint64_t atomic_cas_64(volatile uint64_t *p, uint64_t old, uint64_t new) 
{
    __atomic_compare_exchange_n(p, &old, new, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return old;
}

uint16_t atomic_cas_16(volatile uint16_t *p, uint16_t old, uint16_t new) 
{
    __atomic_compare_exchange_n(p, &old, new, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return old;
}
uint8_t atomic_cas_8(volatile uint8_t *p, uint8_t old, uint8_t new) \
{
    __atomic_compare_exchange_n(p, &old, new, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return old;
}

/* Non-interlocked atomic Compare and Swap*/
uint32_t atomic_cas_32_ni(volatile uint32_t *p, uint32_t o, uint32_t n) { return atomic_cas_32(p, o, n); }
unsigned int atomic_cas_uint_ni(volatile unsigned int *p, unsigned int o, unsigned int n) { return atomic_cas_uint(p, o, n); }
unsigned long atomic_cas_ulong_ni(volatile unsigned long *p, unsigned long o, unsigned long n) { return atomic_cas_ulong(p, o, n); }
void * atomic_cas_ptr_ni(volatile void **p, void *o, void *n) { return atomic_cas_ptr(p, o, n); }
uint64_t atomic_cas_64_ni(volatile uint64_t *p, uint64_t o, uint64_t n) { return atomic_cas_64(p, o, n); }

/*atomic swap and exchange*/
uint32_t atomic_swap_32(volatile uint32_t *p, uint32_t v)      { return __atomic_exchange_n(p, v, __ATOMIC_SEQ_CST); }
unsigned int atomic_swap_uint(volatile unsigned int *p, unsigned int v) { return __atomic_exchange_n(p, v, __ATOMIC_SEQ_CST); }
unsigned long atomic_swap_ulong(volatile unsigned long *p, unsigned long v) { return __atomic_exchange_n(p, v, __ATOMIC_SEQ_CST); }
void * atomic_swap_ptr(volatile void **p, void *v)             { return (void *)__atomic_exchange_n((uintptr_t *)p, (uintptr_t)v, __ATOMIC_SEQ_CST); }
uint64_t atomic_swap_64(volatile uint64_t *p, uint64_t v)      { return __atomic_exchange_n(p, v, __ATOMIC_SEQ_CST); }

/*atomic increment and decrement*/
void atomic_dec_32(volatile uint32_t *p)      { __atomic_fetch_sub(p, 1, __ATOMIC_SEQ_CST); }
void atomic_dec_uint(volatile unsigned int *p){ __atomic_fetch_sub(p, 1, __ATOMIC_SEQ_CST); }
void atomic_dec_ulong(volatile unsigned long *p){ __atomic_fetch_sub(p, 1, __ATOMIC_SEQ_CST); }
void atomic_dec_ptr(volatile void **p)        { __atomic_fetch_sub((uintptr_t *)p, sizeof(void*), __ATOMIC_SEQ_CST); }
void atomic_dec_64(volatile uint64_t *p)      { __atomic_fetch_sub(p, 1, __ATOMIC_SEQ_CST); }

uint32_t atomic_dec_32_nv(volatile uint32_t *p)      { return __atomic_sub_fetch(p, 1, __ATOMIC_SEQ_CST); }
unsigned int atomic_dec_uint_nv(volatile unsigned int *p){ return __atomic_sub_fetch(p, 1, __ATOMIC_SEQ_CST); }
unsigned long atomic_dec_ulong_nv(volatile unsigned long *p){ return __atomic_sub_fetch(p, 1, __ATOMIC_SEQ_CST); }
void * atomic_dec_ptr_nv(volatile void **p)        { return (void *)__atomic_sub_fetch((uintptr_t *)p, sizeof(void*), __ATOMIC_SEQ_CST); }
uint64_t atomic_dec_64_nv(volatile uint64_t *p)      { return __atomic_sub_fetch(p, 1, __ATOMIC_SEQ_CST); }

void atomic_inc_32(volatile uint32_t *p)      { __atomic_fetch_add(p, 1, __ATOMIC_SEQ_CST); }
void atomic_inc_uint(volatile unsigned int *p){ __atomic_fetch_add(p, 1, __ATOMIC_SEQ_CST); }
void atomic_inc_ulong(volatile unsigned long *p){ __atomic_fetch_add(p, 1, __ATOMIC_SEQ_CST); }
void atomic_inc_ptr(volatile void **p)        { __atomic_fetch_add((uintptr_t *)p, sizeof(void*), __ATOMIC_SEQ_CST); }
void atomic_inc_64(volatile uint64_t *p)      { __atomic_fetch_add(p, 1, __ATOMIC_SEQ_CST); }

uint32_t atomic_inc_32_nv(volatile uint32_t *p)      { return __atomic_add_fetch(p, 1, __ATOMIC_SEQ_CST); }
unsigned int atomic_inc_uint_nv(volatile unsigned int *p){ return __atomic_add_fetch(p, 1, __ATOMIC_SEQ_CST); }
unsigned long atomic_inc_ulong_nv(volatile unsigned long *p){ return __atomic_add_fetch(p, 1, __ATOMIC_SEQ_CST); }
void * atomic_inc_ptr_nv(volatile void **p)        { return (void *)__atomic_add_fetch((uintptr_t *)p, sizeof(void*), __ATOMIC_SEQ_CST); }
uint64_t atomic_inc_64_nv(volatile uint64_t *p)      { return __atomic_add_fetch(p, 1, __ATOMIC_SEQ_CST); }

/*memory barriers*/
void membar_enter(void)    { __atomic_thread_fence(__ATOMIC_ACQUIRE); }
void membar_exit(void)     { __atomic_thread_fence(__ATOMIC_RELEASE); }
void membar_producer(void) { __atomic_thread_fence(__ATOMIC_RELEASE); }
void membar_consumer(void) { __atomic_thread_fence(__ATOMIC_ACQUIRE); }
void membar_sync(void)     { __atomic_thread_fence(__ATOMIC_SEQ_CST); }

#ifdef __HAVE_MEMBAR_DATADEP_CONSUMER
void membar_datadep_consumer(void) {  }
#endif