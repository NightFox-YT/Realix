// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// Безопасное обнуление памяти.
// Поддерживает все популярные архитектуры: x86, x86_64, ARM (32/64),
// RISC‑V, PowerPC, MIPS, SPARC.

#ifndef BLAKE3_SECURE_MEMORY_H_
#define BLAKE3_SECURE_MEMORY_H_

#include <stddef.h>
#include <stdint.h>

namespace blake3 {

// -----------------------------------------------------------------------------
// Основные функции
// -----------------------------------------------------------------------------

// Быстрое обнуление (только барьер компилятора).
void SecureZeroMemoryFast(void* ptr, size_t len);

// Сбалансированное обнуление (барьер памяти + компиляторный).
void SecureZeroMemoryBalanced(void* ptr, size_t len);

// Максимальная защита (non-temporal, кэш-флаш, очистка регистров).
void SecureZeroMemoryMilitary(void* ptr, size_t len);

// Универсальное обнуление с адаптивными проходами и максимальной защитой.
// Поддерживает все архитектуры и использует аппаратные оптимизации.
void SecureZeroMemoryUniversal(void* ptr, size_t len);

// -----------------------------------------------------------------------------
// Конфигурация
// -----------------------------------------------------------------------------

namespace config {

// Режим по умолчанию: 0 – Balanced, 1 – Fast, 2 – Military, 3 – Universal.
#ifndef BLAKE3_SECURE_ZERO_MODE
constexpr int kDefaultMode = 0;
#endif

// Количество проходов записи для Universal (по умолчанию 3)
#ifndef BLAKE3_SECURE_ZERO_PASSES
constexpr int kZeroPasses = 3;
#endif

// Использовать случайный паттерн в одном из проходов
#ifndef BLAKE3_SECURE_ZERO_RANDOM_PATTERN
constexpr bool kUseRandomPattern = true;
#endif

// Принудительный кэш-флаш
#ifndef BLAKE3_SECURE_ZERO_CACHE_FLUSH
constexpr bool kEnableCacheFlush = true;
#endif

// Очистка регистров
#ifndef BLAKE3_SECURE_ZERO_CLEAR_REGS
constexpr bool kClearRegisters = true;
#endif

// Добавлять случайную задержку
#ifndef BLAKE3_SECURE_ZERO_RANDOM_DELAY
constexpr bool kEnableRandomDelay = true;
#endif

}  // namespace config

// -----------------------------------------------------------------------------
// Отладочные функции
// -----------------------------------------------------------------------------

#ifdef DEBUG_SECURE_MEMORY
int VerifyZeroMemory(const void* ptr, size_t len);
const char* GetSecureMemoryErrorString(void);
#endif

}  // namespace blake3

#endif  // BLAKE3_SECURE_MEMORY_H_