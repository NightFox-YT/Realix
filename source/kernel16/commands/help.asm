; © Realix > Command: Help
; (17.07.26) v0.1
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
    db '> about     - Show system info        > reverse <t>    - Reverse <t>', ENTER
    db '> beep      - Beep via BIOS speaker   > repeat <n> <t> - Repeat <t> <n> times', ENTER
    db '> meminfo   - Memory information      > ascii <0-255>  - Char by ASCII code', ENTER
    db ENTER
    db '[Numbers]                              [Fat12]', ENTER
    db '> calc <a> <+ - * /> <b> - Calculator > ls          - List root directory', ENTER
    db '> hex <num>  - <num> to hexadecimal   > load <f>    - Load file into RAM', ENTER
    db '> fib <0-24> - Nth Fibonacci number   > type <f>    - Print file as text', ENTER
    db '                                      > hexdump <f> - Hex dump of file',
    db ENTER
    db '[Power]', ENTER
    db '> reboot   - Reboot PC', ENTER
    db '> shutdown - Power off PC', 0