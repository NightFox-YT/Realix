#ifndef __realix_idt__
#define __realix_idt__

#include "../include/io.h"

struct idt_entry {
	uint16_t base_low; // младшие 16 бит адреса обработчика
	uint16_t selector; // Селектор сегмента кода в GDT
	uint8_t zero; // 0
	uint8_t flags; // Флаги доступа (ворота там, кольцо)
	uint16_t base_high; // Старшие 16 бит адреса обработчика
} __attribute__((packed));

// структура которая необходима lidt
struct idt_pointer {
	uint16_t limit;
	uint32_t base;
} __attribute__((packed));

/*функции из ассемблера*/
extern void irq0_handler(void); // *глобальные
extern void irq1_handler(void);

void idt_set_gate(uint8_t num, uint32_t base, uint16_t selector, uint8_t flags);
void pic_init(void);
void idt_init(void);
#endif /*__realix_idt__*/