// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>

#include "secure_memory.h"
#include "secure_memory_arch.h"

#include <string.h>

namespace blake3 {

using internal::kCacheLineSize;
using internal::kWordSize;
using internal::GetRandomPattern;
using internal::TimingDelay;

namespace internal {

static BLAKE3_FORCE_INLINE void TimingDummy(size_t /*len*/) {
  BLAKE3_COMPILER_BARRIER();
}

// -----------------------------------------------------------------------------
// Fast
// -----------------------------------------------------------------------------
BLAKE3_HOT BLAKE3_FORCE_INLINE
void ZeroMemoryFastImpl(void* BLAKE3_RESTRICT ptr, size_t len) {
  volatile uint8_t* p = static_cast<volatile uint8_t*>(ptr);
  volatile uint64_t* p64 = reinterpret_cast<volatile uint64_t*>(p);

  const size_t words = len / kWordSize;
  for (size_t w = 0; w < words; ++w) {
    p64[w] = 0;
  }
  const size_t bytes = len % kWordSize;
  for (size_t b = 0; b < bytes; ++b) {
    p[words * kWordSize + b] = 0;
  }

  BLAKE3_COMPILER_BARRIER();
}

// -----------------------------------------------------------------------------
// Balanced
// -----------------------------------------------------------------------------
BLAKE3_HOT BLAKE3_FORCE_INLINE
void ZeroMemoryBalancedImpl(void* BLAKE3_RESTRICT ptr, size_t len) {
  volatile uint8_t* p = static_cast<volatile uint8_t*>(ptr);
  volatile uint64_t* p64 = reinterpret_cast<volatile uint64_t*>(p);

  const size_t words = len / kWordSize;
  for (size_t w = 0; w < words; ++w) {
    p64[w] = 0;
  }
  const size_t bytes = len % kWordSize;
  for (size_t b = 0; b < bytes; ++b) {
    p[words * kWordSize + b] = 0;
  }

  BLAKE3_READ_BARRIER();
  BLAKE3_MEMORY_BARRIER();
  BLAKE3_COMPILER_BARRIER();
}

// -----------------------------------------------------------------------------
// Military
// -----------------------------------------------------------------------------
BLAKE3_NO_INLINE
void ZeroMemoryMilitaryImpl(void* ptr, size_t len) {
  TimingDelay(len);

  volatile uint8_t* p = static_cast<volatile uint8_t*>(ptr);
  volatile uint64_t* p64 = reinterpret_cast<volatile uint64_t*>(p);

  const size_t words = len / kWordSize;
  const size_t bytes = len % kWordSize;

  uint64_t pattern1 = 0x0000000000000000ULL;
  uint64_t pattern2 = 0xFFFFFFFFFFFFFFFFULL;

#ifdef BLAKE3_SECURE_ZERO_RANDOM_PATTERNS
  if (config::kUseRandomPattern) {
    pattern2 = GetRandomPattern();
  }
#endif

  // Проход 1: 0x00 (non‑temporal, если доступно)
#if defined(BLAKE3_HAVE_MOVNTI) && (defined(BLAKE3_ARCH_X86_64) || defined(BLAKE3_ARCH_X86))
  for (size_t w = 0; w < words; ++w) {
    __asm__ volatile("movnti %1, (%0)" : : "r"(&p64[w]), "r"(pattern1) : "memory");
  }
#elif defined(BLAKE3_HAVE_STNP) && defined(BLAKE3_ARCH_ARM64)
  for (size_t w = 0; w < words; ++w) {
    __asm__ volatile("stnp %x1, %x1, [%0]" : : "r"(&p64[w]), "r"(pattern1) : "memory");
  }
#else
  for (size_t w = 0; w < words; ++w) {
    p64[w] = pattern1;
  }
#endif
  for (size_t b = 0; b < bytes; ++b) {
    p[words * kWordSize + b] = static_cast<uint8_t>(pattern1);
  }

  // Барьер записи
#if defined(BLAKE3_HAVE_MOVNTI) && (defined(BLAKE3_ARCH_X86_64) || defined(BLAKE3_ARCH_X86))
  __asm__ volatile("sfence" ::: "memory");
#elif defined(BLAKE3_HAVE_STNP) && defined(BLAKE3_ARCH_ARM64)
  __asm__ volatile("dsb st" ::: "memory");
#endif
  BLAKE3_COMPILER_BARRIER();

  // Проход 2: pattern2 (non‑temporal)
#if defined(BLAKE3_HAVE_MOVNTI) && (defined(BLAKE3_ARCH_X86_64) || defined(BLAKE3_ARCH_X86))
  for (size_t w = 0; w < words; ++w) {
    __asm__ volatile("movnti %1, (%0)" : : "r"(&p64[w]), "r"(pattern2) : "memory");
  }
#elif defined(BLAKE3_HAVE_STNP) && defined(BLAKE3_ARCH_ARM64)
  for (size_t w = 0; w < words; ++w) {
    __asm__ volatile("stnp %x1, %x1, [%0]" : : "r"(&p64[w]), "r"(pattern2) : "memory");
  }
#else
  for (size_t w = 0; w < words; ++w) {
    p64[w] = pattern2;
  }
#endif
  for (size_t b = 0; b < bytes; ++b) {
    p[words * kWordSize + b] = static_cast<uint8_t>(pattern2);
  }

#if defined(BLAKE3_HAVE_MOVNTI) && (defined(BLAKE3_ARCH_X86_64) || defined(BLAKE3_ARCH_X86))
  __asm__ volatile("sfence" ::: "memory");
#elif defined(BLAKE3_HAVE_STNP) && defined(BLAKE3_ARCH_ARM64)
  __asm__ volatile("dsb st" ::: "memory");
#endif
  BLAKE3_COMPILER_BARRIER();

  // Финальный проход нулями (обычная запись)
  for (size_t w = 0; w < words; ++w) {
    p64[w] = 0;
  }
  for (size_t b = 0; b < bytes; ++b) {
    p[words * kWordSize + b] = 0;
  }
  BLAKE3_MEMORY_BARRIER();
  BLAKE3_COMPILER_BARRIER();

  // Кэш‑флаш (если включён и длина >= кэш‑линии)
  if (config::kEnableCacheFlush && len >= kCacheLineSize) {
#if defined(BLAKE3_ARCH_X86_64) || defined(BLAKE3_ARCH_X86)
    #ifdef BLAKE3_HAVE_CLFLUSHOPT
      for (size_t i = 0; i < len; i += kCacheLineSize) {
        __asm__ volatile("clflushopt (%0)" : : "r"(p + i) : "memory");
      }
    #elif defined(BLAKE3_HAVE_CLFLUSH)
      for (size_t i = 0; i < len; i += kCacheLineSize) {
        __asm__ volatile("clflush (%0)" : : "r"(p + i) : "memory");
      }
    #endif
    __asm__ volatile("sfence" ::: "memory");
#elif defined(BLAKE3_ARCH_ARM64) && defined(BLAKE3_HAVE_DC_CVAC)
    for (size_t i = 0; i < len; i += kCacheLineSize) {
      __asm__ volatile("dc cvac, %0" : : "r"(p + i) : "memory");
    }
    __asm__ volatile("dsb sy" ::: "memory");
#elif defined(BLAKE3_ARCH_ARM32) && defined(BLAKE3_HAVE_DC_CVAC)
    for (size_t i = 0; i < len; i += kCacheLineSize) {
      __asm__ volatile("mcr p15, 0, %0, c7, c14, 1" : : "r"(p + i) : "memory");
    }
    __asm__ volatile("dsb" ::: "memory");
#elif defined(BLAKE3_ARCH_PPC) && defined(BLAKE3_HAVE_DCBF)
    for (size_t i = 0; i < len; i += kCacheLineSize) {
      __asm__ volatile("dcbf %y0" : : "Z"(*(char*)(p + i)) : "memory");
    }
    __asm__ volatile("sync" ::: "memory");
#elif defined(BLAKE3_ARCH_RISCV)
    BLAKE3_MEMORY_BARRIER();
#endif
  }

  // Очистка регистров (если включена)
  if (config::kClearRegisters) {
#if defined(BLAKE3_ARCH_X86_64)
    __asm__ volatile(
        "xor %%eax, %%eax\n\t"
        "xor %%ebx, %%ebx\n\t"
        "xor %%ecx, %%ecx\n\t"
        "xor %%edx, %%edx\n\t"
        "xor %%esi, %%esi\n\t"
        "xor %%edi, %%edi\n\t"
        "xor %%r8d, %%r8d\n\t"
        "xor %%r9d, %%r9d\n\t"
        "xor %%r10d, %%r10d\n\t"
        "xor %%r11d, %%r11d\n\t"
        "xor %%r12d, %%r12d\n\t"
        "xor %%r13d, %%r13d\n\t"
        "xor %%r14d, %%r14d\n\t"
        "xor %%r15d, %%r15d\n\t"
        ::: "eax", "ebx", "ecx", "edx", "esi", "edi",
             "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
             "memory"
    );
#elif defined(BLAKE3_ARCH_X86)
    __asm__ volatile(
        "xor %%eax, %%eax\n\t"
        "xor %%ebx, %%ebx\n\t"
        "xor %%ecx, %%ecx\n\t"
        "xor %%edx, %%edx\n\t"
        "xor %%esi, %%esi\n\t"
        "xor %%edi, %%edi\n\t"
        ::: "eax", "ebx", "ecx", "edx", "esi", "edi", "memory"
    );
#elif defined(BLAKE3_ARCH_ARM64)
    __asm__ volatile(
        "mov x0, #0\n\t"
        "mov x1, #0\n\t"
        "mov x2, #0\n\t"
        "mov x3, #0\n\t"
        "mov x4, #0\n\t"
        "mov x5, #0\n\t"
        "mov x6, #0\n\t"
        "mov x7, #0\n\t"
        "mov x8, #0\n\t"
        "mov x9, #0\n\t"
        "mov x10, #0\n\t"
        "mov x11, #0\n\t"
        "mov x12, #0\n\t"
        "mov x13, #0\n\t"
        "mov x14, #0\n\t"
        "mov x15, #0\n\t"
        "mov x16, #0\n\t"
        "mov x17, #0\n\t"
        "mov x18, #0\n\t"
        "mov x19, #0\n\t"
        "mov x20, #0\n\t"
        "mov x21, #0\n\t"
        "mov x22, #0\n\t"
        "mov x23, #0\n\t"
        "mov x24, #0\n\t"
        "mov x25, #0\n\t"
        "mov x26, #0\n\t"
        "mov x27, #0\n\t"
        "mov x28, #0\n\t"
        "mov x29, #0\n\t"
        "mov x30, #0\n\t"
        ::: "x0","x1","x2","x3","x4","x5","x6","x7","x8","x9",
             "x10","x11","x12","x13","x14","x15","x16","x17","x18",
             "x19","x20","x21","x22","x23","x24","x25","x26","x27",
             "x28","x29","x30","memory"
    );
#elif defined(BLAKE3_ARCH_ARM32)
    __asm__ volatile(
        "mov r0, #0\n\t"
        "mov r1, #0\n\t"
        "mov r2, #0\n\t"
        "mov r3, #0\n\t"
        "mov r4, #0\n\t"
        "mov r5, #0\n\t"
        "mov r6, #0\n\t"
        "mov r7, #0\n\t"
        "mov r8, #0\n\t"
        "mov r9, #0\n\t"
        "mov r10, #0\n\t"
        "mov r11, #0\n\t"
        "mov r12, #0\n\t"
        ::: "r0","r1","r2","r3","r4","r5","r6","r7","r8","r9","r10","r11","r12","memory"
    );
#elif defined(BLAKE3_ARCH_RISCV)
    __asm__ volatile(
        "li x1, 0\n\t"
        "li x2, 0\n\t"
        "li x3, 0\n\t"
        "li x4, 0\n\t"
        "li x5, 0\n\t"
        "li x6, 0\n\t"
        "li x7, 0\n\t"
        "li x8, 0\n\t"
        "li x9, 0\n\t"
        "li x10, 0\n\t"
        "li x11, 0\n\t"
        "li x12, 0\n\t"
        "li x13, 0\n\t"
        "li x14, 0\n\t"
        "li x15, 0\n\t"
        "li x16, 0\n\t"
        "li x17, 0\n\t"
        "li x18, 0\n\t"
        "li x19, 0\n\t"
        "li x20, 0\n\t"
        "li x21, 0\n\t"
        "li x22, 0\n\t"
        "li x23, 0\n\t"
        "li x24, 0\n\t"
        "li x25, 0\n\t"
        "li x26, 0\n\t"
        "li x27, 0\n\t"
        "li x28, 0\n\t"
        "li x29, 0\n\t"
        "li x30, 0\n\t"
        "li x31, 0\n\t"
        ::: "x1","x2","x3","x4","x5","x6","x7","x8","x9",
             "x10","x11","x12","x13","x14","x15","x16","x17","x18",
             "x19","x20","x21","x22","x23","x24","x25","x26","x27",
             "x28","x29","x30","x31","memory"
    );
#endif
  }

  // Случайная задержка для больших размеров
#ifdef BLAKE3_SECURE_ZERO_RANDOM_PATTERNS
  if (config::kUseRandomPattern && len >= 512) {
    volatile size_t delay = GetRandomPattern() & 0x3F;
    while (delay--) {
      BLAKE3_CPU_PAUSE();
      BLAKE3_COMPILER_BARRIER();
    }
  }
#endif
}

// -----------------------------------------------------------------------------
// Universal
// -----------------------------------------------------------------------------
BLAKE3_NO_INLINE
void ZeroMemoryUniversalImpl(void* ptr, size_t len) {
  TimingDelay(len);

  if (BLAKE3_UNLIKELY(len == 0 || ptr == nullptr)) return;

  volatile uint8_t* p = static_cast<volatile uint8_t*>(ptr);
  volatile uint64_t* p64 = reinterpret_cast<volatile uint64_t*>(p);

  const size_t words = len / kWordSize;
  const size_t bytes = len % kWordSize;

  BLAKE3_READ_BARRIER();
  BLAKE3_COMPILER_BARRIER();

  uint64_t pattern1 = 0x0000000000000000ULL;
  uint64_t pattern2 = 0xFFFFFFFFFFFFFFFFULL;
  uint64_t pattern3 = config::kUseRandomPattern ? GetRandomPattern() : 0xAAAAAAAAAAAAAAAAULL;

  const int kPasses = config::kZeroPasses;

  for (int pass = 0; pass < kPasses; ++pass) {
    uint64_t pattern = (pass == 0) ? pattern1 : ((pass == 1) ? pattern2 : pattern3);

#if defined(BLAKE3_HAVE_MOVNTI) && (defined(BLAKE3_ARCH_X86_64) || defined(BLAKE3_ARCH_X86))
    for (size_t w = 0; w < words; ++w) {
      __asm__ volatile("movnti %1, (%0)" : : "r"(&p64[w]), "r"(pattern) : "memory");
    }
#elif defined(BLAKE3_HAVE_STNP) && defined(BLAKE3_ARCH_ARM64)
    for (size_t w = 0; w < words; ++w) {
      __asm__ volatile("stnp %x1, %x1, [%0]" : : "r"(&p64[w]), "r"(pattern) : "memory");
    }
#else
    for (size_t w = 0; w < words; ++w) {
      p64[w] = pattern;
    }
#endif

    for (size_t b = 0; b < bytes; ++b) {
      p[words * kWordSize + b] = static_cast<uint8_t>(pattern);
    }

#if defined(BLAKE3_HAVE_MOVNTI) && (defined(BLAKE3_ARCH_X86_64) || defined(BLAKE3_ARCH_X86))
    __asm__ volatile("sfence" ::: "memory");
#elif defined(BLAKE3_HAVE_STNP) && defined(BLAKE3_ARCH_ARM64)
    __asm__ volatile("dsb st" ::: "memory");
#endif
    BLAKE3_COMPILER_BARRIER();
  }

  // Финальный проход нулями
  for (size_t w = 0; w < words; ++w) {
    p64[w] = 0;
  }
  for (size_t b = 0; b < bytes; ++b) {
    p[words * kWordSize + b] = 0;
  }
  BLAKE3_MEMORY_BARRIER();
  BLAKE3_COMPILER_BARRIER();

  // Кэш‑флаш
  if (config::kEnableCacheFlush && len >= kCacheLineSize) {
#if defined(BLAKE3_ARCH_X86_64) || defined(BLAKE3_ARCH_X86)
    #ifdef BLAKE3_HAVE_CLFLUSHOPT
      for (size_t i = 0; i < len; i += kCacheLineSize) {
        __asm__ volatile("clflushopt (%0)" : : "r"(p + i) : "memory");
      }
    #elif defined(BLAKE3_HAVE_CLFLUSH)
      for (size_t i = 0; i < len; i += kCacheLineSize) {
        __asm__ volatile("clflush (%0)" : : "r"(p + i) : "memory");
      }
    #endif
    __asm__ volatile("sfence" ::: "memory");
#elif defined(BLAKE3_ARCH_ARM64) && defined(BLAKE3_HAVE_DC_CVAC)
    for (size_t i = 0; i < len; i += kCacheLineSize) {
      __asm__ volatile("dc cvac, %0" : : "r"(p + i) : "memory");
    }
    __asm__ volatile("dsb sy" ::: "memory");
#elif defined(BLAKE3_ARCH_ARM32) && defined(BLAKE3_HAVE_DC_CVAC)
    for (size_t i = 0; i < len; i += kCacheLineSize) {
      __asm__ volatile("mcr p15, 0, %0, c7, c14, 1" : : "r"(p + i) : "memory");
    }
    __asm__ volatile("dsb" ::: "memory");
#elif defined(BLAKE3_ARCH_PPC) && defined(BLAKE3_HAVE_DCBF)
    for (size_t i = 0; i < len; i += kCacheLineSize) {
      __asm__ volatile("dcbf %y0" : : "Z"(*(char*)(p + i)) : "memory");
    }
    __asm__ volatile("sync" ::: "memory");
#elif defined(BLAKE3_ARCH_RISCV)
    BLAKE3_MEMORY_BARRIER();
#endif
  }

  // Очистка регистров
  if (config::kClearRegisters) {
#if defined(BLAKE3_ARCH_X86_64)
    __asm__ volatile(
        "xor %%eax, %%eax\n\t"
        "xor %%ebx, %%ebx\n\t"
        "xor %%ecx, %%ecx\n\t"
        "xor %%edx, %%edx\n\t"
        "xor %%esi, %%esi\n\t"
        "xor %%edi, %%edi\n\t"
        "xor %%r8d, %%r8d\n\t"
        "xor %%r9d, %%r9d\n\t"
        "xor %%r10d, %%r10d\n\t"
        "xor %%r11d, %%r11d\n\t"
        "xor %%r12d, %%r12d\n\t"
        "xor %%r13d, %%r13d\n\t"
        "xor %%r14d, %%r14d\n\t"
        "xor %%r15d, %%r15d\n\t"
        ::: "eax", "ebx", "ecx", "edx", "esi", "edi",
             "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
             "memory"
    );
#elif defined(BLAKE3_ARCH_X86)
    __asm__ volatile(
        "xor %%eax, %%eax\n\t"
        "xor %%ebx, %%ebx\n\t"
        "xor %%ecx, %%ecx\n\t"
        "xor %%edx, %%edx\n\t"
        "xor %%esi, %%esi\n\t"
        "xor %%edi, %%edi\n\t"
        ::: "eax", "ebx", "ecx", "edx", "esi", "edi", "memory"
    );
#elif defined(BLAKE3_ARCH_ARM64)
    __asm__ volatile(
        "mov x0, #0\n\t"
        "mov x1, #0\n\t"
        "mov x2, #0\n\t"
        "mov x3, #0\n\t"
        "mov x4, #0\n\t"
        "mov x5, #0\n\t"
        "mov x6, #0\n\t"
        "mov x7, #0\n\t"
        "mov x8, #0\n\t"
        "mov x9, #0\n\t"
        "mov x10, #0\n\t"
        "mov x11, #0\n\t"
        "mov x12, #0\n\t"
        "mov x13, #0\n\t"
        "mov x14, #0\n\t"
        "mov x15, #0\n\t"
        "mov x16, #0\n\t"
        "mov x17, #0\n\t"
        "mov x18, #0\n\t"
        "mov x19, #0\n\t"
        "mov x20, #0\n\t"
        "mov x21, #0\n\t"
        "mov x22, #0\n\t"
        "mov x23, #0\n\t"
        "mov x24, #0\n\t"
        "mov x25, #0\n\t"
        "mov x26, #0\n\t"
        "mov x27, #0\n\t"
        "mov x28, #0\n\t"
        "mov x29, #0\n\t"
        "mov x30, #0\n\t"
        ::: "x0","x1","x2","x3","x4","x5","x6","x7","x8","x9",
             "x10","x11","x12","x13","x14","x15","x16","x17","x18",
             "x19","x20","x21","x22","x23","x24","x25","x26","x27",
             "x28","x29","x30","memory"
    );
#elif defined(BLAKE3_ARCH_ARM32)
    __asm__ volatile(
        "mov r0, #0\n\t"
        "mov r1, #0\n\t"
        "mov r2, #0\n\t"
        "mov r3, #0\n\t"
        "mov r4, #0\n\t"
        "mov r5, #0\n\t"
        "mov r6, #0\n\t"
        "mov r7, #0\n\t"
        "mov r8, #0\n\t"
        "mov r9, #0\n\t"
        "mov r10, #0\n\t"
        "mov r11, #0\n\t"
        "mov r12, #0\n\t"
        ::: "r0","r1","r2","r3","r4","r5","r6","r7","r8","r9","r10","r11","r12","memory"
    );
#elif defined(BLAKE3_ARCH_RISCV)
    __asm__ volatile(
        "li x1, 0\n\t"
        "li x2, 0\n\t"
        "li x3, 0\n\t"
        "li x4, 0\n\t"
        "li x5, 0\n\t"
        "li x6, 0\n\t"
        "li x7, 0\n\t"
        "li x8, 0\n\t"
        "li x9, 0\n\t"
        "li x10, 0\n\t"
        "li x11, 0\n\t"
        "li x12, 0\n\t"
        "li x13, 0\n\t"
        "li x14, 0\n\t"
        "li x15, 0\n\t"
        "li x16, 0\n\t"
        "li x17, 0\n\t"
        "li x18, 0\n\t"
        "li x19, 0\n\t"
        "li x20, 0\n\t"
        "li x21, 0\n\t"
        "li x22, 0\n\t"
        "li x23, 0\n\t"
        "li x24, 0\n\t"
        "li x25, 0\n\t"
        "li x26, 0\n\t"
        "li x27, 0\n\t"
        "li x28, 0\n\t"
        "li x29, 0\n\t"
        "li x30, 0\n\t"
        "li x31, 0\n\t"
        ::: "x1","x2","x3","x4","x5","x6","x7","x8","x9",
             "x10","x11","x12","x13","x14","x15","x16","x17","x18",
             "x19","x20","x21","x22","x23","x24","x25","x26","x27",
             "x28","x29","x30","x31","memory"
    );
#endif
  }

  // Случайная задержка
  if (config::kEnableRandomDelay && len >= 512) {
    volatile size_t delay = GetRandomPattern() & 0x3F;
    while (delay--) {
      BLAKE3_CPU_PAUSE();
      BLAKE3_COMPILER_BARRIER();
    }
  }

  BLAKE3_MEMORY_BARRIER();
  BLAKE3_COMPILER_BARRIER();
}

}  // namespace internal

// -----------------------------------------------------------------------------
// Публичные функции
// -----------------------------------------------------------------------------

void SecureZeroMemoryFast(void* ptr, size_t len) {
  internal::TimingDummy(len);
  internal::ZeroMemoryFastImpl(ptr, len);
}

void SecureZeroMemoryBalanced(void* ptr, size_t len) {
  internal::TimingDummy(len);
  internal::ZeroMemoryBalancedImpl(ptr, len);
}

void SecureZeroMemoryMilitary(void* ptr, size_t len) {
  internal::TimingDummy(len);
  internal::ZeroMemoryMilitaryImpl(ptr, len);
}

void SecureZeroMemoryUniversal(void* ptr, size_t len) {
  internal::TimingDummy(len);
  internal::ZeroMemoryUniversalImpl(ptr, len);
}

// -----------------------------------------------------------------------------
// Отладочные функции
// -----------------------------------------------------------------------------

#ifdef DEBUG_SECURE_MEMORY
static int last_error_code = 0;
static const char* last_error_msg = nullptr;

int VerifyZeroMemory(const void* ptr, size_t len) {
  if (!ptr) {
    last_error_code = -1;
    last_error_msg = "Null pointer";
    return -1;
  }
  const volatile uint8_t* p = static_cast<const volatile uint8_t*>(ptr);
  for (size_t i = 0; i < len; ++i) {
    if (p[i] != 0) {
      last_error_code = -2;
      last_error_msg = "Non-zero byte found";
      return -1;
    }
  }
  last_error_code = 0;
  last_error_msg = "OK";
  return 0;
}

const char* GetSecureMemoryErrorString() {
  return last_error_msg ? last_error_msg : "No error";
}
#endif

}  // namespace blake3