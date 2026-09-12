; © Realix > Command: Echo
; (16.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/io/print, kernel16/shell/utils


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