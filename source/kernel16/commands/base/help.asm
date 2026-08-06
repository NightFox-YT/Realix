; © Realix > Command: Help
; (28.07.26) v0.11
; ================
; ❗️ Зависимости: kernel16/io/print.asm

; > Команда помощи
cmd_help:
    push si

    mov si, msg_help
    call print

    pop si
    ret

; > Сообщение
msg_help:
    db 'Commands:', ENTER
    db '[Base]                                 [Text]', ENTER
    db '> help      - Show this manual        > len <t>        - Length of <t>', ENTER
    db '> clear/cls - Clear screen            > upper <t>      - <t> to upper case', ENTER
    db '> echo <t>  - Print <t> to console    > lower <t>      - <t> to lower case', ENTER
    db '> about     - Show info about Realix  > reverse <t>    - Reverse <t>', ENTER
    db '> beep      - Beep via BIOS speaker   > repeat <n> <t> - Repeat <t> <n> times', ENTER
    db '> meminfo   - Memory information      > ascii <0-255>  - Char by ASCII code', ENTER
    db '> uptime    - Show uptime (seconds)', ENTER
    db '> sysinfo   - System information', ENTER
    db ENTER
    db '[Numbers]                              [Fat12]', ENTER
    db '> calc <a> <+ - * /> <b> - Calculator > ls          - List root directory', ENTER
    db '> hex <num>  - <num> to hexadecimal   > load <f>    - Load file into RAM', ENTER
    db '> fib <0-24> - Nth Fibonacci number   > type <f>    - Print file as text', ENTER
    db '                                      > hexdump <f> - Hex dump of file', ENTER
    db ENTER
    db '[System]                               [Power]', ENTER
    db '> regs  - Show CPU registers snapshot > reboot   - Reboot PC', ENTER
    db '> time  - Show RTC time               > shutdown - Power off PC', ENTER
    db '> date  - Show RTC date', ENTER
    db '> vga   - VGA 320x200 graphics demo    [Tools]', ENTER
    db '> panic - Show panic screen and halt  > key - Show pressed key codes', 0