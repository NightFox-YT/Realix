// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// Заголовочный файл для безопасного обнуления памяти.

#ifndef BLAKE3_SECURE_MEMORY_H_
#define BLAKE3_SECURE_MEMORY_H_

#include <stddef.h>
#include <stdint.h>

namespace blake3 {

// -----------------------------------------------------------------------------
// Основные функции
// -----------------------------------------------------------------------------

// Быстрое обнуление (только барьер компилятора). ~95% скорости memset.
void SecureZeroMemoryFast(void* ptr, size_t len);

// Сбалансированное обнуление (барьер памяти + компиляторный барьер). ~85% скорости.
void SecureZeroMemoryBalanced(void* ptr, size_t len);

// Максимальная защита (non-temporal записи, кэш-флаш, очистка регистров). ~50% скорости.
void SecureZeroMemoryMilitary(void* ptr, size_t len);

// -----------------------------------------------------------------------------
// Конфигурация (определяется на этапе сборки)
// -----------------------------------------------------------------------------

namespace config {

// Режим по умолчанию: 0 – Balanced, 1 – Fast, 2 – Military.
#ifndef BLAKE3_SECURE_ZERO_MODE
constexpr int kDefaultMode = 0;
#endif

// Использовать случайные паттерны в Military режиме (для SPA/DPA защиты).
#ifndef BLAKE3_SECURE_ZERO_RANDOM_PATTERNS
constexpr bool kUseRandomPatterns = false;
#endif

}  // namespace config

// -----------------------------------------------------------------------------
// Отладочные функции
// -----------------------------------------------------------------------------

#ifdef DEBUG_SECURE_MEMORY

// Проверяет, что вся область обнулена. Возвращает 0 при успехе, иначе -1.
int VerifyZeroMemory(const void* ptr, size_t len);

// Выводит информацию о последней ошибке.
const char* GetSecureMemoryErrorString(void);

#endif  // DEBUG_SECURE_MEMORY

}  // namespace blake3

#endif  // BLAKE3_SECURE_MEMORY_H_