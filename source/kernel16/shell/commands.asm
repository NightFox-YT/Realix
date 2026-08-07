; © Realix > Shell: Commands
; ø Вдохновлено @nyxmalware
; (06.08.26) v0.11
; ================
; ❗️ Зависимости: bios-api/memory, kernel16/io, kernel16/shell/parse
;                 kernel16/debug/panic.asm
; TODO:
;  - Возвращение carry_flag при ошибке
;  - В shutdown полагаться не только на APM

; Таблица команд (С названиями и нуль-терминатором)
CMD_ENTRY_SIZE equ 4

align 2
cmd_table:
    dw .str_help,     cmd_help
    dw .str_cls,      cmd_cls
    dw .str_clear,    cmd_cls
    dw .str_reboot,   cmd_reboot
    dw .str_shutdown, cmd_shutdown
    dw .str_meminfo,  cmd_meminfo
    dw .str_echo,     cmd_echo
    dw .str_calc,     cmd_calc
    dw .str_exec,     cmd_exec16
    dw .str_beep,     print_beep_char
    dw .str_about,    cmd_about
    dw .str_reverse,  cmd_reverse
    dw .str_len,      cmd_len
    dw .str_upper,    cmd_upper
    dw .str_lower,    cmd_lower
    dw .str_hex,      cmd_hex
    dw .str_ascii,    cmd_ascii
    dw .str_repeat,   cmd_repeat
    dw .str_fib,      cmd_fib
    dw .str_ls,       cmd_ls
    dw .str_load,     cmd_load
    dw .str_type,     cmd_type
    dw .str_hexdump,  cmd_hexdump
    dw .str_regs,     cmd_regs
    dw .str_time,     cmd_time
    dw .str_date,     cmd_date
    dw .str_vga,      cmd_vga
    dw .str_key,      cmd_key
    dw .str_sysinfo,  cmd_sysinfo
    dw .str_panic,    cmd_panic
    dw .str_uptime,   cmd_uptime
    dw 0

.str_help:     db 'help', 0
.str_cls:      db 'cls', 0
.str_clear:    db 'clear', 0
.str_reboot:   db 'reboot', 0
.str_shutdown: db 'shutdown', 0
.str_meminfo:  db 'meminfo', 0
.str_echo:     db 'echo', 0
.str_calc:     db 'calc', 0
.str_beep:     db 'beep', 0
.str_about:    db 'about', 0
.str_reverse:  db 'reverse', 0
.str_len:      db 'len', 0
.str_upper:    db 'upper', 0
.str_lower:    db 'lower', 0
.str_hex:      db 'hex', 0
.str_ascii:    db 'ascii', 0
.str_repeat:   db 'repeat', 0
.str_fib:      db 'fib', 0
.str_load:     db 'load', 0
.str_ls:       db 'ls', 0
.str_type:     db 'type', 0
.str_hexdump:  db 'hexdump', 0
.str_regs:     db 'regs', 0
.str_time:     db 'time', 0
.str_date:     db 'date', 0
.str_vga:      db 'vga', 0
.str_key:      db 'key', 0
.str_sysinfo:  db 'sysinfo', 0
.str_panic:    db 'panic', 0
.str_uptime:   db 'uptime', 0
.str_exec:     db 'exec', 0

; > Исполнитель команд
; Параметры:
;  - si: указатель на введенную строку
execute_cmd:
    push si
    push di
    push ax
    push bx

.skip_spaces:
    ; Загрузка символа из si в al (Проверка на пробел)
    lodsb
    cmp al, ' '
    je .skip_spaces

    ; si указывает на первый символ команды
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

    ; *Сохраняем указатель на начало ввода пользователя
    push si

.compare_loop:
    ; Загрузка символа из ввода и таблицы
    mov al, [si]
    mov ah, [bx]

    ; Имя команды в таблице закончилась (\0)?
    test ah, ah
    jz .check_match

    ; Перевод символа ввода к нижнему регистру
    call char_to_lower

    ; Сравнение символов ввода и таблицы
    cmp al, ah
    jne .mismatch

    ; Переход к след. символу
    inc si
    inc bx
    jmp .compare_loop

.check_match:
    ; Проверка, след. символа после названия
    cmp al, 0
    je .match
    cmp al, ' '
    je .match

.mismatch:
    pop si                  ; *Восстанавливаем указатель для проверки след.
    add di, CMD_ENTRY_SIZE  ; Сдвигаем на следующую запись
    jmp .search_next

.match:
    ; Убираем сохранённый ввод со стека (si уже указывает на аргументы)
    add sp, 2

    ; Вызываем обработчик команды по адресу из таблицы
    call word [di + 2]
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

; > Команда перезагрузки
cmd_reboot:
    cli

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0x0000

; > Команда вывода информации о памяти
cmd_meminfo:
    push si

    ; Показ строк с информацией о памяти
    call show_lower_memory
    call print_new_line

    call show_usable_memory
    call print_new_line

    call show_map_entries_cnt
    call print_new_line
    call print_new_line

    ; Небольшая заметка
    mov si, note_meminfo
    call print

    pop si
    ret

; > Команда выключения ПК (через APM)
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
    ; Ошибка 8: Не удалось выключить ПК
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

    ; Проверка существования текстового аргумента
    call require_arg
    jc .done

    ; Вывод сообщения
    call print

.done:
    pop si
    pop ax
    ret

; > Команда "About"
cmd_about:
    push si

    mov si, msg_about
    call print

    pop si
    ret

; > Команда самостоятельного вызова паники
cmd_panic:
    call debug_panic_manual
    ret

; > Команда очистки экрана
%include "kernel16/commands/base/cls.asm"

; > Команды для работы с текстом
%include "kernel16/commands/text.asm"

; > Команды для работы с числами
%include "kernel16/commands/nums.asm"

; > Команда простого калькулятора
%include "kernel16/commands/calc.asm"

; > Команда загрузки файла с диска
%include "kernel16/commands/file-system/load.asm"

; > Команда загрузки RLX приложения с диска
%include "kernel16/commands/file-system/exec.asm"

; > Команда вывода списка файлов
%include "kernel16/commands/file-system/ls.asm"

; > Команда печати файла как текста
%include "kernel16/commands/file-system/type.asm"

; > Команда шестнадцатеричного дампа файла
%include "kernel16/commands/file-system/hexdump.asm"

; > Команда вывода снимка регистров
%include "kernel16/commands/debug/regs.asm"

; > Команды вывода времени и даты (RTC)
%include "kernel16/commands/rtc.asm"

; > Команда демонстрации графического режима
%include "kernel16/commands/vga.asm"

; > Команды для вывода ascii/scancode клавиши
%include "kernel16/commands/debug/key.asm"

; > Команды для вывода системной информации
%include "kernel16/commands/debug/sysinfo.asm"

; > Команды для вывода аптайма
%include "kernel16/commands/debug/uptime.asm"

; > Команда помощи
%include "kernel16/commands/base/help.asm"

; > Перевод символа al в верхний регистр (a-z -> A-Z, иначе без изменений)
; Параметры & Вывод:
;  - al: символ (Любой + a-z)
char_to_upper:
    cmp al, 'a'
    jb .done
    cmp al, 'z'
    ja .done
    sub al, 32
.done:
    ret

; > Перевод символа al в нижний регистр (A-Z -> a-z, иначе без изменений)
; Параметры & Вывод:
;  - al: символ (Любой + A-Z)
char_to_lower:
    cmp al, 'A'
    jb .done
    cmp al, 'Z'
    ja .done
    add al, 32
.done:
    ret

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

; > Разбор единственного числового аргумента команды
; (Пропускает пробелы, читает число и требует пустой хвост)
; Параметры:
;  - si: указатель на аргументы команды
; Вывод:
;  - ax: число (uint16)
;  - CF (Carry Flag): 0 (Успех), 1 (Пусто / Не число / Мусор после числа)
parse_uint16_arg:
    ; Пропуск пробелов до аргумента
    call skip_spaces
    cmp byte [si], 0
    je .fail

    ; Парсинг числа
    call parse_uint16
    jc .fail

    ; После числа допустимы только пробелы
    push ax
    call skip_spaces
    cmp byte [si], 0
    pop ax
    jne .fail

.done:
    clc
    ret

.fail:
    stc
    ret

; > Пропуск пробелов с проверкой, что аргумент не пуст
; Параметры:
;  - si: указатель на аргументы команды
; Вывод:
;  - si: указатель на первый символ аргумента
;  - CF (Carry Flag): 0 (Аргумент есть), 1 (Аргумент пуст)
require_arg:
    call skip_spaces
    cmp byte [si], 0
    je .fail

.done:
    clc
    ret

.fail:
    stc
    ret

; Сообщения об ошибках
err_unknown_cmd: db "[!] Unknown command. Type 'help' for list of commands.", 0
err_shutdown:    db '[!] E8: PC shutdown failed! (No APM)', 0

; Сообщения
msg_about:
    db '> Realix version: ', OS_VERSION, ENTER
    db 'Realix is a lightweight hybrid x86 OS.', ENTER
    db 'It supports a built-in boot switcher that lets users choose:', ENTER
    db '1. 16-bit Real Mode kernel for legacy compatibility', ENTER
    db '2. 32-bit Protected Mode kernel for high performance.', 0

note_meminfo: db 'Note: In Real mode you can access only low RAM.', 0
