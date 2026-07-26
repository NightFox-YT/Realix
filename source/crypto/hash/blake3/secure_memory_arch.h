// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// Архитектурно-зависимые макросы, константы и встроенные функции.

#ifndef BLAKE3_SECURE_MEMORY_ARCH_H_
#define BLAKE3_SECURE_MEMORY_ARCH_H_

#include <stddef.h>
#include <stdint.h>

// -----------------------------------------------------------------------------
// Определение архитектуры
// -----------------------------------------------------------------------------

#if defined(__x86_64__) || defined(_M_X64)
  #define BLAKE3_ARCH_X86_64 1
  #define BLAKE3_CACHE_LINE_SIZE 64
  #define BLAKE3_WORD_SIZE 8
  #define BLAKE3_HAVE_MOVNTI 1
  #if defined(__clang__) || defined(__GNUC__)
    #if defined(__clflushopt__) || (defined(__INTEL_COMPILER) && __INTEL_COMPILER >= 1500)
      #define BLAKE3_HAVE_CLFLUSHOPT 1
    #endif
  #endif
  #if defined(__RDRND__) || defined(__x86_64__)
    #define BLAKE3_HAVE_RDRAND 1
  #endif
#elif defined(__aarch64__) || defined(_M_ARM64)
  #define BLAKE3_ARCH_ARM64 1
  #define BLAKE3_CACHE_LINE_SIZE 64
  #define BLAKE3_WORD_SIZE 8
  #define BLAKE3_HAVE_STNP 1
  #define BLAKE3_HAVE_CYCLE_COUNTER 1
#elif defined(__arm__) || defined(_M_ARM)
  #define BLAKE3_ARCH_ARM32 1
  #define BLAKE3_CACHE_LINE_SIZE 32
  #define BLAKE3_WORD_SIZE 4
#elif defined(__riscv)
  #define BLAKE3_ARCH_RISCV 1
  #define BLAKE3_CACHE_LINE_SIZE 64
  #define BLAKE3_WORD_SIZE 8
#elif defined(__powerpc__)
  #define BLAKE3_ARCH_PPC 1
  #define BLAKE3_CACHE_LINE_SIZE 128
  #define BLAKE3_WORD_SIZE 8
#else
  #define BLAKE3_ARCH_GENERIC 1
  #define BLAKE3_CACHE_LINE_SIZE 64
  #define BLAKE3_WORD_SIZE sizeof(void*)
#endif

// -----------------------------------------------------------------------------
// Компиляторные расширения
// -----------------------------------------------------------------------------

#if defined(__GNUC__) || defined(__clang__)
  #define BLAKE3_FORCE_INLINE __attribute__((always_inline)) inline
  #define BLAKE3_NO_INLINE __attribute__((noinline))
  #define BLAKE3_RESTRICT __restrict__
  #define BLAKE3_HOT __attribute__((hot))
  #define BLAKE3_COLD __attribute__((cold))
  #define BLAKE3_ALIGNED(x) __attribute__((aligned(x)))
  #define BLAKE3_LIKELY(x) __builtin_expect(!!(x), 1)
  #define BLAKE3_UNLIKELY(x) __builtin_expect(!!(x), 0)
  #define BLAKE3_COMPILER_BARRIER() __asm__ volatile("" ::: "memory")
  #define BLAKE3_MEMORY_BARRIER() __sync_synchronize()
  #define BLAKE3_READ_BARRIER() __asm__ volatile("lfence" ::: "memory")
  #define BLAKE3_CPU_PAUSE() __asm__ volatile("pause")
  #ifdef BLAKE3_HAVE_RDRAND
    #define BLAKE3_RDRAND(dest) \
      do { unsigned int __tmp; __asm__ volatile("rdrand %0" : "=r"(__tmp) : : "cc"); dest = __tmp; } while(0)
  #endif
#elif defined(_MSC_VER)
  #define BLAKE3_FORCE_INLINE __forceinline
  #define BLAKE3_NO_INLINE __declspec(noinline)
  #define BLAKE3_RESTRICT __restrict
  #define BLAKE3_HOT
  #define BLAKE3_COLD
  #define BLAKE3_ALIGNED(x) __declspec(align(x))
  #define BLAKE3_LIKELY(x) (x)
  #define BLAKE3_UNLIKELY(x) (x)
  #define BLAKE3_COMPILER_BARRIER() _ReadWriteBarrier()
  #define BLAKE3_MEMORY_BARRIER() _mm_mfence()
  #define BLAKE3_READ_BARRIER() _mm_lfence()
  #define BLAKE3_CPU_PAUSE() _mm_pause()
  #ifdef BLAKE3_HAVE_RDRAND
    #include <intrin.h>
    #define BLAKE3_RDRAND(dest) while(!_rdrand32_step(&dest))
  #endif
#else
  #define BLAKE3_FORCE_INLINE inline
  #define BLAKE3_NO_INLINE
  #define BLAKE3_RESTRICT
  #define BLAKE3_HOT
  #define BLAKE3_COLD
  #define BLAKE3_ALIGNED(x)
  #define BLAKE3_LIKELY(x) (x)
  #define BLAKE3_UNLIKELY(x) (x)
  #define BLAKE3_COMPILER_BARRIER()
  #define BLAKE3_MEMORY_BARRIER()
  #define BLAKE3_READ_BARRIER()
  #define BLAKE3_CPU_PAUSE()
#endif

// -----------------------------------------------------------------------------
// Вспомогательные константы
// -----------------------------------------------------------------------------

namespace blake3 {
namespace internal {

constexpr size_t kCacheLineSize = BLAKE3_CACHE_LINE_SIZE;
constexpr size_t kWordSize = BLAKE3_WORD_SIZE;
constexpr size_t kMinAlignment = 16;

// Функция для получения энтропии из аппаратного счётчика или RDRAND.
static BLAKE3_FORCE_INLINE uint64_t GetEntropy() {
  uint64_t entropy = 0;
#if defined(BLAKE3_HAVE_RDRAND)
  // Используем RDRAND для получения случайного числа.
  uint32_t rnd1, rnd2;
  BLAKE3_RDRAND(rnd1);
  BLAKE3_RDRAND(rnd2);
  entropy = ((uint64_t)rnd1 << 32) | rnd2;
#elif defined(BLAKE3_HAVE_CYCLE_COUNTER)
  // ARM64: читаем счётчик циклов.
  uint64_t cnt;
  __asm__ volatile("mrs %0, cntvct_el0" : "=r"(cnt));
  entropy = cnt;
#elif defined(__x86_64__) && defined(__GNUC__)
  // RDTSC как запасной вариант.
  uint32_t lo, hi;
  __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
  entropy = ((uint64_t)hi << 32) | lo;
#else
  volatile uint64_t stack_var = 0;
  entropy = reinterpret_cast<uint64_t>(&stack_var) ^ static_cast<uint64_t>(__LINE__);
#endif
  return entropy;
}

}  // namespace internal
}  // namespace blake3

#endif  // BLAKE3_SECURE_MEMORY_ARCH_H_