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
    db '  [Base]                               [Text]', ENTER
    db '> help      - Show this manual        > len <t>        - Length of <t>', ENTER
    db '> clear/cls - Clear screen            > upper <t>      - <t> to upper case', ENTER
    db '> echo <t>  - Print <t> to console    > lower <t>      - <t> to lower case', ENTER
    db '> about     - Show info about Realix  > reverse <t>    - Reverse <t>', ENTER
    db '> beep      - Beep via BIOS speaker   > repeat <n> <t> - Repeat <t> <n> times', ENTER
    db '> meminfo   - Memory information      > ascii <0-255>  - Char by ASCII code', ENTER
    db '> uptime    - Show uptime (seconds)', ENTER
    db '> sysinfo   - System information', ENTER
    db ENTER
    db '  [System]                              [Fat12]', ENTER
    db '> regs  - Show CPU registers snapshot > ls          - List root directory', ENTER
    db '> time  - Show RTC time               > load <f>    - Load file into RAM', ENTER
    db '> date  - Show RTC date               > type <f>    - Print file as text', ENTER
    db '> vga   - VGA 320x200 graphics demo   > hexdump <f> - Hex dump of file', ENTER
    db '> panic - Show panic screen and halt  > exec <f>    - Run RLX application', ENTER
    db ENTER
    db '  [Numbers]                             [Power]', ENTER
    db '> calc <a> <+ - * /> <b> - Calculator > reboot   - Reboot PC', ENTER
    db '> hex <num>  - <num> to hexadecimal   > shutdown - Power off PC', ENTER
    db '> fib <0-24> - Nth Fibonacci number', ENTER
    db '  [Tools]', ENTER
    db '> key - Show pressed key codes', ENTER
    db '  [Dev]', ENTER
    db '> asm - Enter mini-assembler (blank line to finish)', ENTER
    db '> run - Execute the assembled code', 0