#ifndef __realix_com__
#define __realix_com__

#define COMO_BASE 0x3F8 // базовый serial порт

#define SERIAL_DATA(base) (base + 0)
#define SERIAL_INTERRUPT_ENABLE(base) (base + 1)
#define SERIAL_FIFO_COMMAND(base) (base + 2)
#define SERIAL_LINE_COMMAND(base) (base + 3)
#define SERIAL_MODEM_COMMAND(base) (base + 4)
#define SERIAL_LINE_STATUS(base) (base + 5)

int init_serial(void);
void serial_putc(char c);
void serial_print(const char *str);


#endif /*__realix_com__*/
