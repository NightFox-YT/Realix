#include "keyboard.h"

static const char sctoascii[] = {
    0,  27, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\b', '\t',
    'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', 0,
    'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`', 0,
    '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', 0, '*', 0, ' '
};

static volatile uint8_t kbd_buffer[KBD_BUFF_SIZE];
static volatile uint32_t kbd_head = 0;
static volatile uint32_t kbd_tail = 0;

void keyboard_irq_handler(void)
{
    if (inb(KB_STATUS_PORT) & 0x01)
    {
        uint8_t sc = inb(KB_DATA_PORT);
        
        if (!(sc & 0x80)) 
        {
            uint32_t next = (kbd_head + 1) % KBD_BUFF_SIZE;
            if (next != kbd_tail) 
            { 
                kbd_buffer[kbd_head] = sc;
                kbd_head = next;
            }
        }
    }
    outb(0x20, 0x20); /* EOI (Master PIC) */
}
char keyboard_getc(void)
{
    while (kbd_tail == kbd_head) __asm__ volatile("hlt");

    __asm__ volatile("cli");
    uint8_t sc = kbd_buffer[kbd_tail];
    kbd_tail = (kbd_tail + 1) % KBD_BUFF_SIZE;
    __asm__ volatile("sti");

    if (sc < sizeof(sctoascii)) return sctoascii[sc];
    return 0;
}