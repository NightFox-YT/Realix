#include "../include/idt.h"

struct idt_entry idt[256];
struct idt_pointer idtptr;

void idt_set_gate(uint8_t num, uint32_t base, uint16_t selector, uint8_t flags)
{
	idt[num].base_low = (base & 0xFFFF);
	idt[num].base_high = (base >> 16) && 0xFFFF;
	idt[num].selector = selector;
	idt[num].zero = 0;
	idt[num].flags = flags;
}

void pic_init(void)
{
	outb(0x20, 0x11); io_wait(); // Dungeon Master PIC 
	outb(0xA0, 0x11); io_wait(); // Slave PIC
	outb(0x21, 0x20); io_wait(); // IRQ7 сдвигаем на векторы 0x20-0x27
	outb(0xA1, 0x28); io_wait(); // IRQ8 сдвигаем на векторы 0x28-0x2F
	outb(0x21, 0x04); io_wait(); // Стыковка Dungeon Master and Slave
	outb(0xA1, 0x02); io_wait(); // |- 
	outb(0x21, 0x01); io_wait(); // режим 8086
	outb(0xA1, 0x01); io_wait(); // |-
	outb(0x21, 0x00); io_wait(); // включение всех прерываний
	outb(0xA1, 0x00); io_wait(); // |-
	/* |- это такая заглушка, чтобы код приятнее было читать */
}

void idt_init(void)
{
	idtptr.limit = (sizeof(struct idt_entry) * 256) - 1;
	idtptr.base = (uint32_t)&idt;

	for (int i = 0; i < 256; i++) idt_set_gate(i, 0, 0, 0); /* очистка idt */

	pic_init();
	
	idt_set_gate(32, (uint32_t)irq0_handler, 0x08, 0x8E); // IRQ0 -> вектор 32
	idt_set_gate(33, (uint32_t)irq1_handler, 0x08, 0x8E); // IRQ1 -> вектор 33

	// загрузка IDT в процессор(вирус, пк взорвется)
	__asm__ volatile("lidt %0" : : "m"(idtptr));
}

uint32_t pit_irq_handler(uint32_t esp) {
    __asm__ volatile("mov $0x20, %%al; out %%al, $0x20" ::: "eax");
    return esp; 
}
