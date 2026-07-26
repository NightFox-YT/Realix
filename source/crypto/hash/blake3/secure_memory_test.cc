// Copyright 2026 Gleb Obitotsky. All rights reserved.
// Use of this source code is governed by a GNU GPL V3 license.
//
// Author: Gleb Obitotsky <glebobitotsky@yandex.com>
//
// Набор тестов для проверки корректности обнуления памяти.

#include "secure_memory.h"

#include <iostream>
#include <vector>
#include <random>

namespace {

bool TestZeroing(void (*zero_func)(void*, size_t), const char* name) {
  std::cout << "Testing " << name << "... ";
  const size_t kLen = 256;
  std::vector<uint8_t> buffer(kLen, 0xAA);

  zero_func(buffer.data(), kLen);

  for (size_t i = 0; i < kLen; ++i) {
    if (buffer[i] != 0) {
      std::cerr << "FAILED at byte " << i << "\n";
      return false;
    }
  }
  std::cout << "PASSED\n";
  return true;
}

bool TestEdgeCases() {
  std::cout << "Testing edge cases... ";
  // Проверяем, что функции не падают при нулевой длине
  // В данном тесте мы просто проверяем, что при len=0 ничего не происходит
  // (но циклы не выполнятся, так как words=0, bytes=0).
  uint8_t dummy = 0xFF;
  blake3::SecureZeroMemoryFast(&dummy, 0);
  blake3::SecureZeroMemoryBalanced(&dummy, 0);
  blake3::SecureZeroMemoryMilitary(&dummy, 0);
  // Проверяем, что dummy не изменился (он не должен, т.к. len=0)
  if (dummy != 0xFF) {
    std::cerr << "FAILED: dummy changed with len=0\n";
    return false;
  }
  std::cout << "PASSED\n";
  return true;
}

}  // anonymous namespace

int main() {
  bool ok = true;
  ok &= TestZeroing(blake3::SecureZeroMemoryFast, "SecureZeroMemoryFast");
  ok &= TestZeroing(blake3::SecureZeroMemoryBalanced, "SecureZeroMemoryBalanced");
  ok &= TestZeroing(blake3::SecureZeroMemoryMilitary, "SecureZeroMemoryMilitary");
  ok &= TestEdgeCases();

  if (ok) {
    std::cout << "\nAll tests passed.\n";
    return 0;
  } else {
    std::cout << "\nSome tests failed.\n";
    return 1;
  }
}