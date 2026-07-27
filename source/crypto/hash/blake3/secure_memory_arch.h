// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// Архитектурно-зависимые макросы, константы и генератор энтропии.

#ifndef BLAKE3_SECURE_MEMORY_ARCH_H_
#define BLAKE3_SECURE_MEMORY_ARCH_H_

#include <stddef.h>
#include <stdint.h>

// -----------------------------------------------------------------------------
// Автоматическое определение архитектуры
// -----------------------------------------------------------------------------

#if defined(__x86_64__) || defined(_M_X64)
  #define BLAKE3_ARCH_X86_64 1
  #define BLAKE3_CACHE_LINE_SIZE 64
  #define BLAKE3_WORD_SIZE 8
  #define BLAKE3_HAVE_MOVNTI 1
  #if !defined(BLAKE3_DISABLE_CLFLUSH) && !defined(BLAKE3_HAVE_CLFLUSH)
    #define BLAKE3_HAVE_CLFLUSH 1
  #endif
  #if !defined(BLAKE3_DISABLE_CLFLUSHOPT) && !defined(BLAKE3_HAVE_CLFLUSHOPT)
    #if defined(__clflushopt__) || (defined(__INTEL_COMPILER) && __INTEL_COMPILER >= 1500)
      #define BLAKE3_HAVE_CLFLUSHOPT 1
    #endif
  #endif
  #if !defined(BLAKE3_DISABLE_RDRAND) && !defined(BLAKE3_HAVE_RDRAND)
    #if defined(__RDRND__) || defined(__x86_64__)
      #define BLAKE3_HAVE_RDRAND 1
    #endif
  #endif
  #if defined(__AVX2__)
    #define BLAKE3_HAVE_AVX2 1
  #endif
#elif defined(__i386__) || defined(_M_IX86)
  #define BLAKE3_ARCH_X86 1
  #define BLAKE3_CACHE_LINE_SIZE 64
  #define BLAKE3_WORD_SIZE 4
  #define BLAKE3_HAVE_MOVNTI 1
  #if !defined(BLAKE3_DISABLE_CLFLUSH) && !defined(BLAKE3_HAVE_CLFLUSH)
    #define BLAKE3_HAVE_CLFLUSH 1
  #endif
  #if !defined(BLAKE3_DISABLE_RDRAND) && !defined(BLAKE3_HAVE_RDRAND)
    #if defined(__RDRND__)
      #define BLAKE3_HAVE_RDRAND 1
    #endif
  #endif
#elif defined(__aarch64__) || defined(_M_ARM64)
  #define BLAKE3_ARCH_ARM64 1
  #define BLAKE3_CACHE_LINE_SIZE 64
  #define BLAKE3_WORD_SIZE 8
  #define BLAKE3_HAVE_STNP 1
  #define BLAKE3_HAVE_DC_CVAC 1
  #define BLAKE3_HAVE_CYCLE_COUNTER 1
  #if !defined(BLAKE3_DISABLE_RNDR) && !defined(BLAKE3_HAVE_RNDR)
    #if defined(__ARM_FEATURE_RNG)
      #define BLAKE3_HAVE_RNDR 1
    #endif
  #endif
  #ifdef __ARM_NEON
    #define BLAKE3_HAVE_NEON 1
  #endif
#elif defined(__arm__) || defined(_M_ARM)
  #define BLAKE3_ARCH_ARM32 1
  #define BLAKE3_CACHE_LINE_SIZE 32
  #define BLAKE3_WORD_SIZE 4
  #define BLAKE3_HAVE_DC_CVAC 1
#elif defined(__riscv)
  #define BLAKE3_ARCH_RISCV 1
  #define BLAKE3_CACHE_LINE_SIZE 64
  #define BLAKE3_WORD_SIZE 8
  #ifdef __riscv_zkr
    #define BLAKE3_HAVE_RISC_V_RND 1
  #endif
#elif defined(__powerpc__) || defined(__ppc__)
  #define BLAKE3_ARCH_PPC 1
  #define BLAKE3_CACHE_LINE_SIZE 128
  #define BLAKE3_WORD_SIZE 8
  #define BLAKE3_HAVE_DCBF 1
  #if !defined(BLAKE3_DISABLE_DARN) && !defined(BLAKE3_HAVE_DARN)
    #define BLAKE3_HAVE_DARN 1
  #endif
#elif defined(__mips__)
  #define BLAKE3_ARCH_MIPS 1
  #define BLAKE3_CACHE_LINE_SIZE 32
  #define BLAKE3_WORD_SIZE 4
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
  #if defined(BLAKE3_ARCH_X86_64) || defined(BLAKE3_ARCH_X86)
    #define BLAKE3_READ_BARRIER() __asm__ volatile("lfence" ::: "memory")
    #define BLAKE3_CPU_PAUSE() __asm__ volatile("pause")
  #elif defined(BLAKE3_ARCH_ARM64) || defined(BLAKE3_ARCH_ARM32)
    #define BLAKE3_READ_BARRIER() __asm__ volatile("isb" ::: "memory")
    #define BLAKE3_CPU_PAUSE() __asm__ volatile("yield")
  #else
    #define BLAKE3_READ_BARRIER() BLAKE3_COMPILER_BARRIER()
    #define BLAKE3_CPU_PAUSE() BLAKE3_COMPILER_BARRIER()
  #endif
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
// Генератор энтропии
// -----------------------------------------------------------------------------

namespace blake3 {
namespace internal {

static constexpr size_t kCacheLineSize = BLAKE3_CACHE_LINE_SIZE;
static constexpr size_t kWordSize = BLAKE3_WORD_SIZE;

// Получение аппаратной случайности
static BLAKE3_FORCE_INLINE uint64_t HardwareRandom() {
  uint64_t val = 0;
#if defined(BLAKE3_HAVE_RDRAND)
  uint32_t lo, hi;
  int ok1, ok2;
  __asm__ volatile("rdrand %0\n\t" "setc %1" : "=r"(lo), "=qm"(ok1) : : "cc");
  __asm__ volatile("rdrand %0\n\t" "setc %1" : "=r"(hi), "=qm"(ok2) : : "cc");
  if (ok1 && ok2) val = ((uint64_t)hi << 32) | lo;
#elif defined(BLAKE3_HAVE_RNDR)
  uint64_t tmp;
  int ok;
  __asm__ volatile("mrs %0, s3_3_c2_c4_0\n\t" "mov %1, #0\n\t" "tst %0, #1\n\t" "csel %1, %1, #1, ne\n\t"
                   : "=r"(tmp), "=r"(ok) : : "cc");
  if (ok) val = tmp;
#elif defined(BLAKE3_HAVE_RISC_V_RND)
  uint64_t tmp;
  int ok;
  __asm__ volatile("pollentropy %0\n\t" "andi %1, %0, 1\n\t" "srli %0, %0, 1\n\t"
                   : "=r"(tmp), "=r"(ok) : : "cc");
  if (ok) val = tmp;
#elif defined(BLAKE3_HAVE_DARN)
  uint64_t tmp;
  int ok;
  __asm__ volatile("darn %0, 0\n\t" "cntlzd %1, %0\n\t" "cmpdi %1, 64\n\t" "crandc %1, %1, %1\n\t"
                   : "=r"(tmp), "=r"(ok) : : "cc");
  if (ok) val = tmp;
#endif

  // Если аппаратный RNG не удался, используем таймер
  if (val == 0) {
#if defined(__x86_64__) || defined(__i386__)
    uint32_t lo, hi;
    __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
    val = ((uint64_t)hi << 32) | lo;
#elif defined(__aarch64__)
    uint64_t cnt;
    __asm__ volatile("mrs %0, cntvct_el0" : "=r"(cnt));
    val = cnt;
#else
    volatile uint64_t stack = 0;
    val = reinterpret_cast<uint64_t>(&stack) ^ (uint64_t)__LINE__;
#endif
  }
  return val;
}

static BLAKE3_FORCE_INLINE uint64_t MixEntropy(uint64_t x) {
  uint64_t z = (x + 0x9E3779B97F4A7C15ULL);
  z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
  z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
  return z ^ (z >> 31);
}

static uint64_t entropy_state = 0;

static BLAKE3_FORCE_INLINE void InitEntropy() {
  if (entropy_state == 0) {
    uint64_t entropy = HardwareRandom();
    entropy ^= reinterpret_cast<uint64_t>(&entropy_state);
    entropy_state = MixEntropy(entropy);
  }
}

static BLAKE3_FORCE_INLINE uint64_t GetRandomPattern() {
  InitEntropy();
  uint64_t x = entropy_state;
  x ^= x >> 12;
  x ^= x << 25;
  x ^= x >> 27;
  entropy_state = x;
  return x * 0x2545F4914F6CDD1DULL;
}

static BLAKE3_FORCE_INLINE void TimingDelay(size_t len) {
  if (len >= 512) return;
  volatile size_t dummy = 32;
  while (dummy--) {
    BLAKE3_CPU_PAUSE();
    BLAKE3_COMPILER_BARRIER();
  }
}

}  // namespace internal
}  // namespace blake3

#endif  // BLAKE3_SECURE_MEMORY_ARCH_H_