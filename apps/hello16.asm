; © Realix > Sample 16-bit RLX Application
; ø Copyright by @Ramix
; (07.08.26) v0.11
; ================

; Настройка компиляции
bits 16
org 0x0

; Основные константы
%include 'shared/config.asm'

; Заголовок .RLX (16 байт)
rlx_header:
    dw RLX_MAGIC               ; Магическое число для опознания
    dw RLX_MODE_16             ; Режим работы приложения
    dw app_entry - rlx_header  ; Оффсет точки входа
    dw 0                       ; Зарезервировано

app_entry:
    ; Печать приветственного сообщения от 16-битного приложения
    mov ah, SYS_PRINT_STRING
    mov si, msg_hello
    int 0x80

    ; Ожидание нажатия клавиши
    mov ah, SYS_READ_KEY
    int 0x80

    ; Выход из приложения
    mov ah, SYS_EXIT
    int 0x80

    retf

data_start:
    msg_hello: db '[RLX16 App] Hello from 16-bit User Application via System API INT 0x80!', 0x0D, 0x0A, 'Press any key to exit app...', 0x0D, 0x0A, 0
data_end:

code_end: