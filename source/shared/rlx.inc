; © Realix > Shared: RLX Application & Syscall Definitions
; (27.07.26) v0.1
; =======================================================

%ifndef RLX_INC
%define RLX_INC

; Магический заголовок файла .RLX
%define RLX_MAGIC_0 'R'
%define RLX_MAGIC_1 'L'
%define RLX_MAGIC_2 'X'

; Флаги режима архитектуры
%define RLX_MODE_16        0x16   ; 16-битное приложение (Real Mode / System API)
%define RLX_MODE_32        0x32   ; 32-битное приложение (Protected Mode / Ring 3)
%define RLX_MODE_UNIVERSAL 'U'    ; Универсальное бинарное приложение (16-bit + 32-bit)

; Системные вызовы (INT 0x80)
%define SYS_PRINT_STRING 1  ; Печать строки (DS:SI для 16-бит, ESI для 32-бит)
%define SYS_PUTCHAR      2  ; Печать символа (DL для 16/32-бит)
%define SYS_EXIT         3  ; Завершение работы программы
%define SYS_READ_KEY     4  ; Ожидание нажатия клавиши (символ в AL)
%define SYS_CLEAR        5  ; Очистка экрана
%define SYS_GET_SYSINFO  6  ; Запрос системной информации (EDI указатель на RealixSysInfo)

%endif
