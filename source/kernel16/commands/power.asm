; © Realix > Commands: Power category
; (16.08.26) v0.12
; ================
; ❗️ Зависимости: kernel16/shell/commands, bios-api/io: print & print_reg

; > Команда перезагрузки
cmd_reboot:
    cli

    ; Аппаратный сброс процессора через вектор BIOS
    jmp 0xFFFF:0x0000


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


; Сообщения об ошибками
err_shutdown:    db '[!] E8: PC shutdown failed! (No APM)', 0