; © Realix > Shell: Commands Handler
; (16.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/memory, bios-api/io, kernel16/shell/parse
;                 kernel16/panic, display/memory
; TODO:
;  - Возвращение carry_flag при ошибке
;  - В shutdown полагаться не только на APM

; Основные константы
%include 'shared/config.asm'

; Таблица команд (С названиями и нуль-терминатором)
CMD_ENTRY_SIZE equ 4

align 2
cmd_table:
    dw .str_help,     cmd_help
    dw .str_cls,      clear_screen
    dw .str_clear,    clear_screen
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




; > Команда самостоятельного вызова паники
cmd_panic:
    call debug_panic_manual
    ret

; > Команды управления питанием
%include "kernel16/commands/power.asm"

; > Команда просмотра информации о памяти
%include "kernel16/commands/base/meminfo.asm"

; > Команда вывода строки на экран
%include "kernel16/commands/base/echo.asm"

; > Команда просмотра информации о проекте
%include "kernel16/commands/base/about.asm"

; > Команды для работы с текстом
%include "kernel16/commands/text.asm"

; > Команды для работы с числами
%include "kernel16/commands/nums.asm"

; > Команда простого калькулятора
%include "kernel16/commands/calc.asm"

; > Команда загрузки файла с диска
%include "kernel16/commands/filesystem/load.asm"

; > Команда загрузки RLX приложения с диска
%include "kernel16/commands/filesystem/exec.asm"

; > Команда вывода списка файлов
%include "kernel16/commands/filesystem/ls.asm"

; > Команда печати файла как текста
%include "kernel16/commands/filesystem/type.asm"

; > Команда шестнадцатеричного дампа файла
%include "kernel16/commands/filesystem/hexdump.asm"

; > Команда вывода снимка регистров
%include "kernel16/commands/debug/regs.asm"

; > Команды вывода времени и даты (RTC)
%include "kernel16/commands/debug/date-time.asm"

; > Команда демонстрации графического режима
%include "kernel16/commands/debug/vga.asm"

; > Команды для вывода ascii/scancode клавиши
%include "kernel16/commands/debug/key.asm"

; > Команды для вывода системной информации
%include "kernel16/commands/base/sysinfo.asm"

; > Команды для вывода аптайма
%include "kernel16/commands/base/uptime.asm"

; > Команда помощи
%include "kernel16/commands/base/help.asm"

; Подключение доп. модуля для команд
%include "kernel16/shell/utils.asm"


; Сообщение о неизвестной команде
err_unknown_cmd: db "[!] Unknown command. Type 'help' for list of commands.", 0
