; © Realix > Command: Mini-Assembler (asm / run)
; (15.08.26) v0.1
; ================
; ❗️ Зависимости: kernel16/shell: cli (input_buffer, ENTER/BACKSPACE/SPACE_KEY,
;                 visual_erase_char), commands (skip_spaces, char_to_lower,
;                 str_equal), parse (parse_uint16), kernel16/io: print & print_reg
; ❗️ 'asm' очищает буфер и собирает инструкции по одной за строку в RAM,
;    'run' выполняет собранный код. Регистры общего назначения обнуляются
;    перед запуском; push/pop должны быть сбалансированы - код исполняется
;    без песочницы (как настоящий DEBUG.COM), поэтому за стек отвечает автор.

; Буфер собранного кода: свободная зона за regionом load/exec (0x2000)
ASM_BUFFER_SEGMENT equ 0x3000
ASM_BUFFER_OFFSET   equ 0x0000
ASM_BUFFER_SIZE     equ 512

; Макс. длина одного токена (мнемоника / регистр / число вида 0xFFFF)
TOKEN_MAX equ 8

; Типы операндов
OPTYPE_NONE  equ 0
OPTYPE_REG16 equ 1
OPTYPE_REG8  equ 2
OPTYPE_IMM   equ 3

; ASCII-код Ctrl+C (одинаков для Ctrl+C и Ctrl+Shift+C через int 0x16 ah=0 -
; BIOS не различает Shift для управляющих символов, поэтому Shift проверяется
; отдельно через флаги клавиатуры в BDA)
CTRL_C_KEY equ 0x03

; Адрес байта флагов клавиатуры в BIOS Data Area (биты 0-1: Right/Left Shift)
BDA_KB_FLAGS_SEGMENT equ 0x40
BDA_KB_FLAGS_OFFSET  equ 0x17
BDA_SHIFT_MASK       equ 0x03

; > Команда входа в мини-ассемблер: asm
; Сбрасывает буфер и читает инструкции по одной, пока не введена пустая строка
cmd_asm:
    push ax
    push si

    mov word [asm_cursor], 0

    ; Сбрасываем зависшие в буфере BIOS клавиши (напр. авто-повтор Enter,
    ; которым была отправлена сама команда 'asm') - иначе первый A> может
    ; сразу "проглотить" их и завершиться, не дав ничего ввести
    call drain_keyboard_buffer

    mov si, msg_asm_intro
    call print

.loop:
    mov si, asm_prompt
    call print

    call asm_read_line
    jc .aborted

    ; Пустая строка - завершение ввода
    cmp byte [input_buffer], 0
    je .finish

    mov si, input_buffer
    call assemble_line
    jc .show_error

    ; Проверка переполнения буфера ассемблера
    mov ax, [asm_cursor]
    add ax, [emit_count]
    cmp ax, ASM_BUFFER_SIZE
    ja .buffer_full

    call store_emitted_bytes
    jmp .loop

.buffer_full:
    mov si, err_asm_full
    call print
    call print_new_line
    jmp .loop

.show_error:
    call print
    call print_new_line
    jmp .loop

.aborted:
    mov si, msg_asm_aborted
    call print
    mov ax, [asm_cursor]
    call print_dec16
    mov si, msg_asm_done2
    call print

    pop si
    pop ax
    ret

.finish:
    mov si, msg_asm_done
    call print
    mov ax, [asm_cursor]
    call print_dec16
    mov si, msg_asm_done2
    call print

    pop si
    pop ax
    ret


; > Команда запуска собранного кода: run
cmd_run:
    push ax
    push bx
    push cx
    push dx
    push si
    push di
    push es

    cmp word [asm_cursor], 0
    je .no_code

    mov si, msg_run_start
    call print
    mov ax, [asm_cursor]
    call print_dec16
    mov si, msg_run_start2
    call print

    ; Дописываем безопасный дальний возврат (retf) сразу за собранным кодом.
    ; Не увеличивает asm_cursor - следующий 'asm' перезапишет этот байт.
    mov ax, ASM_BUFFER_SEGMENT
    mov es, ax
    mov di, [asm_cursor]
    mov byte [es:di], 0xCB

    ; Предсказуемое стартовое состояние регистров (sp/bp стека не касаемся)
    xor ax, ax
    xor bx, bx
    xor cx, cx
    xor dx, dx
    xor si, si
    xor di, di

    ; "Дальний вызов" в буфер вручную через стек (см. exec.asm: тот же приём) -
    ; так парный retf в конце буфера корректно вернётся на .after
    push cs
    push .after
    push ASM_BUFFER_SEGMENT
    push ASM_BUFFER_OFFSET
    retf

.after:
    ; На случай, если код выполнил cli и не восстановил флаг
    sti

    call print_new_line_if_needed
    mov si, msg_run_done
    call print
    jmp .return

.no_code:
    mov si, err_run_empty
    call print

.return:
    pop es
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret


; > Копирует emit_buf в буфер ассемблера по текущему курсору,
; печатает байты (обратная связь), обновляет asm_cursor
store_emitted_bytes:
    push ax
    push cx
    push si
    push di
    push es

    mov ax, ASM_BUFFER_SEGMENT
    mov es, ax
    mov di, [asm_cursor]

    mov si, emit_buf
    mov cx, [emit_count]
    test cx, cx
    jz .done

.copy_loop:
    mov al, [si]
    mov [es:di], al

    call print_byte

    inc si
    inc di
    loop .copy_loop

    mov ax, [asm_cursor]
    add ax, [emit_count]
    mov [asm_cursor], ax

.done:
    call print_new_line

    pop es
    pop di
    pop si
    pop cx
    pop ax
    ret


; > Разбор и сборка одной строки инструкции в emit_buf/emit_count
; Параметры:
;  - si: указатель на строку (одна инструкция)
; Вывод:
;  - CF: 0 успех (байты в emit_buf, emit_count - кол-во), 1 ошибка (si - сообщение)
assemble_line:
    push ax
    push bx
    push cx
    push dx

    mov word [emit_count], 0

    call skip_spaces
    mov di, mnemonic_buf
    call read_token
    jc .err_unknown

    call skip_spaces
    mov di, operand1_buf
    call read_token
    call skip_operand_sep

    mov di, operand2_buf
    call read_token

    ; Разбор операндов в (тип, значение)
    mov si, operand1_buf
    call resolve_operand
    jc .err_operand
    mov [op1_type], al
    mov [op1_value], bx

    mov si, operand2_buf
    call resolve_operand
    jc .err_operand
    mov [op2_type], al
    mov [op2_value], bx

    ; Поиск обработчика по мнемонике
    mov si, mnemonic_buf

    mov di, str_mov
    call str_equal
    je .enc_mov
    mov di, str_add
    call str_equal
    je .enc_add
    mov di, str_sub
    call str_equal
    je .enc_sub
    mov di, str_push
    call str_equal
    je .enc_push
    mov di, str_pop
    call str_equal
    je .enc_pop
    mov di, str_int
    call str_equal
    je .enc_int
    mov di, str_nop
    call str_equal
    je .enc_nop
    mov di, str_hlt
    call str_equal
    je .enc_hlt
    mov di, str_cli
    call str_equal
    je .enc_cli
    mov di, str_sti
    call str_equal
    je .enc_sti

    jmp .err_unknown

.enc_mov:
    call encode_mov
    jmp .after

.enc_add:
    call encode_add
    jmp .after

.enc_sub:
    call encode_sub
    jmp .after

.enc_push:
    call encode_push
    jmp .after

.enc_pop:
    call encode_pop
    jmp .after

.enc_int:
    call encode_int
    jmp .after

.enc_nop:
    cmp byte [op1_type], OPTYPE_NONE
    jne .err_operand
    cmp byte [op2_type], OPTYPE_NONE
    jne .err_operand
    mov al, 0x90
    call emit_byte
    clc
    jmp .after

.enc_hlt:
    cmp byte [op1_type], OPTYPE_NONE
    jne .err_operand
    cmp byte [op2_type], OPTYPE_NONE
    jne .err_operand
    mov al, 0xF4
    call emit_byte
    clc
    jmp .after

.enc_cli:
    cmp byte [op1_type], OPTYPE_NONE
    jne .err_operand
    cmp byte [op2_type], OPTYPE_NONE
    jne .err_operand
    mov al, 0xFA
    call emit_byte
    clc
    jmp .after

.enc_sti:
    cmp byte [op1_type], OPTYPE_NONE
    jne .err_operand
    cmp byte [op2_type], OPTYPE_NONE
    jne .err_operand
    mov al, 0xFB
    call emit_byte
    clc
    jmp .after

.after:
    jc .err_operand
    clc
    jmp .return

.err_operand:
    mov si, err_bad_operand
    stc
    jmp .return

.err_unknown:
    mov si, err_unknown_mnemonic
    stc

.return:
    pop dx
    pop cx
    pop bx
    pop ax
    ret


; -------------------------------------------------------------------------
; Кодировщики инструкций (используют op1_type/op1_value, op2_type/op2_value)
; Вывод: CF 0 - успех (байты в emit_buf через emit_byte/emit_word), 1 - ошибка
; -------------------------------------------------------------------------

encode_mov:
    mov al, [op1_type]
    mov ah, [op2_type]

    cmp al, OPTYPE_REG16
    jne .try_reg8

    cmp ah, OPTYPE_REG16
    je .reg16_reg16
    cmp ah, OPTYPE_IMM
    je .reg16_imm
    jmp .bad

.try_reg8:
    cmp al, OPTYPE_REG8
    jne .bad

    cmp ah, OPTYPE_REG8
    je .reg8_reg8
    cmp ah, OPTYPE_IMM
    je .reg8_imm
    jmp .bad

.reg16_reg16:
    mov al, 0x89
    call emit_byte

    mov cl, [op2_value]
    shl cl, 3
    or cl, [op1_value]
    or cl, 0xC0
    mov al, cl
    call emit_byte
    jmp .ok

.reg16_imm:
    mov al, [op1_value]
    add al, 0xB8
    call emit_byte

    mov ax, [op2_value]
    call emit_word
    jmp .ok

.reg8_reg8:
    mov al, 0x88
    call emit_byte

    mov cl, [op2_value]
    shl cl, 3
    or cl, [op1_value]
    or cl, 0xC0
    mov al, cl
    call emit_byte
    jmp .ok

.reg8_imm:
    cmp word [op2_value], 256
    jae .bad

    mov al, [op1_value]
    add al, 0xB0
    call emit_byte

    mov al, [op2_value]
    call emit_byte
    jmp .ok

.ok:
    clc
    ret

.bad:
    stc
    ret


encode_add:
    cmp byte [op1_type], OPTYPE_REG16
    jne .bad

    cmp byte [op2_type], OPTYPE_REG16
    je .reg_reg
    cmp byte [op2_type], OPTYPE_IMM
    je .reg_imm
    jmp .bad

.reg_reg:
    mov al, 0x01
    call emit_byte

    mov cl, [op2_value]
    shl cl, 3
    or cl, [op1_value]
    or cl, 0xC0
    mov al, cl
    call emit_byte
    jmp .ok

.reg_imm:
    mov al, 0x81
    call emit_byte

    mov cl, [op1_value]
    or cl, 0xC0  ; reg field = 0 (/0 = ADD)
    mov al, cl
    call emit_byte

    mov ax, [op2_value]
    call emit_word
    jmp .ok

.ok:
    clc
    ret

.bad:
    stc
    ret


encode_sub:
    cmp byte [op1_type], OPTYPE_REG16
    jne .bad

    cmp byte [op2_type], OPTYPE_REG16
    je .reg_reg
    cmp byte [op2_type], OPTYPE_IMM
    je .reg_imm
    jmp .bad

.reg_reg:
    mov al, 0x29
    call emit_byte

    mov cl, [op2_value]
    shl cl, 3
    or cl, [op1_value]
    or cl, 0xC0
    mov al, cl
    call emit_byte
    jmp .ok

.reg_imm:
    mov al, 0x81
    call emit_byte

    mov cl, [op1_value]
    or cl, 0x28  ; reg field = 5 (/5 = SUB) -> 5<<3
    or cl, 0xC0
    mov al, cl
    call emit_byte

    mov ax, [op2_value]
    call emit_word
    jmp .ok

.ok:
    clc
    ret

.bad:
    stc
    ret


encode_push:
    cmp byte [op1_type], OPTYPE_REG16
    jne .bad
    cmp byte [op2_type], OPTYPE_NONE
    jne .bad

    mov al, [op1_value]
    add al, 0x50
    call emit_byte

    clc
    ret

.bad:
    stc
    ret


encode_pop:
    cmp byte [op1_type], OPTYPE_REG16
    jne .bad
    cmp byte [op2_type], OPTYPE_NONE
    jne .bad

    mov al, [op1_value]
    add al, 0x58
    call emit_byte

    clc
    ret

.bad:
    stc
    ret


encode_int:
    cmp byte [op1_type], OPTYPE_IMM
    jne .bad
    cmp byte [op2_type], OPTYPE_NONE
    jne .bad
    cmp word [op1_value], 256
    jae .bad

    mov al, 0xCD
    call emit_byte
    mov al, [op1_value]
    call emit_byte

    clc
    ret

.bad:
    stc
    ret


; -------------------------------------------------------------------------
; Разбор операндов и токенов
; -------------------------------------------------------------------------

; > Определяет тип и значение операнда из 0-terminated буфера токена
; Параметры:
;  - si: буфер токена (нижний регистр, может быть пустым)
; Вывод:
;  - al: тип операнда (OPTYPE_*)
;  - bx: значение (код регистра 0-7 либо число 0-65535, 0 если NONE)
;  - CF: 1 при ошибке (не распознанный операнд)
resolve_operand:
    push cx
    push di

    cmp byte [si], 0
    je .none

    mov di, reg16_names
    mov cx, 8
    call match_table
    jnc .found_reg16

    mov di, reg8_names
    mov cx, 8
    call match_table
    jnc .found_reg8

    call parse_operand_number
    jc .bad

    mov al, OPTYPE_IMM
    jmp .return

.found_reg16:
    mov al, OPTYPE_REG16
    mov bx, cx
    jmp .return

.found_reg8:
    mov al, OPTYPE_REG8
    mov bx, cx
    jmp .return

.none:
    xor bx, bx
    mov al, OPTYPE_NONE
    clc
    jmp .return

.bad:
    stc
    jmp .actual_return

.return:
    clc

.actual_return:
    pop di
    pop cx
    ret


; > Поиск токена (si) в таблице 3-байтовых записей (2 символа + 0)
; Параметры:
;  - si: строка токена
;  - di: начало таблицы
;  - cx: количество записей
; Вывод:
;  - cx: индекс найденной записи (0-based)
;  - CF: 0 (найдено), 1 (не найдено)
match_table:
    push ax
    push dx

    xor dx, dx

.scan:
    call str_equal
    je .found

    add di, 3
    inc dx
    loop .scan

    stc
    jmp .return

.found:
    mov cx, dx
    clc

.return:
    pop dx
    pop ax
    ret


; > Разбор числового токена: десятичное или 0xHEX (до 4 hex-цифр)
; Параметры:
;  - si: буфер токена (0-terminated, нижний регистр)
; Вывод:
;  - bx: число (0-65535)
;  - CF: 1 при ошибке (не число / мусор / переполнение hex-цифр)
parse_operand_number:
    push ax
    push cx

    cmp byte [si], '0'
    jne .decimal
    mov al, [si + 1]
    cmp al, 'x'
    je .hex
    jmp .decimal

.decimal:
    call parse_uint16
    jc .bad
    cmp byte [si], 0
    jne .bad
    mov bx, ax
    jmp .ok

.hex:
    add si, 2
    xor bx, bx
    xor cx, cx

.hex_loop:
    mov al, [si]
    test al, al
    jz .hex_done

    cmp al, '0'
    jb .bad
    cmp al, '9'
    jbe .hex_digit

    cmp al, 'a'
    jb .bad
    cmp al, 'f'
    ja .bad

    sub al, 'a' - 10
    jmp .hex_accum

.hex_digit:
    sub al, '0'

.hex_accum:
    cmp cx, 4
    jae .bad

    shl bx, 4
    or bl, al

    inc si
    inc cx
    jmp .hex_loop

.hex_done:
    test cx, cx
    jz .bad
    jmp .ok

.ok:
    clc
    jmp .return

.bad:
    stc

.return:
    pop cx
    pop ax
    ret


; > Читает токен (буквенно-цифровой, до пробела/запятой/0) из si в буфер di
; Токен приводится к нижнему регистру, всегда 0-terminated (пустой, если нечего читать)
; Параметры:
;  - si: позиция чтения
;  - di: буфер назначения (минимум TOKEN_MAX байт)
; Вывод:
;  - si: указывает на разделитель после токена (не пропускает его)
;  - CF: 1, если токен пустой
read_token:
    push ax
    push cx

    xor cx, cx

.loop:
    mov al, [si]
    test al, al
    jz .end
    cmp al, ' '
    je .end
    cmp al, ','
    je .end

    cmp cx, TOKEN_MAX - 1
    jae .skip_char

    call char_to_lower
    mov [di], al
    inc di
    inc cx

.skip_char:
    inc si
    jmp .loop

.end:
    mov byte [di], 0
    test cx, cx
    jz .empty

    pop cx
    pop ax
    clc
    ret

.empty:
    pop cx
    pop ax
    stc
    ret


; > Сбрасывает все клавиши, зависшие в буфере BIOS (не блокирует)
; Использует int 0x16 ah=1 (проверка без удаления) + ah=0 (чтение/удаление)
drain_keyboard_buffer:
    push ax

.loop:
    mov ah, 1
    int 0x16
    jz .done        ; ZF=1 - буфер пуст

    xor ah, ah
    int 0x16         ; Читаем клавишу (удаляя её из буфера)
    jmp .loop

.done:
    pop ax
    ret


; > Чтение одной строки ввода для asm-режима (без истории, в отличие от cli_input)
; Backspace работает как обычно. Ctrl+C (без Shift) очищает текущую строку.
; Ctrl+Shift+C - немедленный выход (BIOS не различает Shift для управляющих
; символов через int 0x16 ah=0, поэтому Shift проверяется отдельно через BDA).
; Вывод:
;  - input_buffer: 0-terminated строка
;  - CF: 0 - обычная строка (Enter), 1 - запрошен выход (Ctrl+Shift+C)
asm_read_line:
    push ax
    push bx
    push di

    xor bx, bx
    mov di, input_buffer

.input_loop:
    xor ah, ah
    int 0x16

    cmp al, ENTER_KEY
    je .enter_pressed

    cmp al, BACKSPACE_KEY
    je .backspace_pressed

    cmp al, CTRL_C_KEY
    je .ctrl_c_pressed

    ; Игнорируем прочие управляющие символы (Escape, стрелки и т.п.)
    cmp al, SPACE_KEY
    jb .input_loop

    ; Переполнение буфера - лишние символы молча игнорируются
    cmp bx, INPUT_BUFFER_LEN
    jae .input_loop

    call print_char
    mov [di], al
    inc di
    inc bx
    jmp .input_loop

.backspace_pressed:
    test bx, bx
    jz .input_loop

    dec di
    dec bx
    mov byte [di], 0
    call visual_erase_char
    jmp .input_loop

.ctrl_c_pressed:
    ; Проверка удержан ли Shift (BDA, флаги клавиатуры: биты 0-1)
    push es
    mov ax, BDA_KB_FLAGS_SEGMENT
    mov es, ax
    mov al, [es:BDA_KB_FLAGS_OFFSET]
    pop es
    test al, BDA_SHIFT_MASK
    jnz .abort

    ; Обычный Ctrl+C (без Shift) - просто очищаем текущую строку
.clear_line:
    test bx, bx
    jz .clear_done
    call visual_erase_char
    dec bx
    jmp .clear_line

.clear_done:
    mov di, input_buffer
    mov byte [di], 0
    jmp .input_loop

.abort:
    call print_new_line
    mov byte [di], 0
    stc
    jmp .return

.enter_pressed:
    call print_new_line
    mov byte [di], 0
    clc

.return:
    pop di
    pop bx
    pop ax
    ret


; > Пропускает пробелы и не более одной запятой между операндами
skip_operand_sep:
    call skip_spaces
    cmp byte [si], ','
    jne .done
    inc si
    call skip_spaces
.done:
    ret


; > Записывает байт al в emit_buf[emit_count], увеличивает emit_count
emit_byte:
    push bx
    push di

    mov bx, [emit_count]
    mov di, emit_buf
    add di, bx
    mov [di], al

    inc bx
    mov [emit_count], bx

    pop di
    pop bx
    ret


; > Записывает ax как 2 байта little-endian через emit_byte
emit_word:
    call emit_byte
    mov al, ah
    call emit_byte
    ret


; Таблицы имён регистров (записи по 3 байта: 2 символа + 0), порядок = код 0-7
reg16_names: db 'ax', 0, 'cx', 0, 'dx', 0, 'bx', 0, 'sp', 0, 'bp', 0, 'si', 0, 'di', 0
reg8_names:  db 'al', 0, 'cl', 0, 'dl', 0, 'bl', 0, 'ah', 0, 'ch', 0, 'dh', 0, 'bh', 0

; Имена мнемоник
str_mov:  db 'mov', 0
str_add:  db 'add', 0
str_sub:  db 'sub', 0
str_push: db 'push', 0
str_pop:  db 'pop', 0
str_int:  db 'int', 0
str_nop:  db 'nop', 0
str_hlt:  db 'hlt', 0
str_cli:  db 'cli', 0
str_sti:  db 'sti', 0

; Строки
asm_prompt: db 'A>', 0

msg_asm_intro:
    db 'Realix Mini-Assembler. Blank line to finish, run to execute.', ENTER
    db 'Ops: mov add sub push pop int nop hlt cli sti', ENTER
    db 'Regs: ax cx dx bx sp bp si di (16-bit) / al ah cl ch dl dh bl bh (8-bit)', ENTER
    db 'Imm: decimal or 0xHEX, e.g. mov ax,0x0e00', ENTER
    db 'Ctrl+C clears the current line, Ctrl+Shift+C exits immediately.', ENTER, 0

msg_asm_done:    db ENTER, 'Assembled ', 0
msg_asm_aborted: db ENTER, 'Aborted (Ctrl+Shift+C). Assembled ', 0
msg_asm_done2:   db ' bytes. Type "run" to execute.', ENTER, 0

msg_run_start:  db 'Executing ', 0
msg_run_start2: db ' bytes...', ENTER, 0
msg_run_done:   db 'Done.', ENTER, 0

err_run_empty:        db '[!] No code assembled yet. Use "asm" first.', 0
err_asm_full:         db '[!] Assembler buffer full (max 512 bytes).', 0
err_unknown_mnemonic: db '[!] Unknown instruction. Ops: mov add sub push pop int nop hlt cli sti', 0
err_bad_operand:      db '[!] Invalid operand(s) for this instruction.', 0

; Переменные модуля
asm_cursor: dw 0

; Буфер токенов текущей строки
mnemonic_buf: times TOKEN_MAX db 0
operand1_buf: times TOKEN_MAX db 0
operand2_buf: times TOKEN_MAX db 0

; Разобранные операнды текущей строки
op1_type:  db 0
op1_value: dw 0
op2_type:  db 0
op2_value: dw 0

; Байты, собранные для текущей строки
emit_buf:   times 6 db 0
emit_count: dw 0
