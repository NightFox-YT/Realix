; © Realix > Load Commands
; (16.07.26) v0.1
; ================
; ❗️ Зависимости: bios-api/fat12/file_load.asm,
;                 kernel16/io/print.asm, boot_drive_num (main.asm)

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

    ; Пропуск пробелов до аргумента
    call skip_spaces
    cmp byte [si], 0
    je .usage

    ; Конвертация имени в формат FAT 8.3 (11 байт)
    call format_83
    jc .usage

    ; Загрузка файла с защитой региона работающего ядра
    mov si, filename_83
    mov cx, FILE_DEST_SEGMENT
    mov bx, FILE_DEST_OFFSET
    mov dl, [boot_drive_num]
    mov di, guard_kernel16
    call file_load
    jc .fail

    ; Логирование успеха
    mov si, msg_load_ok
    call print
    jmp .done

; Вывод сообщения об ошибке (Возврат si из file_load)
.fail:
    call print
    jmp .done

.usage:
    mov si, msg_load_usage
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
;  - CF: 0 (успех), 1 (некорректное имя)
;  - si: указывает за имя
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

    ; Перевод в верхний регистр (a-z, другие не трогаем)
    cmp al, 'a'
    jb .name_store
    cmp al, 'z'
    ja .name_store
    sub al, 32

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

    ; Перевод в верхний регистр (a-z, другие не трогаем)
    cmp al, 'a'
    jb .ext_store
    cmp al, 'z'
    ja .ext_store
    sub al, 32

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

; Буфер имени в формате 8.3
filename_83: times 11 db ' '

; Сообщения
msg_load_usage: db '[?] Usage: load <filename>', 0
msg_load_ok:    db '[+] File loaded at 2000:0000', 0
