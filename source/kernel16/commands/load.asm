; © Realix > Command: Load
; (27.07.26) v0.1
; ================
; ❗️ Зависимости: bios-api/fat12/file_load.asm, boot_drive_num (main.asm),
;                 kernel16/io: print & print_reg

; Адрес назначения загружаемых файлов (свободная зона за регионом kernel16)
FILE_DEST_SEGMENT equ 0x2000
FILE_DEST_OFFSET  equ 0x0

; > Команда загрузки файла с диска: load <имя файла>
; Параметры:
;  - si: указатель на аргументы (после имени команды)
cmd_load:
    push ax
    push bx
    push cx
    push dx
    push di
    push si

    ; Разбор имени и загрузка файла в FILE_DEST_SEGMENT:0 (общий пролог)
    call parse_and_load
    jc .fail

    ; Логирование успеха с реальным адресом назначения
    mov si, msg_load_ok
    call print
    mov ax, FILE_DEST_SEGMENT
    call print_hex16
    mov al, ':'
    call print_char
    mov ax, FILE_DEST_OFFSET
    call print_hex16
    jmp .done

; Вывод сообщения (usage при пустом имени / ошибка file_load — адрес в si)
.fail:
    call print

.done:
    pop si
    pop di
    pop dx
    pop cx
    pop bx
    pop ax

    ret


; > Конвертация имени файла из ввода в формат FAT 8.3 (11 байт)
; Параметры:
;  - si: указатель на имя (до пробела или конца строки)
; Вывод:
;  - filename_83: буфер с именем (заглавные, паддинг пробелами)
;  - si: указывает за имя
;  - CF (Carry Flag): 0 (Успех), 1 (Некорректное имя)
format_83:
    push ax
    push cx
    push di

    ; Подготовка буфера
    mov di, filename_83
    mov cx, 11

.fill:
    ; Заполняем буфер пробелами вместо
    mov byte [di], ' '
    inc di
    loop .fill

    ; Считываем имя: до 8 символов, до '.', пробела или конца строки
    mov di, filename_83
    xor cx, cx           ; Счётчик символов имени

.name_loop:
    ; Загрузка и проверка символа имени
    mov al, [si]
    test al, al     ; 0 -> имя закончилось (дальше нет аргументов)
    jz .check_name
    cmp al, ' '     ; Пробел -> имя закончилось (дальше др. аргументы)
    je .check_name
    cmp al, '.'     ; Точка -> переходим к расширению файла
    je .ext_start

    ; Лимит имени - 8 символов
    cmp cx, 8
    jae .invalid

    ; Перевод в верхний регистр
    call char_to_upper

.name_store:
    ; Записываем символ в буфер, увеличивая смещения и счётчик
    mov [di], al
    inc di
    inc si
    inc cx
    jmp .name_loop

.ext_start:
    ; Проверка на пустое имя до расширения
    test cx, cx
    jz .invalid

    ; Считываем расширение: до 3 символов, после '.'
    inc si      ; Пропускаем '.'
    mov di, filename_83 + 8
    xor cx, cx  ; Счётчик символов расширения

.ext_loop:
    mov al, [si]
    test al, al
    jz .ok
    cmp al, ' '
    je .ok
    cmp al, '.'  ; Вторая точка - некорректное имя файла
    je .invalid

    ; Лимит расширения - 3 символа
    cmp cx, 3
    jae .invalid

    ; Перевод в верхний регистр
    call char_to_upper

.ext_store:
    mov [di], al
    inc di
    inc si
    inc cx
    jmp .ext_loop

.check_name:
    ; Проверка имени на пустоту (Маловероятно, но ок...)
    test cx, cx
    jz .invalid

.ok:
    clc
    jmp .return

.invalid:
    stc

.return:
    pop di
    pop cx
    pop ax
    ret

; > Разбор аргумента-имени и загрузка файла в `FILE_DEST_SEGMENT:FILE_DEST_OFFSET`
; (Общий пролог для команд, отображающих файл: type, hexdump и др.)
; Параметры:
;  - si: указатель на аргументы команды (после имени)
; Вывод:
;  - Файл загружен, размер в [file_size] (Если успех)
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка, si - сообщение для печати)
parse_and_load:
    ; Пропуск пробелов до аргумента
    call skip_spaces
    cmp byte [si], 0
    je .no_arg

    ; Конвертация имени в формат FAT 8.3
    call format_83
    jc .no_arg

    ; Загрузка файла с защитой региона работающего ядра
    mov si, filename_83
    mov cx, FILE_DEST_SEGMENT
    mov bx, FILE_DEST_OFFSET
    mov dl, [boot_drive_num]
    mov di, guard_kernel16
    call file_load          ; CF + si (сообщение) + [file_size]
    ret

.no_arg:
    mov si, msg_load_usage
    stc
    ret

; Буфер имени в формате 8.3
filename_83: times 11 db ' '

; Сообщения
msg_load_usage: db '[?] Usage: load <filename>', 0
msg_load_ok:    db '[+] File loaded at ', 0
