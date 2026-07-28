/* Memory Mapped Input Output -> MMIO. for RealixOS.
 * Copyright (C) 2026 Alexander Silaev.
 */

#ifndef __R_MMIO__
#define __R_MMIO__

#include <stdint.h>
#include <stddef.h>

static inline uint8_t mmio_read8(uintptr_t addr) {
    return *(volatile uint8_t*)addr;
}

static inline uint16_t mmio_read16(uintptr_t addr) {
    return *(volatile uint16_t*)addr;
}

static inline uint32_t mmio_read32(uintptr_t addr) {
    return *(volatile uint32_t*)addr;
}

static inline uint64_t mmio_read64(uintptr_t addr) {
    return *(volatile uint64_t*)addr;
}

static inline void mmio_write8(uintptr_t addr, uint8_t value) {
    *(volatile uint8_t*)addr = value;
}

static inline void mmio_write16(uintptr_t addr, uint16_t value) {
    *(volatile uint16_t*)addr = value;
}

static inline void mmio_write32(uintptr_t addr, uint32_t value) {
    *(volatile uint32_t*)addr = value;
}

static inline void mmio_write64(uintptr_t addr, uint64_t value) {
    *(volatile uint64_t*)addr = value;
}

#endif