; © Realix > Sample 32-bit RLX Application
; ø Copyright by @Ramix
; (07.09.26) v0.12
; ================
; ❗️ Адрес `org` обязан совпадать с RLX32_LOAD_ADDR в kernel32/src/commands/exec.rs

; Настройка компиляции
bits 32
org 0x300000

; Основные константы
%include 'shared/config.asm'

; Заголовок .RLX (16 байт, формат общий с 16-битными приложениями)
rlx_header:
    dw RLX_MAGIC               ; Магическое число для опознания
    dw RLX_MODE_32              ; Режим работы приложения
    dw app_entry - rlx_header  ; Оффсет точки входа
    dw 0                       ; Зарезервировано

app_entry:
    ; ❗️ exec.rs вызывает точку входа как `unsafe extern "C" fn()` - по соглашению C
    ; регистры EBX/ESI/EDI/EBP должны сохраняться вызываемой стороной (Используем esi ниже)
    push esi

    ; Печать приветственного сообщения от 32-битного приложения
    mov ah, SYS_PRINT_STRING
    mov esi, msg_hello
    int 0x80

    ; Ожидание нажатия клавиши
    mov ah, SYS_READ_KEY
    int 0x80

    ; Выход из приложения
    mov ah, SYS_EXIT
    int 0x80

    pop esi
    ret

data_start:
    msg_hello: db '[RLX32 App] Hello from 32-bit User Application via System API INT 0x80!', 0x0D, 0x0A, 'Press any key to exit app...', 0x0D, 0x0A, 0
data_end:

code_end:
