; © Realix > Kernel16: .RLX Application Loader
; =============================================

bits 16

%include 'shared/rlx.inc'

RLX16_APP_SEGMENT equ 0x3000

; Запуск .RLX приложения по имени файла
; Вход: si = указатель на аргумент имени файла из командной строки
exec_rlx16:
    pusha
    push es
    push ds

    call skip_spaces
    cmp byte [si], 0
    je .no_arg

    call format_83
    jc .bad_name

    ; 1. Загрузка файла в память RLX16_APP_SEGMENT:0000
    mov si, filename_83
    mov cx, RLX16_APP_SEGMENT
    xor bx, bx
    mov di, guard_kernel16
    call file_load
    jnc .file_loaded

    ; Ошибка загрузки (сообщение возвращено в si)
    call print
    call print_new_line
    pop ds
    pop es
    popa
    ret

.file_loaded:
    ; 2. Проверка заголовка .RLX
    mov ax, RLX16_APP_SEGMENT
    mov es, ax

    cmp byte [es:0], RLX_MAGIC_0
    jne .invalid_header
    cmp byte [es:1], RLX_MAGIC_1
    jne .invalid_header
    cmp byte [es:2], RLX_MAGIC_2
    jne .invalid_header
    cmp byte [es:3], RLX_MODE_16
    je .mode_ok
    cmp byte [es:3], RLX_MODE_UNIVERSAL
    je .mode_ok
    jmp .invalid_mode
.mode_ok:

    ; 3. Вычисление точки входа и запуск
    mov bx, [es:4] ; Entry offset (например 0x0010)

    mov si, msg_rlx_starting
    call print

    mov byte [cs:app16_running], 1

    ; Настройка сегментов и дальний вызов (Far Call) 0x3000:bx
    push ds
    push es
    
    mov ds, ax
    mov es, ax

    ; Сохраняем адреса на стеке для retf
    push cs
    push .app_return

    push ax  ; Segment (0x3000)
    push bx  ; Offset (0x0010)
    retf     ; Far jump в 0x3000:bx

.app_return:
    pop es
    pop ds

    mov si, msg_rlx_finished
    call print

    pop ds
    pop es
    popa
    ret

.no_arg:
    mov si, err_exec_no_arg
    call print
    pop ds
    pop es
    popa
    ret

.bad_name:
    mov si, err_exec_bad_name
    call print
    pop ds
    pop es
    popa
    ret

.invalid_header:
    mov si, err_rlx_header
    call print
    pop ds
    pop es
    popa
    ret

.invalid_mode:
    mov si, err_rlx_mode
    call print
    pop ds
    pop es
    popa
    ret

err_exec_no_arg:   db '[!] Usage: exec <filename.rlx>', ENTER, 0
err_exec_bad_name: db '[!] Invalid filename format for FAT12.', ENTER, 0
msg_rlx_starting:  db '[RLX16 Loader] Executing 16-bit application...', ENTER, 0
msg_rlx_finished:  db '[RLX16 Loader] Application finished.', ENTER, 0
err_rlx_header:    db '[!] RLX Loader Error: Invalid .RLX header magic!', ENTER, 0
err_rlx_mode:      db '[!] RLX Loader Error: Not a 16-bit .RLX binary!', ENTER, 0
