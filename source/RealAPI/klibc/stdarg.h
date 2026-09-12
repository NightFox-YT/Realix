#ifndef __realix_stdarg__
#define __realix_stdarg__

typedef __builtin_va_list va_list;

#define VariativeBegin(ap, last) __builtin_va_start(ap, last)
#define VariativeEnd(ap)           __builtin_va_end(ap)
#define VariativeArg(ap, type)     __builtin_va_arg(ap, type)
#define VariativeCopy(dest, src)   __builtin_va_copy(dest, src)

#endif /*__realix_stdarg__*/