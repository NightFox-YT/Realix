// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// Реализация функций обнуления.

#include "secure_memory.h"
#include "secure_memory_arch.h"

#include <string.h>

namespace blake3 {

using internal::kCacheLineSize;
using internal::kWordSize;

namespace internal {

// -----------------------------------------------------------------------------
// Генератор псевдослучайных чисел
// -----------------------------------------------------------------------------
static BLAKE3_FORCE_INLINE uint64_t GetRandomPattern() {
  static volatile uint64_t seed = 0;
  if (seed == 0) {
    uint64_t entropy = GetEntropy();
    entropy ^= reinterpret_cast<uint64_t>(&GetRandomPattern);
    #if defined(__x86_64__) && defined(__GNUC__)
      uint64_t tsc;
      __asm__ volatile("rdtsc" : "=A"(tsc));
      entropy ^= tsc;
    #endif
    uint64_t z = (entropy + 0x9E3779B97F4A7C15ULL);
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    z = z ^ (z >> 31);
    seed = z;
  }
  uint64_t x = seed;
  x ^= x >> 12;
  x ^= x << 25;
  x ^= x >> 27;
  seed = x;
  return x * 0x2545F4914F6CDD1DULL;
}

// -----------------------------------------------------------------------------
// TimingDummy
// -----------------------------------------------------------------------------
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
  volatile uint8_t* p = static_cast<volatile uint8_t*>(ptr);
  volatile uint64_t* p64 = reinterpret_cast<volatile uint64_t*>(p);

  const size_t words = len / kWordSize;
  const size_t bytes = len % kWordSize;

  uint64_t pattern1 = 0x0000000000000000ULL;
  uint64_t pattern2 = 0xFFFFFFFFFFFFFFFFULL;

#ifdef BLAKE3_SECURE_ZERO_RANDOM_PATTERNS
  if (config::kUseRandomPatterns) {
    pattern2 = GetRandomPattern();
  }
#endif

  // Проход 1: 0x00
#if defined(BLAKE3_ARCH_X86_64) && defined(BLAKE3_HAVE_MOVNTI)
  for (size_t w = 0; w < words; ++w) {
    __asm__ volatile("movnti %1, (%0)" : : "r"(&p64[w]), "r"(pattern1) : "memory");
  }
#elif defined(BLAKE3_ARCH_ARM64) && defined(BLAKE3_HAVE_STNP)
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
#if defined(BLAKE3_ARCH_X86_64)
  __asm__ volatile("sfence" ::: "memory");
#elif defined(BLAKE3_ARCH_ARM64)
  __asm__ volatile("dsb st" ::: "memory");
#endif
  BLAKE3_COMPILER_BARRIER();

  // Проход 2: pattern2
#if defined(BLAKE3_ARCH_X86_64) && defined(BLAKE3_HAVE_MOVNTI)
  for (size_t w = 0; w < words; ++w) {
    __asm__ volatile("movnti %1, (%0)" : : "r"(&p64[w]), "r"(pattern2) : "memory");
  }
#elif defined(BLAKE3_ARCH_ARM64) && defined(BLAKE3_HAVE_STNP)
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
#if defined(BLAKE3_ARCH_X86_64)
  __asm__ volatile("sfence" ::: "memory");
#elif defined(BLAKE3_ARCH_ARM64)
  __asm__ volatile("dsb st" ::: "memory");
#endif
  BLAKE3_COMPILER_BARRIER();

  // Финальный проход с нулями
  for (size_t w = 0; w < words; ++w) {
    p64[w] = 0;
  }
  for (size_t b = 0; b < bytes; ++b) {
    p[words * kWordSize + b] = 0;
  }
  BLAKE3_MEMORY_BARRIER();
  BLAKE3_COMPILER_BARRIER();

  // Кэш-флаш выполняем только если длина >= кэш-линии
  if (len >= kCacheLineSize) {
#if defined(BLAKE3_ARCH_X86_64)
    #ifdef BLAKE3_HAVE_CLFLUSHOPT
      for (size_t i = 0; i < len; i += kCacheLineSize) {
        __asm__ volatile("clflushopt (%0)" : : "r"(p + i) : "memory");
      }
    #else
      for (size_t i = 0; i < len; i += kCacheLineSize) {
        __asm__ volatile("clflush (%0)" : : "r"(p + i) : "memory");
      }
    #endif
    __asm__ volatile("sfence" ::: "memory");
#elif defined(BLAKE3_ARCH_ARM64)
    for (size_t i = 0; i < len; i += kCacheLineSize) {
      __asm__ volatile("dc cvac, %0" : : "r"(p + i) : "memory");
    }
    __asm__ volatile("dsb sy" ::: "memory");
#endif
  }

  // Очистка регистров
#if defined(BLAKE3_ARCH_X86_64)
  __asm__ volatile(
      "xor %%rax, %%rax\n\t"
      "xor %%rcx, %%rcx\n\t"
      "xor %%rdx, %%rdx\n\t"
      "xor %%rbx, %%rbx\n\t"
      "xor %%rsi, %%rsi\n\t"
      "xor %%rdi, %%rdi\n\t"
      "xor %%r8, %%r8\n\t"
      "xor %%r9, %%r9\n\t"
      "xor %%r10, %%r10\n\t"
      "xor %%r11, %%r11\n\t"
      "xor %%r12, %%r12\n\t"
      "xor %%r13, %%r13\n\t"
      "xor %%r14, %%r14\n\t"
      "xor %%r15, %%r15\n\t"
      ::: "rax", "rbx", "rcx", "rdx", "rsi", "rdi",
           "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
           "memory"
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
#endif

  // Случайная задержка только для больших размеров
#ifdef BLAKE3_SECURE_ZERO_RANDOM_PATTERNS
  if (config::kUseRandomPatterns && len >= 512) {
    volatile size_t delay = GetRandomPattern() & 0x3F;
    while (delay--) {
      BLAKE3_CPU_PAUSE();
      BLAKE3_COMPILER_BARRIER();
    }
  }
#endif
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