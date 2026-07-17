; © Realix > Type Command
; (17.07.26) v0.1
; ================
; ❗️ Зависимости: kernel16/commands/load.asm (parse_and_load, file_size),
;                 kernel16/io: print & print_ctrl

; > Команда печати файла как текста: type <имя файла>
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_type:
    push ax
    push bx
    push cx
    push dx
    push si
    push di
    push es

    ; Разбор имени и загрузка файла
    call parse_and_load
    jc .fail

    ; Подготовка к чтению содержимого
    mov ax, FILE_DEST_SEGMENT
    mov es, ax
    xor si, si            ; (es:si) Адрес данных файла
    mov ecx, [file_size]  ; Счётчик остатка файла

.type_loop:
    ; Дошли до конца файла
    test ecx, ecx
    jz .done

    ; Печать очередного байта как символа
    mov al, [es:si]
    call print_char

    ; Продвижение по памяти с переносом сегмента на границе 64 КБ
    inc si
    jnz .next
    mov ax, es
    add ax, 0x1000
    mov es, ax

.next:
    ; Уменьшаем счётчик и идём дальше
    dec ecx
    jmp .type_loop

; Неудачное завершение (si - сообщение об ошибке от parse_and_load)
.fail:
    call print

.done:
    call print_new_line
    pop es
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret
