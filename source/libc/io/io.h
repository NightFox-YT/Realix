#ifndef __realix_io_h__
#define __realix_io_h__

#include "../stddef.h"
#include "../stdint.h"

typedef struct _IO_FILE {
	int fd; 			// файловый дескриптор
	int flags; 			// флаги доступа
	char *buf_start; 	// указательн на начало буффера
	char *buf_end; 		// указатель на конец буффера
	char *ptr;			// текущая позиция для чтения/записи
	int buf_size;		// общий размер буффера
	//volatile int lock;  // мьютекс, пока отключил, многопоточности у нас всё равно ещё нет
	int ungetc_buf;		// доп. поле
} FILE;

int vsnprintf(char *str, size_t maxlen, const char *format, va_list ap);
int printf(const char *format, ...);
#endif /*__realix_io_h__*/