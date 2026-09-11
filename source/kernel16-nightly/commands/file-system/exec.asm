; © Realix > Command: .rlx loader
; ø Copyright by @Ramix
; (07.08.26) v0.11
; ================
; ❗️ Зависимости: kernel16-nightly/syscall, bios-api/fat12/file_load,
;                 kernel16-nightly/io: print & print_reg

; Настройка компиляции
bits 16

; Основные константы
%include 'shared/config.asm'

; Полный адрес загрузки приложения
RLX16_APP_SEGMENT equ 0x2000
RLX16_APP_OFFSET  equ 0x0000

; > Команда запуска .RLX приложения по имени файла с диска: exec <имя файла>
; Параметры:
;  - si: указатель на имя файла (после имени команды)
cmd_exec16:
    pusha
    push es
    push ds

    ; Разбор имени и загрузка файла по адресу RLX16_APP_SEGMENT:RLX16_APP_OFFSET
    call parse_and_load
    jc .fail

.file_loaded:
    ; Проверка заголовка .RLX
    mov ax, RLX16_APP_SEGMENT
    mov es, ax

    ; Проверяем магическое число и правильность режима
    cmp word [es:RLX16_APP_OFFSET], RLX_MAGIC
    jne .invalid_header
    cmp word [es:RLX16_APP_OFFSET + 2], RLX_MODE_16
    je .valid_header

    jmp .invalid_mode

.valid_header:
    ; Вычисление точки входа из файла
    mov bx, [es:RLX16_APP_OFFSET + 4]

    ; Сообщение о запуске программы
    mov si, msg_rlx_starting
    call print

    ; Расставляем флаги работы
    mov byte [cs:app16_running], 1

    ; Настройка сегментов с их сохранением (ax - RLX16_APP_SEGMENT)
    push ds
    push es
    
    mov ds, ax
    mov es, ax

    ; Сохраняем полный адрес возврата на стеке (для retf)
    push cs
    push .done

    ; Сохраняем полный адрес загрузки
    ; (ax - segment, bx - offset, для retf)
    push ax
    push bx
    retf

.done:
    ; Восстановление значений сегментов
    pop es
    pop ds

    ; Вывод сообщения о успешном завершении
    call print_new_line_if_needed
    mov si, msg_rlx_finished
    call print
    jmp .return

.fail:
    call print
    jmp .return

.invalid_header:
    mov si, err_rlx_header
    call print
    
    stc
    jmp .return

.invalid_mode:
    mov si, err_rlx_mode
    call print
    
    stc
    jmp .return

.return:
    pop ds
    pop es
    popa
    ret

; Строки и ошибки
err_exec_bad_name: db '[!] Invalid filename format for FAT12.', ENTER, 0
msg_rlx_starting:  db '[+ | RLX16 Loader] Executing 16-bit application...', ENTER, 0
msg_rlx_finished:  db '[+ | RLX16 Loader] Application finished.', ENTER, 0
err_rlx_header:    db '[! | RLX Loader] Invalid .RLX header magic!', ENTER, 0
err_rlx_mode:      db '[! | RLX Loader] Not a 16-bit .RLX binary!', ENTER, 0