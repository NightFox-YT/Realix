; © Realix > Input
; (05.04.26) v0.06
; ================
; Зависимости: kernel/print.asm

; Символы
%define ENTER 0x0D, 0x0A

; > Функция ввода команды
; Вывод:
;  - Возвращает управление
input:
    push di
    push ax
    push si

    xor di, di

.input_loop:
    ; Ожидание нажатия
	mov ah, 0x0
    int 0x16

    ; ENTER > Переходим к проверке введённой строки
	cmp al, 0x0D
    je .check_input

    ; Backspace > Переходим к функции стирания последнего символа
    cmp al, 0x8
    je .backspace

	; Вывод введённого символа
	mov ah, 0x0E
	int 0x10

    ; Сохраняем символ в input_str
    mov [input_str + di], al
    inc di

    ; Проверяем на переполнение input_str
    cmp di, 64
    je .input_str_error

    ; Возвращаемся в поток ввода
    jmp .input_loop

; Выполнение команды введённой строки
.check_input:
    ; Переход на новую строку
    mov si, input_new_line
    call print

    ; Поиск команды по таблице
    mov si, cmd_table

; Цикл поиска команды
.search_cmd:
    ; Проверка указателя (bx) на 0 (Конец таблицы)
    mov bx, [si]
    test bx, bx
    jz .wrong_command

    ; Сравниваем с текущей командой, сохраняя позицию в таблице
    push si
    mov si, bx
    call str_compare
    pop si

    ; Проверка на совпадение команды
    cmp dx, 1
    je .dispatch

    ; Переход к след. записи в таблице
    add si, 4
    jmp .search_cmd

; Обработка найденной команды
.dispatch:
    call [si + 2]
    jmp .done

.help_command:
    ; Сообщение-справочник
    mov si, help_output
    call print

    ret

.clear_command:
    ; Очистка экрана
    mov ah, 0x00
    mov al, 0x03
    int 0x10

    ret

.reboot_command:
    jmp 0FFFFh:0

.shutdown_command:
    ; Подключение к APM:
    ; > ax - APM функция с подключением реального режима
    ; > bx - устройство: "System BIOS"
    mov ax, 0x5301
    xor bx, bx
    int 0x15

    ; Установка версии APM
    ; > al - выбор версии
    ; > bx - устройство: "System BIOS"
    ; > cx - запрашиваемая версия APM 1.2
    mov ax, 0x530E
    xor bx, bx
    mov cx, 0x0102
    int 0x15

    ; Команда выключения питания
    ; > al - установка состояния питания
    ; > bx - устройство: "All devices" (все устройства)
    ; > cx - состояние: "Off" (выключить)
    mov ax, 0x5307
    mov bx, 0x0001
    mov cx, 0x0003
    int 0x15

    jmp .shutdown_error

.wrong_command:
    ; Ошибка: "Неизвестная команда"
    mov si, err_wrong_cmd
    call print

    jmp input.done

.backspace:
    ; Проверяем на пустой input_str
    cmp di, 0
    je .input_loop    

    ; Стираем последний символ
    mov si, cls_char
    call print

    ; Убираем из input_str символ, который стёрли
    mov byte [input_str + di], 0
    dec di

    ; Возвращаемся в поток ввода
    jmp .input_loop

; Ошибка переполнения input_str
.input_str_error:
    mov si, err_input_str_max
	call print

    jmp .done

; Ошибка выключения ПК
.shutdown_error:
    mov si, err_shutdown
	call print

    jmp .done

; Выход
.done:
    call clear_input  ; Очищаем input_str

    pop di
    pop ax
    pop si
    ret


; > Очищение введённой строки
; Вывод:
;  - di: 0
clear_input:
    xor di, di

.clear_loop:
    ; Проверяем, что мы дошли до пустых символов
    cmp byte [input_str + di], 0
    je .done

    ; Иначе проверяем, что мы дошли до конца input_str
    cmp di, 64
    je .done

    ; Очищаем символ
    mov byte [input_str + di], 0
    inc di

    jmp .clear_loop

.done:
    ret


; > Сравнение ввода пользователя с командой
; Параметры (Регистры):
;  - si: Команда для сравнения
; Вывод:
;  - dx: Результат сравнения
str_compare:
    push si
    push di
    push ax

    ; Сохраняем в di адрес нахождения ввода пользователя
    mov di, input_str

.compare_loop:
    mov ah, [di]
    cmp [si], ah
    jne .not_equal

    ; Проверка на конец команды сравнения
    cmp byte [si], 0
    je .zero

    ; Переход к след. символу
    inc si
    inc di

    jmp .compare_loop

.zero:
    ; Проверка, что строка ввода тоже был завершёна
    cmp byte [di], 0
    jne .not_equal

    ; Возвращаем результат
    mov dx, 1
    jmp .done

.not_equal:
    ; Возвращаем результат
    mov dx, 0
    jmp .done

.done:
    pop ax      
    pop di
    pop si

    ret


; Сообщения
help_output:
    db ENTER, 'Realix - Help:', ENTER, ENTER
    db '  [ Base ]  ', ENTER
    db '> clear/cls - Clear console.', ENTER
    db '> help - List of commands.', ENTER
    db '  [ Power ]  ', ENTER
    db '> reboot - Reboot PC.', ENTER
    db '> shutdown - Power off PC.', ENTER, 0

; Команды
cmd_table:
    dw cmd_help,         input.help_command
    dw cmd_clear,        input.clear_command
    dw cmd_clear_alias,  input.clear_command
    dw cmd_reboot,       input.reboot_command
    dw cmd_shutdown,     input.shutdown_command
    dw 0

cmd_help:        db 'help', 0
cmd_clear:       db 'clear', 0
cmd_clear_alias: db 'cls', 0
cmd_reboot:      db 'reboot', 0
cmd_shutdown:    db 'shutdown', 0

; Ошибки
err_wrong_cmd:     db '[!] Unknown Command, write help for list of commands', ENTER, 0
err_input_str_max: db ENTER, '[!] The buffer maximum is 64 symbols!', ENTER, 0
err_shutdown:      db '[!] Critical error! PC did not shutdown...', ENTER, 0

; Вспомогательные строки
cls_char: db 0x8, ' ', 0x8, 0
input_new_line: db ENTER, 0

; Буфер ввода
input_str: times 64 db 0