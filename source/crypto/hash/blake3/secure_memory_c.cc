// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// C ABI реализация.

#include "secure_memory_c.h"
#include "secure_memory.h"

#include <stddef.h>

using blake3::SecureZeroMemoryFast;
using blake3::SecureZeroMemoryBalanced;
using blake3::SecureZeroMemoryMilitary;
using blake3::SecureZeroMemoryUniversal;

extern "C" int secure_zero_memory(void* ptr, size_t len, SecureZeroMode mode) {
    if (!ptr || len == 0) {
        return -1;
    }

    switch (mode) {
        case SECURE_ZERO_FAST:
            SecureZeroMemoryFast(ptr, len);
            break;
        case SECURE_ZERO_BALANCED:
            SecureZeroMemoryBalanced(ptr, len);
            break;
        case SECURE_ZERO_MILITARY:
            SecureZeroMemoryMilitary(ptr, len);
            break;
        case SECURE_ZERO_UNIVERSAL:
            SecureZeroMemoryUniversal(ptr, len);
            break;
        default:
            return -1;
    }
    return 0;
}

#ifdef DEBUG_SECURE_MEMORY
#include "secure_memory.h"

extern "C" int secure_verify_zero(const void* ptr, size_t len) {
    return blake3::VerifyZeroMemory(ptr, len);
}

extern "C" const char* secure_last_error() {
    return blake3::GetSecureMemoryErrorString();
}
#endif