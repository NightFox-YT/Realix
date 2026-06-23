#ifndef __realix_keyboard__
#define __realix_keyboard__

#include "../../include/io.h"
#include "../../libc/klibc/stdint.h"

#define KB_DATA_PORT 0x60
#define KB_STATUS_PORT 0x64
#define US_KEY_ENTER 0x1C
#define US_KEY_BACKSPACE 0x0E
#define US_KEY_SPACE 0x39
#define KBD_BUFF_SIZE 256

char keyboard_getc(void);
void keyboard_irq_handler(void);

#endif /*__realix_keyboard__*/