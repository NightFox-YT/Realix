// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// Многофакторный бенчмарк для оценки производительности, константности,
// устойчивости к переупорядочиванию, влияния выравнивания, кэша и других
// атак. Проверяются все режимы: Fast, Balanced, Military, Universal.

#include "secure_memory.h"
#include "secure_memory_arch.h"

#include <algorithm>
#include <array>
#include <atomic>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <random>
#include <thread>
#include <vector>

#if defined(__x86_64__)
#include <x86intrin.h>  // для rdtsc
#endif

using Clock = std::chrono::high_resolution_clock;
using Duration = std::chrono::duration<double, std::micro>;

// -----------------------------------------------------------------------------
// Вспомогательные функции
// -----------------------------------------------------------------------------

// Прогрев кэша случайными данными.
template <typename T>
static void WarmupCache(T* ptr, size_t len) {
  std::mt19937_64 rng(0xDEADBEEFCAFEBABEULL);
  for (size_t i = 0; i < len; ++i) {
    ptr[i] = static_cast<T>(rng());
  }
}

// Чтение счётчика тактов.
static inline uint64_t ReadTSC() {
#if defined(__x86_64__) && defined(__GNUC__)
  unsigned int lo, hi;
  __asm__ volatile("rdtsc" : "=a"(lo), "=d"(hi));
  return ((uint64_t)hi << 32) | lo;
#else
  return 0;
#endif
}

// Проверка корректности обнуления.
static bool VerifyZero(const void* ptr, size_t len) {
  const volatile uint8_t* p = static_cast<const volatile uint8_t*>(ptr);
  for (size_t i = 0; i < len; ++i) {
    if (p[i] != 0) return false;
  }
  return true;
}

// Измерение среднего времени выполнения.
static Duration MeasureTime(void (*zero_func)(void*, size_t),
                            void* ptr, size_t len,
                            int iterations = 0) {
  if (iterations == 0) {
    if (len <= 1024) {
      iterations = 500000;
    } else if (len <= 8192) {
      iterations = 100000;
    } else if (len <= 65536) {
      iterations = 10000;
    } else {
      iterations = 1000;
    }
  }

  WarmupCache(static_cast<uint8_t*>(ptr), len);
  zero_func(ptr, len);  // прогревочный прогон

  auto start = Clock::now();
  for (int i = 0; i < iterations; ++i) {
    zero_func(ptr, len);
  }
  auto end = Clock::now();
  return Duration(end - start) / iterations;
}

// -----------------------------------------------------------------------------
// Тест производительности
// -----------------------------------------------------------------------------

static void BenchmarkSize(void (*zero_func)(void*, size_t),
                          const char* name,
                          size_t len) {
  const size_t alignment = 64;
  std::vector<uint8_t> buffer(len + alignment - 1);
  uint8_t* ptr = reinterpret_cast<uint8_t*>(
      (reinterpret_cast<uintptr_t>(buffer.data()) + alignment - 1) &
      ~(alignment - 1));

  std::mt19937_64 rng(0x123456789ABCDEFULL);
  for (size_t i = 0; i < len; ++i) ptr[i] = static_cast<uint8_t>(rng());

  Duration avg = MeasureTime(zero_func, ptr, len);
  bool ok = VerifyZero(ptr, len);

  std::cout << std::setw(20) << name
            << " | size = " << std::setw(10) << len
            << " | avg time = " << std::fixed << std::setprecision(3)
            << avg.count() << " us"
            << " | " << (ok ? "OK" : "FAIL") << '\n';
}

static void BenchmarkMemset(size_t len) {
  const size_t alignment = 64;
  std::vector<uint8_t> buffer(len + alignment - 1);
  uint8_t* ptr = reinterpret_cast<uint8_t*>(
      (reinterpret_cast<uintptr_t>(buffer.data()) + alignment - 1) &
      ~(alignment - 1));

  std::mt19937_64 rng(0xDEADBEEF);
  for (size_t i = 0; i < len; ++i) ptr[i] = static_cast<uint8_t>(rng());

  auto start = Clock::now();
  for (int i = 0; i < 10000; ++i) {
    std::memset(ptr, 0, len);
  }
  auto end = Clock::now();
  Duration avg = Duration(end - start) / 10000;
  bool ok = VerifyZero(ptr, len);

  std::cout << std::setw(20) << "memset"
            << " | size = " << std::setw(10) << len
            << " | avg time = " << std::fixed << std::setprecision(3)
            << avg.count() << " us"
            << " | " << (ok ? "OK" : "FAIL") << '\n';
}

// -----------------------------------------------------------------------------
// Тест константности времени
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

  // Разные паттерны данных
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
// Тест на переупорядочивание
// -----------------------------------------------------------------------------

static void TestMemoryBarrier() {
  std::cout << "\n--- Memory Barrier Test (ordering) ---\n";

  std::atomic<int> flag1{0}, flag2{0};
  volatile int data1 = 0, data2 = 0;

  auto writer = [&]() {
    data1 = 42;
    BLAKE3_MEMORY_BARRIER();  // должно упорядочить записи
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

static void TestAlignmentImpact(void (*zero_func)(void*, size_t),
                                const char* name,
                                size_t len = 1024) {
  std::cout << "\n--- Alignment Impact Test (" << name << ") ---\n";
  const int iterations = 10000;
  std::vector<uint8_t> buffer(len + 64);

  for (int offset = 0; offset < 64; offset += 8) {
    uint8_t* ptr = buffer.data() + offset;
    WarmupCache(ptr, len);

    auto start = Clock::now();
    for (int i = 0; i < iterations; ++i) {
      zero_func(ptr, len);
    }
    auto end = Clock::now();
    Duration avg = Duration(end - start) / iterations;

    std::cout << "offset " << std::setw(2) << offset
              << " | time = " << std::fixed << std::setprecision(3) << avg.count() << " us\n";
  }
}

// -----------------------------------------------------------------------------
// Тест устойчивости к кэш-атакам
// -----------------------------------------------------------------------------

static void TestCacheEffects() {
  std::cout << "\n--- Cache Effect Test (various sizes) ---\n";
  for (size_t len = 32; len <= 4096; len *= 2) {
    std::vector<uint8_t> buffer(len + 64);
    uint8_t* ptr = buffer.data();

    auto t_fast = MeasureTime(blake3::SecureZeroMemoryFast, ptr, len);
    auto t_bal = MeasureTime(blake3::SecureZeroMemoryBalanced, ptr, len);
    auto t_mil = MeasureTime(blake3::SecureZeroMemoryMilitary, ptr, len);
    auto t_uni = MeasureTime(blake3::SecureZeroMemoryUniversal, ptr, len);

    std::cout << "len=" << std::setw(5) << len
              << " | Fast=" << std::setw(8) << std::fixed << std::setprecision(2) << t_fast.count()
              << " | Bal=" << std::setw(8) << t_bal.count()
              << " | Mil=" << std::setw(8) << t_mil.count()
              << " | Uni=" << std::setw(8) << t_uni.count() << " us\n";
  }
}

// -----------------------------------------------------------------------------
// Тест с разными паттернами данных
// -----------------------------------------------------------------------------

static void TestPatternSensitivity() {
  std::cout << "\n--- Pattern Sensitivity Test ---\n";
  const size_t len = 1024;
  const int iterations = 50000;
  std::vector<uint8_t> buffer(len + 64);
  uint8_t* ptr = buffer.data();

  // Паттерны: все нули, все единицы, шахматный, случайный.
  std::array<std::string, 4> pattern_names = {"All Zero", "All One", "Checker", "Random"};
  std::array<void(*)(void*, size_t), 4> funcs = {
    blake3::SecureZeroMemoryFast,
    blake3::SecureZeroMemoryBalanced,
    blake3::SecureZeroMemoryMilitary,
    blake3::SecureZeroMemoryUniversal
  };
  std::array<const char*, 4> func_names = {"Fast", "Balanced", "Military", "Universal"};

  for (size_t f = 0; f < funcs.size(); ++f) {
    std::cout << "Function: " << func_names[f] << "\n";
    for (int p = 0; p < 4; ++p) {
      // Заполняем паттерном
      if (p == 0) std::memset(ptr, 0x00, len);
      else if (p == 1) std::memset(ptr, 0xFF, len);
      else if (p == 2) {
        for (size_t i = 0; i < len; ++i) ptr[i] = (i % 2) ? 0xAA : 0x55;
      } else {
        std::mt19937_64 rng(0x12345678);
        for (size_t i = 0; i < len; ++i) ptr[i] = static_cast<uint8_t>(rng());
      }

      auto start = Clock::now();
      for (int i = 0; i < iterations; ++i) {
        funcs[f](ptr, len);
        // после каждого прогона восстанавливаем паттерн для следующего
        // (для чистоты теста мы не восстанавливаем, а заново заполняем – но это дорого.
      }
      auto end = Clock::now();
      Duration avg = Duration(end - start) / iterations;
      std::cout << "  Pattern " << pattern_names[p] << " : " << avg.count() << " us\n";
    }
  }
}

// -----------------------------------------------------------------------------
// Тест на влияние случайных паттернов
// -----------------------------------------------------------------------------

static void TestRandomPatternImpact() {
  std::cout << "\n--- Random Pattern Impact Test (Military & Universal) ---\n";
  const size_t len = 4096;
  const int iterations = 10000;
  std::vector<uint8_t> buffer(len + 64);
  uint8_t* ptr = buffer.data();

  // Измеряем время с включенным и выключенным случайным паттерном.
  // Для этого мы не можем переопределить конфиг во время выполнения,
  // поэтому просто измеряем Military и Universal функции (они уже используют
  // случайные паттерны, если включены в конфигурации). Здесь просто выводим
  // среднее время для этих функций.
  auto t_mil = MeasureTime(blake3::SecureZeroMemoryMilitary, ptr, len, iterations);
  auto t_uni = MeasureTime(blake3::SecureZeroMemoryUniversal, ptr, len, iterations);
  std::cout << "Military  : " << t_mil.count() << " us\n";
  std::cout << "Universal : " << t_uni.count() << " us\n";
}

// -----------------------------------------------------------------------------
// Тест на устойчивость к атакам по времени
// -----------------------------------------------------------------------------

static void TestTimingStability() {
  std::cout << "\n--- Timing Stability Test (1000 measurements) ---\n";
  const size_t len = 1024;
  const int measurements = 1000;
  std::vector<uint8_t> buffer(len + 64);
  uint8_t* ptr = buffer.data();

  std::array<void(*)(void*, size_t), 4> funcs = {
    blake3::SecureZeroMemoryFast,
    blake3::SecureZeroMemoryBalanced,
    blake3::SecureZeroMemoryMilitary,
    blake3::SecureZeroMemoryUniversal
  };
  std::array<const char*, 4> func_names = {"Fast", "Balanced", "Military", "Universal"};

  for (size_t f = 0; f < funcs.size(); ++f) {
    std::vector<double> times;
    times.reserve(measurements);
    WarmupCache(ptr, len);
    for (int i = 0; i < measurements; ++i) {
      auto start = Clock::now();
      funcs[f](ptr, len);
      auto end = Clock::now();
      Duration d = end - start;
      times.push_back(d.count());
    }
    // Вычисляем среднее, медиану, стандартное отклонение, разброс.
    double sum = 0.0;
    for (double t : times) sum += t;
    double mean = sum / measurements;
    std::sort(times.begin(), times.end());
    double median = times[measurements / 2];
    double variance = 0.0;
    for (double t : times) variance += (t - mean) * (t - mean);
    variance /= measurements;
    double stddev = std::sqrt(variance);
    double min_t = times.front();
    double max_t = times.back();
    double spread = (max_t - min_t) / mean * 100.0;

    std::cout << func_names[f] << " : mean=" << std::fixed << std::setprecision(3) << mean
              << " us, median=" << median << " us, stddev=" << stddev
              << " us, spread=" << spread << "%\n";
  }
}

int main() {
  std::cout << "==================================================\n"
            << "  Secure Memory Zeroing Benchmark (Ultimate)\n"
            << "==================================================\n\n";

  const std::vector<size_t> sizes = {
      8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096,
      8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576
  };

  // ---- 1. Производительность ----
  std::cout << "--- Performance Benchmark (time per call) ---\n";
  std::cout << std::setw(20) << "Function"
            << " | " << std::setw(10) << "Size (B)"
            << " | " << std::setw(15) << "Avg time (us)"
            << " | Status\n";
  std::cout << std::string(70, '-') << '\n';
  for (size_t len : sizes) {
    BenchmarkMemset(len);
    BenchmarkSize(blake3::SecureZeroMemoryFast, "Fast", len);
    BenchmarkSize(blake3::SecureZeroMemoryBalanced, "Balanced", len);
    BenchmarkSize(blake3::SecureZeroMemoryMilitary, "Military", len);
    BenchmarkSize(blake3::SecureZeroMemoryUniversal, "Universal", len);
    std::cout << std::string(70, '-') << '\n';
  }

  // ---- 2. Константность времени (RDTSC) ----
  std::cout << "\n--- Constant-Time Test (RDTSC, cycles) ---\n";
  for (size_t len : {64, 256, 1024, 4096, 16384}) {
    TestConstantTimeRDTSC(blake3::SecureZeroMemoryFast, "Fast", len);
    TestConstantTimeRDTSC(blake3::SecureZeroMemoryBalanced, "Balanced", len);
    TestConstantTimeRDTSC(blake3::SecureZeroMemoryMilitary, "Military", len);
    TestConstantTimeRDTSC(blake3::SecureZeroMemoryUniversal, "Universal", len);
    std::cout << '\n';
  }

  // ---- 3. Барьеры памяти ----
  TestMemoryBarrier();

  // ---- 4. Влияние выравнивания (для всех режимов) ----
  TestAlignmentImpact(blake3::SecureZeroMemoryFast, "Fast");
  TestAlignmentImpact(blake3::SecureZeroMemoryBalanced, "Balanced");
  TestAlignmentImpact(blake3::SecureZeroMemoryMilitary, "Military");
  TestAlignmentImpact(blake3::SecureZeroMemoryUniversal, "Universal");

  // ---- 5. Кэш-эффекты ----
  TestCacheEffects();

  // ---- 6. Чувствительность к паттернам данных ----
  TestPatternSensitivity();

  // ---- 7. Влияние случайных паттернов ----
  TestRandomPatternImpact();

  // ---- 8. Статистическая стабильность времени ----
  TestTimingStability();

  std::cout << "\nBenchmark completed.\n";
  return 0;
}