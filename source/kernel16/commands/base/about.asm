; © Realix > Command: About
; (16.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/io/print


; > Команда "About"
cmd_about:
    push si

    mov si, msg_about
    call print

    pop si
    ret


; Сообщение
msg_about:
    db '> Realix version: ', OS_VERSION, ENTER
    db 'Realix is a lightweight hybrid x86 OS.', ENTER
    db 'It supports a built-in boot switcher that lets users choose:', ENTER
    db '1. 16-bit Real Mode kernel for legacy compatibility', ENTER
    db '2. 32-bit Protected Mode kernel for high performance.', 0