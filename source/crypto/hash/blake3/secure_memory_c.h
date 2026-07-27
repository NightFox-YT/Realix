// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// C ABI для безопасного обнуления памяти.

#ifndef BLAKE3_SECURE_MEMORY_C_H_
#define BLAKE3_SECURE_MEMORY_C_H_

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Режимы обнуления
typedef enum {
    SECURE_ZERO_FAST      = 1,  // только компиляторный барьер
    SECURE_ZERO_BALANCED  = 2,  // барьеры памяти + компилятор
    SECURE_ZERO_MILITARY  = 3,  // non‑temporal, кэш‑флаш, регистры
    SECURE_ZERO_UNIVERSAL = 4   // максимальная защита
} SecureZeroMode;

// Основная функция – обнуление памяти с выбором режима.
// Возвращает 0 при успехе, -1 при ошибке (если ptr == NULL или len == 0).
int secure_zero_memory(void* ptr, size_t len, SecureZeroMode mode);

// обёртки для каждого режима
static inline int secure_zero_memory_fast(void* ptr, size_t len) {
    return secure_zero_memory(ptr, len, SECURE_ZERO_FAST);
}
static inline int secure_zero_memory_balanced(void* ptr, size_t len) {
    return secure_zero_memory(ptr, len, SECURE_ZERO_BALANCED);
}
static inline int secure_zero_memory_military(void* ptr, size_t len) {
    return secure_zero_memory(ptr, len, SECURE_ZERO_MILITARY);
}
static inline int secure_zero_memory_universal(void* ptr, size_t len) {
    return secure_zero_memory(ptr, len, SECURE_ZERO_UNIVERSAL);
}

// Отладочная функция
#ifdef DEBUG_SECURE_MEMORY
int secure_verify_zero(const void* ptr, size_t len);
const char* secure_last_error(void);
#endif

#ifdef __cplusplus
}
#endif

#endif  // BLAKE3_SECURE_MEMORY_C_H_ 