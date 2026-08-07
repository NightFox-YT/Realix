; © Realix > Sample 32-bit RLX Application
; Runs in Protected Mode Ring 3 using System API (INT 0x80)
; =========================================================

bits 32
org 0x00400000

%include 'shared/rlx.inc'

; Заголовок .RLX (16 байт)
rlx_header:
    db RLX_MAGIC_0, RLX_MAGIC_1, RLX_MAGIC_2, RLX_MODE_32
    dd app_entry - rlx_header ; Офсет точки входа (0x10)
    dd code_end - app_entry   ; Размер кода
    dd 0                      ; Зарезервировано

app_entry:
    ; Печать сообщения от 32-битного Ring 3 приложения через INT 0x80
    mov eax, SYS_PRINT_STRING
    mov esi, msg_hello
    int 0x80

    ; Завершение работы приложения (возврат в Ring 0 Shell)
    mov eax, SYS_EXIT
    int 0x80

data_start:
    msg_hello: db '[RLX32 Ring 3 App] Success! Hello from 32-bit User Space (Ring 3) via INT 0x80!', 0x0D, 0x0A, 0
data_end:

code_end:
