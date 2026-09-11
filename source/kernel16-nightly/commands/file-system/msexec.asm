; © Realix > Command: msexec (.COM MS-DOS loader)
; ø Copyright by @Ramix
; (08.09.26) v0.12 [Nightly]
; ================
; ❗️ Зависимости: kernel16-nightly/shell (format_83, skip_spaces), bios-api/fat12/file_load,
;                 dos/psp.asm, dos/int21.asm
; ❗️ Поддерживает только .COM-модель памяти (Код+данные+стек в одном 64 КБ сегменте,
;    точка входа - offset 0x100). .EXE (Со своим заголовком/релокациями) не поддерживается.

; Настройка компиляции
bits 16

; Основные константы
%include 'shared/config.asm'

; > Команда запуска .COM-приложения MS-DOS: msexec <имя файла> [аргументы]
; Параметры:
;  - si: указатель на аргументы (после имени команды)
; ❗️ Общие точки выхода (msexec_fail/usage/return) объявлены обычными (не dot-local)
;    метками - на msexec_return переходит и dos_terminate_return (int21.asm), а
;    dot-локальные метки были бы недоступны извне области видимости cmd_msexec
cmd_msexec:
    pusha
    push es
    push ds

    ; Пропуск пробелов, проверка на пустой аргумент
    call skip_spaces
    cmp byte [si], 0
    je msexec_usage

    ; Конвертация имени в формат FAT 8.3 (si -> остаток строки после имени)
    call format_83
    jc msexec_usage

    ; Сохраняем оставшуюся часть командной строки (Аргументы для программы)
    call skip_spaces
    mov [cmdline_ptr], si

    ; Загрузка .COM-файла в DOS_PSP_SEGMENT:DOS_COM_ENTRY_OFFSET (Сразу после PSP)
    mov si, filename_83
    mov cx, DOS_PSP_SEGMENT
    mov bx, DOS_COM_ENTRY_OFFSET
    mov di, guard_kernel16
    call file_load
    jc msexec_fail

    ; Построение PSP (es = DOS_PSP_SEGMENT)
    mov ax, DOS_PSP_SEGMENT
    mov es, ax
    mov si, [cmdline_ptr]
    call dos_build_psp

    ; Сообщение о запуске программы
    mov si, msg_msexec_starting
    call print

    ; Сохраняем стек ядра - dos_terminate вернёт нас именно сюда
    mov [dos_saved_ss], ss
    mov [dos_saved_sp], sp

    ; Настройка сегментов и свежего стека программы (Модель памяти .COM: ds=es=ss=cs)
    mov ax, DOS_PSP_SEGMENT
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0xFFFE

    ; Дальний переход в точку входа .COM-программы (Без возврата обычным способом -
    ; см. dos_terminate_return, куда прыгает dos_terminate при int 20h/21h ah=00h/4Ch)
    jmp DOS_PSP_SEGMENT:DOS_COM_ENTRY_OFFSET

; > Точка возврата после завершения .COM-программы (Прыжок из dos_terminate, int21.asm)
; ❗️ ds/ss/sp к этому моменту уже восстановлены в dos_terminate
dos_terminate_return:
    call print_new_line_if_needed
    mov si, msg_msexec_finished
    call print
    jmp msexec_return

msexec_fail:
    call print
    jmp msexec_return

msexec_usage:
    mov si, msg_msexec_usage
    call print

msexec_return:
    pop ds
    pop es
    popa
    ret

; Переменные
cmdline_ptr: dw 0

; Строки
msg_msexec_usage:    db '[?] Usage: msexec <filename.com> [args]', ENTER, 0
msg_msexec_starting: db '[+ | MSEXEC] Executing MS-DOS .COM application...', ENTER, 0
msg_msexec_finished: db '[+ | MSEXEC] Application terminated.', ENTER, 0
