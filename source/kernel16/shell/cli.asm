; © Realix > Command Line Interface
; ø Вдохновлено @nyxmalware
; (13.06.26) v0.06
; ================
; ❗️ Зависимости: kernel16/io

; Основные константы
%include 'config.asm'

; > Главный цикл CLI (Вызывается из ядра для обработки команд)
run_cli:
    push si

.prompt:
    ; "Realix@User >> "
    mov si, prompt_sign
    call print

    call cli_input

    ; Если ввод пустой, заново ждём ввод
    cmp byte [input_buffer], 0
    je .prompt

    ; Выполнение команды
    mov si, input_buffer
    call execute_cmd

    ; Перевод строки на экране
    call print_new_line

    jnc .prompt

.done:
    pop si
    ret


; > Функция чтения строки
cli_input:
    push ax
    push bx
    push di

    ; Счётчик введённых символов и указатель адреса буффера
    xor bx, bx
    mov di, input_buffer

.input_loop:
    ; Ожидание нажатия (ASCII > al)
    xor ah, ah
    int 0x16

    ; ENTER > Проверка введённой строки
    cmp al, ENTER_KEY
    je .enter_pressed
    
    ; Backspace > Стирание последнего символа
    cmp al, BACKSPACE_KEY
    je .backspace_pressed

    ; Проверка на переполнение
    cmp bx, INPUT_BUFFER_LEN
    jae input_buffer_overflow_error

    ; Игнор управляющих символов
    cmp al, 0x20
    jb .input_loop

    ; Отображение символа на экране (Эхо)
    mov ah, 0x0E
    int 0x10

    ; Сохранение символа в буфер
    mov [di], al
    inc di
    inc bx

    ; Возвращаемся в поток ввода
    jmp .input_loop

.backspace_pressed:
    ; Проверка на пустой буффер
    test bx, bx
    jz .input_loop
    
    ; Стирание последнего символа из буфера
    dec di
    dec bx
    mov byte [di], 0

    ; Визуальное стирание символа на экране
    push bx
    mov ah, 0x0E
    xor bx, bx

    mov al, 0x08
    int 0x10
    mov al, ' '
    int 0x10
    mov al, 0x08
    int 0x10
    pop bx

    ; Возвращаемся в поток ввода
    jmp .input_loop

.enter_pressed:
    call print_new_line

    ; Закрываем строку нуль-терминатором
    mov byte [di], 0
    pop di
    pop bx
    pop ax
    ret


input_buffer_overflow_error:
    push si

    ; Вывод ошибки
    call print_new_line
    mov si, err_input_buffer_overflow
	call print

    ; Сброс текущего ввода
    xor bx, bx
    mov di, input_buffer
    mov byte [di], 0

    pop si
    jmp cli_input.enter_pressed


; Сообщения и строки
prompt_sign:               db 'Realix@User >> ', 0
err_input_buffer_overflow: db '[!] The input buffer maximum is 64 symbols!', 0

; Переменные
input_buffer: times 65 db 0