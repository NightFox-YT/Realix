; © Realix > Shell Commands
<<<<<<< HEAD
; (13.06.26) v0.06
; ø Вдохновлено @nyxmalware
; ================
; ❗️ Зависимости: bios-api/memory (модуль)
; TODO: Возвращение carry_flag при ошибке
=======
; ø Вдохновлено @nyxmalware
; (13.06.26) v0.06
; ================
; ❗️ Зависимости: bios-api/memory (модуль), bios-api/network
; TODO:
;  - Возвращение carry_flag при ошибке
;  - В shutdown полагаться не только на APM
>>>>>>> development

; Таблица команд (С названиями)
align 2
cmd_table:
    dw .str_help,     cmd_help
    dw .str_cls,      cmd_cls
    dw .str_clear,    cmd_cls
    dw .str_reboot,   cmd_reboot
    dw .str_shutdown, cmd_shutdown
    dw .str_meminfo,  cmd_meminfo
    dw .str_echo,     cmd_echo
<<<<<<< HEAD
=======
    dw .str_calc,     cmd_calc
>>>>>>> development
    dw 0, 0

.str_help:     db 'help', 0
.str_cls:      db 'cls', 0
.str_clear:    db 'clear', 0
.str_reboot:   db 'reboot', 0
.str_shutdown: db 'shutdown', 0
.str_meminfo:  db 'meminfo', 0
.str_echo:     db 'echo', 0
<<<<<<< HEAD
=======
.str_calc:     db 'calc', 0
>>>>>>> development


; > Исполнитель команд
; Параметры:
;  - si: указатель на введенную строку
execute_cmd:
    push si
    push di
    push ax
    push bx

.skip_spaces:
    lodsb
    cmp al, ' '
    je .skip_spaces

    ; SI указывает на первый символ команды
    dec si

    ; Проверка на пустую строку после пробелов
    cmp byte [si], 0
    je .done

    ; Инициализируем проход по таблице команд
    mov di, cmd_table

.search_next:
    mov bx, [di]   ; Получаем имя команды по адресу
    test bx, bx    ; Проверяем имя команды на конец таблицы (0)
    jz .not_found
    
    ; Сохраняем указатель на начало ввода пользователя
    push si

.compare_loop:
    ; Загрузка символа из ввода и таблицы
    mov al, [si]
    mov ah, [bx]

    ; Имя команды в таблице закончилась (\0)?
    test ah, ah
    jz .check_match

    ; Сравнение символов ввода и таблицы
    cmp al, ah
    jne .mismatch

    ; Переход к след. символу
    inc si
    inc bx
    jmp .compare_loop

.check_match:
    ; Проверка, что во вводе дальше
    cmp al, 0
    je .match
    cmp al, ' '
    je .match

.mismatch:
    pop si            ; *Восстанавливаем для проверки след. команды
    add di, 4         ; Сдвигаем на следующую запись
    jmp .search_next

.match:
    pop ax            ; *Восстанавливаем push si из стека
    mov ax, [di + 2]  ; Берем адрес функции
    call ax           ; Вызываем функцию команды
    jmp .done

.not_found:
    mov si, err_unknown_cmd
    call print

.done:
    pop bx
    pop ax
    pop di
    pop si
    ret


; > Команда помощи
cmd_help:
    push si

    mov si, msg_help
    call print

    pop si
    ret

; > Команда очистки экрана
%include "kernel16/shell/cmd_cls.asm"

; > Команда перезагрузки
cmd_reboot:
    cli

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0x0000

; > Команда вывода информации о памяти
cmd_meminfo:
    push si
    push di

    ; Показ строк с информацией о памяти
    mov di, PCINFO_ADDR
    call show_lower_memory
    call print_new_line

    mov di, PCINFO_ADDR
    call show_free_memory
    call print_new_line

    mov di, PCINFO_ADDR
    call show_map_entries_cnt

<<<<<<< HEAD
=======
    call print_new_line
    call print_new_line

    ; Небольшая заметка
    mov si, note_meminfo
    call print

>>>>>>> development
    pop si
    pop di
    ret

cmd_shutdown:
    push ax
    push bx
    push cx
    push si

    ; Подключение к APM:
    ; > ax - APM функция с подключением реального режима
    ; > bx - устройство: "System BIOS"
    mov ax, 0x5301
    xor bx, bx
    int 0x15
    jc .error

    ; Установка версии APM
    ; > al - выбор версии
    ; > bx - устройство: "System BIOS"
    ; > cx - запрашиваемая версия APM 1.2
    mov ax, 0x530E
    xor bx, bx
    mov cx, 0x0102
    int 0x15
    jc .error

    ; Команда выключения питания
    ; > al - установка состояния питания
    ; > bx - устройство: "All devices" (все устройства)
    ; > cx - состояние: "Off" (выключить)
    mov ax, 0x5307
    mov bx, 0x0001
    mov cx, 0x0003
    int 0x15
    jc .error
    
    jmp $

.error:
    mov si, err_shutdown
    call print
    
    pop si
    pop cx
    pop bx
    pop ax
    ret

; > Команда "Echo"
; Параметры:
;  - si: указатель на начало аргументов команды
cmd_echo:
    push ax
    push si

<<<<<<< HEAD
.skip_spaces:
    lodsb
    cmp al, ' '
    je .skip_spaces

    ; Если аргументов нет (сразу конец строки)
    test al, al
    jz .done

    ; Возвращаем si назад на первый символ аргументов, выводим
    dec si                 
=======
    ; Пропускаем пробелы
    call skip_spaces

    ; Если аргументов нет (сразу конец строки)
    cmp byte [si], 0
    jz .done

    ; Вывод сообщения         
>>>>>>> development
    call print

.done:
    pop si
    pop ax

    ret

<<<<<<< HEAD
=======
; > Команда простого калькулятора
%include "kernel16/shell/cmd_calc.asm"

; > Пропуск пробелов до первого символа
; Вывод:
;  - si: указатель на первый символ строки
skip_spaces:
    ; Пропуск пробела c переходом к след. символу
    lodsb
    cmp al, ' '
    je skip_spaces

.done:
    ; Возвращаем si назад на первый символ строки
    dec si
    ret

>>>>>>> development
err_unknown_cmd: db '[!] Unknown command, write help for list of commands.', 0
err_shutdown:    db '[!] PC shutdown failed! (No APM)', 0

; Сообщения
msg_help:
<<<<<<< HEAD
    db 'Realix - Help:', ENTER
    db '  [Base]', ENTER
    db '> clear/cls - Clear screen', ENTER
    db '> help - Show this manual', ENTER
    db '> echo [text] - Print [text] to console', ENTER
    db '> meminfo - Display RAM configuration', ENTER
    db '  [Power]', ENTER
    db '> reboot - Reboot PC', ENTER
    db '> shutdown - Power off PC', 0

=======
    db 'Commands:', ENTER
    db '  [Base]', ENTER
    db '> help      - Show this manual', ENTER
    db '> clear/cls - Clear screen', ENTER
    db '> echo [t]  - Print [text] to console', ENTER
    db '> meminfo   - Display RAM configuration', ENTER
    db '> calc [num1] [+ - * /] [num2] - Simple Calculator (Only positive nums)', ENTER
    db '  [Power]', ENTER
    db '> reboot   - Reboot PC', ENTER
    db '> shutdown - Power off PC', 0

note_meminfo: db 'Note: In Real mode you can access only up to 1 MB RAM.', 0

>>>>>>> development
; Буфер ввода
input_str: times 64 db 0
