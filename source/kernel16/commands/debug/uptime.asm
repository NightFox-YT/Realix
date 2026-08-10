; © Realix > Command: Uptime
; (06.08.26) v0.11
; ================
; ❗️ Зависимости: kernel16/main.asm (last_tick_*)

; > Вычисление текущего аптайма в секундах при помощи таймера BIOS
cmd_uptime:
    push eax
    push ebx
    push cx
    push dx
    push si

    ; Выводим "заголовок команды"
    mov si, msg_uptime
    call print

    ; Получаем текущее кол-во тиков
    mov ah, 00h
    int 0x1A

    ; Считаем сколько прошло с момента выбора 
    sub cx, [init_tick_high]
    sub dx, [init_tick_low]

    ; Записываем результат в eax
    movzx eax, cx
    shl eax, 16
    mov ax, dx
    
    ; Перевод в секунды (Не точный...)
    mov ebx, 10
    mul ebx
    mov ebx, 182
    div ebx

    ; Выводим кол-во прошедших секунд
    call print_dec32

.done:
    pop si
    pop dx
    pop cx
    pop ebx
    pop eax
    ret

; Строки
msg_uptime: db 'Uptime seconds: ', 0
