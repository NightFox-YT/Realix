#include "../include/io.h"
#include "com.h"

int init_serial(void)
{
    outb(COMO_BASE + 1, 0x00); // Отключение всех прерываний от UART
    outb(COMO_BASE + 3, 0x80); //  Включение DLAB

    outb(COMO_BASE + 0, 0x03); // младший байт делителя
    outb(COMO_BASE + 1, 0x00); // старший байт делителя

    outb(COMO_BASE + 3, 0x03); // отключение DLAB и ставим режим: 8N1
    outb(COMO_BASE + 2, 0xC7); // включение FIFO, очистка буфферов чтения и записи, ставим триггер на 14 байт
    outb(COMO_BASE + 4, 0x0B); // включение IRQ, устанавливаем сигналы RTS и DTR

    outb(COMO_BASE + 4, 0x1E); // проверка порта, перевод в Loopback 
    outb(COMO_BASE + 0, 0xAE); // пишем тестовый байт

    if(inb(COMO_BASE + 0) != 0xAE) return 1; // если не вернулся тот же байт который и отправили то порт неисправен или отсутствует

    outb(COMO_BASE + 4, 0x0F); // если всё ок то переводим в UART
    return 0;
}
// проверка пуст ли буффер перед отправкой
static int is_transmit_empty() { return inb(COMO_BASE + 5) & 0x20; } // 5'й бит регистра LSR возвращает 1 если буффер пуст

void serial_putc(char c)
{
    while (is_transmit_empty() == 0);
    outb(COMO_BASE, c);
}

void serial_print(const char *str)
{
    while (*str)
    {
        // дополнение возвратом коретки, хотя обычно так не делают но для совместимости с щитдовс пусть будет
        if (*str == '\n') serial_putc('\r');
        serial_putc(*str);
        str++;
    }
}