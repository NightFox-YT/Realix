/* stddef.h
    © 2026 Alexander Silaev
    The realix libc
*/

#ifndef __realix_stddef_h__
#define __realix_stddef_h__

#undef NULL
#if defined(__cplusplus)
#  define NULL 0
#else
#  define NULL ((void *)0)
#endif

typedef __SIZE_TYPE__ size_t;
typedef __PTRDIFF_TYPE__ ptrdiff_t;
//typedef __WCHAR_TYPE__ wchar_t; clangd почему то ругается на это, пока уберу

#if defined(__GNUC__) || defined(__clang__)
#  define offsetof(type, member) __builtin_offsetof(type, member)
#else
#  define offsetof(type, member) ((size_t)&(((type *)0)->member))
#endif

#endif /* __realix_stddef_h__ */