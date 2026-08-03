; © Realix > Command: Hexdump
; (17.07.26) v0.1
; ================
; ❗️ Зависимости: kernel16/commands/load.asm (parse_and_load, file_size),
;                 kernel16/io: print, print_ctrl, print_reg (print_byte, print_hex16)

; Кол-во байт в одной строке дампа
HEXDUMP_ROW_WIDTH equ 16

; > Команда шестнадцатеричного дампа файла: hexdump <имя файла>
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_hexdump:
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
    xor bx, bx            ; Смещение для метки
    mov ecx, [file_size]  ; Счётчик остатка файла

.row:
    ; Дошли до конца файла
    test ecx, ecx
    jz .done

    ; Метка смещения строки: "XXXX: "
    mov ax, bx
    call print_hex16
    mov al, ':'
    call print_char
    mov al, ' '
    call print_char

    ; Счётчик байт в текущей строке
    mov dl, HEXDUMP_ROW_WIDTH

.next_byte:
    ; Печать байта в виде "XX"
    mov al, [es:si]
    call print_byte

    ; Продвижение по памяти с переносом сегмента на границе 64 КБ
    inc si
    jnz .advance
    mov ax, es
    add ax, 0x1000
    mov es, ax

.advance:
    ; Переход к след. байту
    inc bx
    dec ecx
    jz .newline  ; Проверка, что файл не кончился посреди строки

    ; Обновление счётчика
    dec dl
    jnz .next_byte

.newline:
    call print_new_line
    jmp .row

; Неудачное завершение (si - сообщение об ошибке от parse_and_load)
.fail:
    call print
    call print_new_line

.done:
    pop es
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret
