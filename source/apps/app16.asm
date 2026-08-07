; © Realix > Sample 16-bit RLX Application
; Runs using Kernel16 System API (INT 0x80)
; =========================================

bits 16
org 0x0

%include 'shared/rlx.inc'

; Заголовок .RLX (16 байт)
rlx_header:
    db RLX_MAGIC_0, RLX_MAGIC_1, RLX_MAGIC_2, RLX_MODE_16
    dw app_entry - rlx_header ; Офсет точки входа (0x10)
    dw code_end - app_entry   ; Размер кода
    dw data_end - data_start  ; Размер данных
    dw 0x0200                 ; Размер стека (512 байт)
    dd 0                      ; Зарезервировано

app_entry:
    ; Печать приветственного сообщения от 16-битного приложения
    mov ax, SYS_PRINT_STRING
    mov si, msg_hello
    int 0x80

    ; Ожидание нажатия клавиши
    mov ax, SYS_READ_KEY
    int 0x80

    ; Выход из приложения
    mov ax, SYS_EXIT
    int 0x80

    retf

data_start:
    msg_hello: db '[RLX16 App] Hello from 16-bit User Application via System API INT 0x80!', 0x0D, 0x0A, 'Press any key to exit app...', 0x0D, 0x0A, 0
data_end:

code_end:
