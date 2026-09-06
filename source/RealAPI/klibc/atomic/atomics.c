#include "atomic.h"

void AtomicBytePush(volatile uint8_t *object, int8_t delta) {
    __atomic_fetch_add(object, delta, __ATOMIC_SEQ_CST);
}
void Atomic16Push(volatile uint16_t *object, int16_t delta) {
    __atomic_fetch_add(object, delta, __ATOMIC_SEQ_CST);
}
void Atomic32Push(volatile uint32_t *object, int32_t delta) {
    __atomic_fetch_add(object, delta, __ATOMIC_SEQ_CST);
}
void AtomicPtrPush(volatile void **object, ptrdiff_t delta) {
    __atomic_fetch_add((uintptr_t *)object, delta * sizeof(void *), __ATOMIC_SEQ_CST);
}

uint8_t AtomicBytePushNewVal(volatile uint8_t *object, int8_t delta) {
    return __atomic_add_fetch(object, delta, __ATOMIC_SEQ_CST);
}
uint16_t Atomic16PushNewVal(volatile uint16_t *object, int16_t delta) {
    return __atomic_add_fetch(object, delta, __ATOMIC_SEQ_CST);
}
uint32_t Atomic32PushNewVal(volatile uint32_t *object, int32_t delta) {
    return __atomic_add_fetch(object, delta, __ATOMIC_SEQ_CST);
}
void *AtomicPtrPushNewVal(volatile void **object, ptrdiff_t delta) {
    return (void *)__atomic_add_fetch((uintptr_t *)object, delta * sizeof(void *), __ATOMIC_SEQ_CST);
}

void AtomicByteAnd(volatile uint8_t *object, uint8_t mask) {
    __atomic_fetch_and(object, mask, __ATOMIC_SEQ_CST);
}
void Atomic16And(volatile uint16_t *object, uint16_t mask) {
    __atomic_fetch_and(object, mask, __ATOMIC_SEQ_CST);
}
void Atomic32And(volatile uint32_t *object, uint32_t mask) {
    __atomic_fetch_and(object, mask, __ATOMIC_SEQ_CST);
}

uint8_t AtomicByteAndNewVal(volatile uint8_t *object, uint8_t mask) {
    return __atomic_and_fetch(object, mask, __ATOMIC_SEQ_CST);
}
uint16_t Atomic16AndNewVal(volatile uint16_t *object, uint16_t mask) {
    return __atomic_and_fetch(object, mask, __ATOMIC_SEQ_CST);
}
uint32_t Atomic32AndNewVal(volatile uint32_t *object, uint32_t mask) {
    return __atomic_and_fetch(object, mask, __ATOMIC_SEQ_CST);
}

void AtomicByteOr(volatile uint8_t *object, uint8_t mask) {
    __atomic_fetch_or(object, mask, __ATOMIC_SEQ_CST);
}
void Atomic16Or(volatile uint16_t *object, uint16_t mask) {
    __atomic_fetch_or(object, mask, __ATOMIC_SEQ_CST);
}
void Atomic32Or(volatile uint32_t *object, uint32_t mask) {
    __atomic_fetch_or(object, mask, __ATOMIC_SEQ_CST);
}

uint8_t AtomicByteOrNewVal(volatile uint8_t *object, uint8_t mask) {
    return __atomic_or_fetch(object, mask, __ATOMIC_SEQ_CST);
}
uint16_t Atomic16OrNewVal(volatile uint16_t *object, uint16_t mask) {
    return __atomic_or_fetch(object, mask, __ATOMIC_SEQ_CST);
}
uint32_t Atomic32OrNewVal(volatile uint32_t *object, uint32_t mask) {
    return __atomic_or_fetch(object, mask, __ATOMIC_SEQ_CST);
}

uint8_t AtomicByteCas(volatile uint8_t *object, uint8_t expected, uint8_t desired) {
    __atomic_compare_exchange_n(object, &expected, desired, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return expected;
}
uint16_t Atomic16Cas(volatile uint16_t *object, uint16_t expected, uint16_t desired) {
    __atomic_compare_exchange_n(object, &expected, desired, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return expected;
}
uint32_t Atomic32Cas(volatile uint32_t *object, uint32_t expected, uint32_t desired) {
    __atomic_compare_exchange_n(object, &expected, desired, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return expected;
}
void *AtomicPtrCas(volatile void **object, void *expected, void *desired) {
    __atomic_compare_exchange_n((uintptr_t *)object, (uintptr_t *)&expected, (uintptr_t)desired, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return expected;
}

uint8_t NonInterlockedAtomicByteCas(volatile uint8_t *object, uint8_t expected, uint8_t desired) {
    __atomic_compare_exchange_n(object, &expected, desired, 0, __ATOMIC_RELAXED, __ATOMIC_RELAXED);
    return expected;
}
uint16_t NonInterlockedAtomic16Cas(volatile uint16_t *object, uint16_t expected, uint16_t desired) {
    __atomic_compare_exchange_n(object, &expected, desired, 0, __ATOMIC_RELAXED, __ATOMIC_RELAXED);
    return expected;
}
uint32_t NonInterlockedAtomic32Cas(volatile uint32_t *object, uint32_t expected, uint32_t desired) {
    __atomic_compare_exchange_n(object, &expected, desired, 0, __ATOMIC_RELAXED, __ATOMIC_RELAXED);
    return expected;
}
void *NonInterlockedAtomicPtrCas(volatile void **object, void *expected, void *desired) {
    __atomic_compare_exchange_n((uintptr_t *)object, (uintptr_t *)&expected, (uintptr_t)desired, 0, __ATOMIC_RELAXED, __ATOMIC_RELAXED);
    return expected;
}

uint8_t AtomicByteSwap(volatile uint8_t *object, uint8_t value) {
    return __atomic_exchange_n(object, value, __ATOMIC_SEQ_CST);
}
uint16_t Atomic16Swap(volatile uint16_t *object, uint16_t value) { 
    return __atomic_exchange_n(object, value, __ATOMIC_SEQ_CST);
}
uint32_t Atomic32Swap(volatile uint32_t *object, uint32_t value) {
    return __atomic_exchange_n(object, value, __ATOMIC_SEQ_CST);
}
void *AtomicPtrSwap(volatile void **object, void *value) {
    return (void *)__atomic_exchange_n((uintptr_t *)object, (uintptr_t)value, __ATOMIC_SEQ_CST);
}

void AtomicByteDecrement(volatile uint8_t *object) {
    __atomic_fetch_sub(object, 1, __ATOMIC_SEQ_CST);
}
void Atomic16Decrement(volatile uint16_t *object) {
    __atomic_fetch_sub(object, 1, __ATOMIC_SEQ_CST);
}
void Atomic32Decrement(volatile uint32_t *object) {
    __atomic_fetch_sub(object, 1, __ATOMIC_SEQ_CST);
}
void AtomicPtrDecrement(volatile void **object) {
    __atomic_fetch_sub((uintptr_t *)object, sizeof(void *), __ATOMIC_SEQ_CST);
}

uint8_t AtomicByteDecrementNewVal(volatile uint8_t *object) {
    return __atomic_sub_fetch(object, 1, __ATOMIC_SEQ_CST);
}
uint16_t Atomic16DecrementNewVal(volatile uint16_t *object) {
    return __atomic_sub_fetch(object, 1, __ATOMIC_SEQ_CST);
}
uint32_t Atomic32DecrementNewVal(volatile uint32_t *object) {
    return __atomic_sub_fetch(object, 1, __ATOMIC_SEQ_CST);
}
void *AtomicPtrDecrementNewVal(volatile void **object) {
    return (void *)__atomic_sub_fetch((uintptr_t *)object, sizeof(void *), __ATOMIC_SEQ_CST);
}

void AtomicByteIncrement(volatile uint8_t *object) {
    __atomic_fetch_add(object, 1, __ATOMIC_SEQ_CST);
}
void Atomic16Increment(volatile uint16_t *object) {
    __atomic_fetch_add(object, 1, __ATOMIC_SEQ_CST);
}
void Atomic32Increment(volatile uint32_t *object) {
    __atomic_fetch_add(object, 1, __ATOMIC_SEQ_CST);
}
void AtomicPtrIncrement(volatile void **object) {
    __atomic_fetch_add((uintptr_t *)object, sizeof(void *), __ATOMIC_SEQ_CST);
}

uint8_t AtomicByteIncrementNewVal(volatile uint8_t *object) {
    return __atomic_add_fetch(object, 1, __ATOMIC_SEQ_CST);
}
uint16_t Atomic16IncrementNewVal(volatile uint16_t *object) {
    return __atomic_add_fetch(object, 1, __ATOMIC_SEQ_CST);
}
uint32_t Atomic32IncrementNewVal(volatile uint32_t *object) {
    return __atomic_add_fetch(object, 1, __ATOMIC_SEQ_CST);
}
void *AtomicPtrIncrementNewVal(volatile void **object) {
    return (void *)__atomic_add_fetch((uintptr_t *)object, sizeof(void *), __ATOMIC_SEQ_CST);
}

void MemBarrierEnter(void) {
    __atomic_thread_fence(__ATOMIC_ACQUIRE);
}
void MemBarrierExit(void) {
    __atomic_thread_fence(__ATOMIC_RELEASE);
}
void MemBarrierProducer(void) {
    __atomic_thread_fence(__ATOMIC_RELEASE);
}
void MemBarrierConsumer(void) {
    __atomic_thread_fence(__ATOMIC_ACQUIRE);
}
void MemBarrierSync(void) {
    __atomic_thread_fence(__ATOMIC_SEQ_CST);
}

#ifdef __HAVE_MEMBARRIER_DATADEPENDENCY_CONSUMER
void MemBarrierDataDependencyConsumer(void) {  }
#endif