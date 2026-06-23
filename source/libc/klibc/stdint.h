/* stdint.h\standard integer
    © 2026 Alexander Silaev
    The realix libc
*/
#ifndef __realix_stdint_h__
#define __realix_stdint_h__

/* 1. Точные типы */
typedef __INT8_TYPE__   int8_t;
typedef __UINT8_TYPE__  uint8_t;
typedef __INT16_TYPE__  int16_t;
typedef __UINT16_TYPE__ uint16_t;
typedef __INT32_TYPE__  int32_t;
typedef __UINT32_TYPE__ uint32_t;
typedef __INT64_TYPE__  int64_t;
typedef __UINT64_TYPE__ uint64_t;

/* 2. Минимально необходимые типы */
typedef __INT_LEAST8_TYPE__   int_least8_t;
typedef __UINT_LEAST8_TYPE__  uint_least8_t;
typedef __INT_LEAST16_TYPE__  int_least16_t;
typedef __UINT_LEAST16_TYPE__ uint_least16_t;
typedef __INT_LEAST32_TYPE__  int_least32_t;
typedef __UINT_LEAST32_TYPE__ uint_least32_t;
typedef __INT_LEAST64_TYPE__  int_least64_t;
typedef __UINT_LEAST64_TYPE__ uint_least64_t;

/* 3. Самые быстрые типы */
typedef __INT_FAST8_TYPE__  int_fast8_t;
typedef __UINT_FAST8_TYPE__  uint_fast8_t;
typedef __INT_FAST16_TYPE__  int_fast16_t;
typedef __UINT_FAST16_TYPE__ uint_fast16_t;
typedef __INT_FAST32_TYPE__ int_fast32_t;
typedef __UINT_FAST32_TYPE__ uint_fast32_t;
typedef __INT_FAST64_TYPE__ int_fast64_t;
typedef __UINT_FAST64_TYPE__ uint_fast64_t;

/* 4. Типы для хранения указателей */
typedef __INTPTR_TYPE__ intptr_t;
typedef __UINTPTR_TYPE__ uintptr_t;

/* 5. Максимально емкие типы */
typedef __INTMAX_TYPE__ intmax_t;
typedef __UINTMAX_TYPE__ uintmax_t;


/* --- Предельные значения --- */

#define INT8_MIN         (-128)
#define INT8_MAX         (127)
#define UINT8_MAX        (255)

#define INT16_MIN        (-32768)
#define INT16_MAX        (32767)
#define UINT16_MAX       (65535)

#define INT32_MIN        (-2147483647 - 1)
#define INT32_MAX        (2147483647)
#define UINT32_MAX       (4294967295U)

#define INT64_MIN        (-9223372036854775807LL - 1LL)
#define INT64_MAX        (9223372036854775807LL)
#define UINT64_MAX       (18446744073709551615ULL)

#define INTPTR_MIN       INT32_MIN
#define INTPTR_MAX       INT32_MAX
#define UINTPTR_MAX      UINT32_MAX

#define INTMAX_MIN       INT64_MIN
#define INTMAX_MAX       INT64_MAX
#define UINTMAX_MAX      UINT64_MAX

#define PTRDIFF_MIN      INT32_MIN
#define PTRDIFF_MAX      INT32_MAX

#define SIZE_MAX         UINT32_MAX

#endif /* __realix_stdint_h__ */