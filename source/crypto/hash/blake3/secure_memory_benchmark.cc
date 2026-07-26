// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// Многофакторный бенчмарк для оценки производительности, константности,
// устойчивости к переупорядочиванию и другим атакам.

#include <algorithm>
#include <array>
#include <atomic>
#include <chrono>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <random>
#include <thread>
#include <vector>

#include "secure_memory.h"
#include "secure_memory_arch.h"

#if defined(__x86_64__)
#include <x86intrin.h>  // для rdtsc
#endif

using Clock = std::chrono::high_resolution_clock;
using Duration = std::chrono::duration<double, std::micro>;

// -----------------------------------------------------------------------------
// Вспомогательные функции
// -----------------------------------------------------------------------------

template <typename T>
static void WarmupCache(T* ptr, size_t len) {
  std::mt19937_64 rng(0xDEADBEEFCAFEBABEULL);
  for (size_t i = 0; i < len; ++i) {
    ptr[i] = static_cast<T>(rng());
  }
}

static inline uint64_t ReadTSC() {
#if defined(__x86_64__) && defined(__GNUC__)
  unsigned int lo, hi;
  __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
  return ((uint64_t)hi << 32) | lo;
#else
  return 0;
#endif
}

static Duration MeasureTime(void (*zero_func)(void*, size_t),
                            void* ptr, size_t len,
                            int iterations = 10000) {
  // Определяем число итераций в зависимости от размера
  if (len <= 1024) {
    iterations = 500000;  // для малых размеров увеличиваем
  } else if (len <= 8192) {
    iterations = 100000;
  } else {
    iterations = 10000;
  }

  WarmupCache(static_cast<uint8_t*>(ptr), len);

  // Прогрев
  zero_func(ptr, len);

  auto start = Clock::now();
  for (int i = 0; i < iterations; ++i) {
    zero_func(ptr, len);
  }
  auto end = Clock::now();
  return Duration(end - start) / iterations;
}

static bool VerifyZero(void* ptr, size_t len) {
  const volatile uint8_t* p = static_cast<const volatile uint8_t*>(ptr);
  for (size_t i = 0; i < len; ++i) {
    if (p[i] != 0) return false;
  }
  return true;
}

// -----------------------------------------------------------------------------
// Тесты производительности
// -----------------------------------------------------------------------------

static void BenchmarkSize(void (*zero_func)(void*, size_t),
                          const char* name,
                          size_t len,
                          int iterations = 10000) {
  const size_t alignment = 64;
  std::vector<uint8_t> buffer(len + alignment - 1);
  uint8_t* ptr = reinterpret_cast<uint8_t*>(
      (reinterpret_cast<uintptr_t>(buffer.data()) + alignment - 1) &
      ~(alignment - 1));

  std::mt19937_64 rng(0x123456789ABCDEFULL);
  for (size_t i = 0; i < len; ++i) ptr[i] = static_cast<uint8_t>(rng());

  Duration avg = MeasureTime(zero_func, ptr, len, iterations);
  bool ok = VerifyZero(ptr, len);

  std::cout << std::setw(20) << name
            << " | size = " << std::setw(10) << len
            << " | avg time = " << std::fixed << std::setprecision(3)
            << avg.count() << " us"
            << " | " << (ok ? "OK" : "FAIL") << '\n';
}

static void BenchmarkMemset(size_t len, int iterations = 10000) {
  const size_t alignment = 64;
  std::vector<uint8_t> buffer(len + alignment - 1);
  uint8_t* ptr = reinterpret_cast<uint8_t*>(
      (reinterpret_cast<uintptr_t>(buffer.data()) + alignment - 1) &
      ~(alignment - 1));

  std::mt19937_64 rng(0xDEADBEEF);
  for (size_t i = 0; i < len; ++i) ptr[i] = static_cast<uint8_t>(rng());

  auto start = Clock::now();
  for (int i = 0; i < iterations; ++i) {
    std::memset(ptr, 0, len);
  }
  auto end = Clock::now();
  Duration avg = Duration(end - start) / iterations;
  bool ok = VerifyZero(ptr, len);

  std::cout << std::setw(20) << "memset"
            << " | size = " << std::setw(10) << len
            << " | avg time = " << std::fixed << std::setprecision(3)
            << avg.count() << " us"
            << " | " << (ok ? "OK" : "FAIL") << '\n';
}

// -----------------------------------------------------------------------------
// Тест константности времени (с использованием rdtsc)
// -----------------------------------------------------------------------------

static void TestConstantTimeRDTSC(void (*zero_func)(void*, size_t),
                                  const char* name,
                                  size_t len,
                                  int iterations = 50000) {
  const size_t alignment = 64;
  std::vector<uint8_t> buffers[4];
  uint8_t* ptrs[4];
  for (int i = 0; i < 4; ++i) {
    buffers[i].resize(len + alignment - 1);
    ptrs[i] = reinterpret_cast<uint8_t*>(
        (reinterpret_cast<uintptr_t>(buffers[i].data()) + alignment - 1) &
        ~(alignment - 1));
  }

  std::mt19937_64 rng(0xCAFEBABE);
  for (size_t i = 0; i < len; ++i) {
    ptrs[0][i] = static_cast<uint8_t>(rng());
    ptrs[1][i] = 0x55;
    ptrs[2][i] = 0xAA;
    ptrs[3][i] = 0xFF;
  }

  double times[4];
  for (int i = 0; i < 4; ++i) {
    uint64_t start = ReadTSC();
    for (int it = 0; it < iterations; ++it) {
      zero_func(ptrs[i], len);
    }
    uint64_t end = ReadTSC();
    times[i] = static_cast<double>(end - start) / iterations;
  }

  double min_t = *std::min_element(times, times+4);
  double max_t = *std::max_element(times, times+4);
  double avg_t = (times[0]+times[1]+times[2]+times[3]) / 4.0;
  double spread = (max_t - min_t) / avg_t * 100.0;

  std::cout << std::setw(20) << name
            << " | len=" << len
            << " | min=" << std::fixed << std::setprecision(1) << min_t << " cycles"
            << " | max=" << max_t << " cycles"
            << " | spread=" << std::setprecision(2) << spread << "%"
            << (spread < 5.0 ? " (good)" : " (WARNING)")
            << '\n';
}

// -----------------------------------------------------------------------------
// Тест на переупорядочивание (проверка барьеров)
// -----------------------------------------------------------------------------

static void TestMemoryBarrier() {
  std::cout << "\n--- Memory Barrier Test (ordering) ---\n";

  std::atomic<int> flag1{0}, flag2{0};
  volatile int data1 = 0, data2 = 0;

  auto writer = [&]() {
    data1 = 42;
    // Барьер должен гарантировать, что data2 запишется после data1.
    BLAKE3_MEMORY_BARRIER();
    data2 = 43;
    flag1.store(1, std::memory_order_release);
  };

  auto reader = [&]() {
    while (flag1.load(std::memory_order_acquire) == 0) {}
    int d1 = data1;
    int d2 = data2;
    if (d1 == 42 && d2 == 43) {
      flag2.store(1, std::memory_order_release);
    } else {
      flag2.store(2, std::memory_order_release);
    }
  };

  std::thread t1(writer);
  std::thread t2(reader);
  t1.join();
  t2.join();

  int result = flag2.load(std::memory_order_acquire);
  if (result == 1) {
    std::cout << "Barrier works: ordering preserved.\n";
  } else {
    std::cout << "WARNING: barrier may not prevent reordering (result=" << result << ")\n";
  }
}

// -----------------------------------------------------------------------------
// Тест влияния выравнивания
// -----------------------------------------------------------------------------

static void TestAlignmentImpact() {
  std::cout << "\n--- Alignment Impact Test ---\n";
  const size_t len = 1024;
  const int iterations = 10000;
  std::vector<uint8_t> buffer(len + 64);

  for (int offset = 0; offset < 64; offset += 8) {
    uint8_t* ptr = buffer.data() + offset;
    WarmupCache(ptr, len);

    auto start = Clock::now();
    for (int i = 0; i < iterations; ++i) {
      blake3::SecureZeroMemoryMilitary(ptr, len);
    }
    auto end = Clock::now();
    Duration avg = Duration(end - start) / iterations;

    std::cout << "offset " << std::setw(2) << offset
              << " | time = " << std::fixed << std::setprecision(3) << avg.count() << " us\n";
  }
}

// -----------------------------------------------------------------------------
// Тест устойчивости к кэш-атакам (размер буфера кратен кэш-линии)
// -----------------------------------------------------------------------------

static void TestCacheEffects() {
  std::cout << "\n--- Cache Effect Test (various sizes) ---\n";
  const int iterations = 10000;
  for (size_t len = 32; len <= 4096; len *= 2) {
    std::vector<uint8_t> buffer(len + 64);
    uint8_t* ptr = buffer.data();

    auto t_fast = MeasureTime(blake3::SecureZeroMemoryFast, ptr, len, iterations);
    auto t_bal = MeasureTime(blake3::SecureZeroMemoryBalanced, ptr, len, iterations);
    auto t_mil = MeasureTime(blake3::SecureZeroMemoryMilitary, ptr, len, iterations);

    std::cout << "len=" << std::setw(5) << len
              << " | Fast=" << std::setw(8) << std::fixed << std::setprecision(2) << t_fast.count()
              << " | Bal=" << std::setw(8) << t_bal.count()
              << " | Mil=" << std::setw(8) << t_mil.count() << " us\n";
  }
}

// -----------------------------------------------------------------------------
// Main
// -----------------------------------------------------------------------------

int main() {
  std::cout << "==================================================\n"
            << "  Secure Memory Zeroing Benchmark (Enhanced)\n"
            << "==================================================\n\n";

  const std::vector<size_t> sizes = {
      8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096,
      8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576
  };
  const int iter = 10000;

  std::cout << "--- Performance Benchmark (time per call) ---\n";
  std::cout << std::setw(20) << "Function"
            << " | " << std::setw(10) << "Size (B)"
            << " | " << std::setw(15) << "Avg time (us)"
            << " | Status\n";
  std::cout << std::string(70, '-') << '\n';

  for (size_t len : sizes) {
    BenchmarkMemset(len, iter);
    BenchmarkSize(blake3::SecureZeroMemoryFast, "Fast", len, iter);
    BenchmarkSize(blake3::SecureZeroMemoryBalanced, "Balanced", len, iter);
    BenchmarkSize(blake3::SecureZeroMemoryMilitary, "Military", len, iter);
    std::cout << std::string(70, '-') << '\n';
  }

  std::cout << "\n--- Constant-Time Test (RDTSC, cycles) ---\n";
  for (size_t len : {64, 256, 1024, 4096, 16384}) {
    TestConstantTimeRDTSC(blake3::SecureZeroMemoryFast, "Fast", len, 50000);
    TestConstantTimeRDTSC(blake3::SecureZeroMemoryBalanced, "Balanced", len, 50000);
    TestConstantTimeRDTSC(blake3::SecureZeroMemoryMilitary, "Military", len, 50000);
    std::cout << '\n';
  }

  TestMemoryBarrier();
  TestAlignmentImpact();
  TestCacheEffects();

  std::cout << "\nBenchmark completed.\n";
  return 0;
}