#ifndef __realix_stdlib__
#define __realix_stdlib__

#include "stdint.h"

void xtoa(uint32_t num, char *buf, uint32_t base, int uppercase);
void itoa(int32_t value, char *str, int base);


#endif /*__realix_stdlib__*/