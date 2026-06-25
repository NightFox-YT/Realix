; Realix Kernel16 CLI
; Simple text command line for real mode.

bits 16

; Основные константы
%include 'shared/config.asm'

HISTORY_SLOTS    equ 5
HISTORY_LINE_LEN equ INPUT_BUFFER_LEN + 1

; Main shell loop.
run_cli:
    push si

.prompt:
    mov si, prompt_sign
    call print

    call cli_input

    cmp byte [input_buffer], 0
    je .prompt

    mov si, input_buffer
    call execute_cmd
    call print_new_line

    jmp .prompt

; Read one line into input_buffer.
;
; Keys:
;   Enter      submit
;   Backspace  delete last character
;   Up/Down    command history
;   PageUp     scroll console up
;   PageDown   scroll console down
;   Esc/Ctrl+U clear current line
;
; Left/Right editing is disabled for now.
; It needs prompt-boundary handling, otherwise the cursor can move before "Realix>".
cli_input:
    push ax
    push bx
    push di

    xor bx, bx
    mov di, input_buffer
    mov byte [input_buffer], 0
    mov byte [history_view], 0

.input_loop:
    xor ah, ah
    int 0x16

    cmp al, 0
    je .extended_key

    cmp al, 0xE0
    je .extended_key

    cmp al, ENTER_KEY
    je .enter

    cmp al, BACKSPACE_KEY
    je .backspace

    cmp al, 0x1B            ; Esc
    je .clear_line

    cmp al, 0x15            ; Ctrl+U
    je .clear_line

    cmp al, 0x20
    jb .input_loop

    mov byte [history_view], 0
    call console_to_bottom

    cmp bx, INPUT_BUFFER_LEN
    jae .buffer_full

    mov [di], al
    inc di
    inc bx
    mov byte [di], 0

    mov al, [di - 1]
    call cli_print_char

    jmp .input_loop

.extended_key:
    cmp ah, 0x48            ; Up
    je .history_up

    cmp ah, 0x50            ; Down
    je .history_down

    cmp ah, 0x49            ; PageUp
    je .page_up

    cmp ah, 0x51            ; PageDown
    je .page_down

    ; Left/Right/Home/End/Delete are ignored for now.
    jmp .input_loop

.history_up:
    call history_prev
    jmp .input_loop

.history_down:
    call history_next
    jmp .input_loop

.page_up:
    call console_page_up
    jmp .input_loop

.page_down:
    call console_page_down
    jmp .input_loop

.backspace:
    mov byte [history_view], 0
    call console_to_bottom

    test bx, bx
    jz .input_loop

    dec di
    dec bx
    mov byte [di], 0

    call cli_erase_char
    jmp .input_loop

.clear_line:
    mov byte [history_view], 0
    call console_to_bottom
    call cli_clear_input
    jmp .input_loop

.buffer_full:
    mov al, BEEP_CHAR
    call cli_print_char
    jmp .input_loop

.enter:
    call print_new_line
    mov byte [di], 0

    cmp byte [input_buffer], 0
    je .skip_save

    call history_save

.skip_save:
    mov byte [history_view], 0

    pop di
    pop bx
    pop ax
    ret

; Print AL through BIOS TTY.
; Input echo is intentionally not written to the scrollback buffer.
cli_print_char:
    push ax
    push bx

    mov ah, 0x0E
    xor bx, bx
    int 0x10

    pop bx
    pop ax
    ret

; Visually erase one character from the screen.
cli_erase_char:
    push ax

    mov al, 0x08
    call cli_print_char
    mov al, ' '
    call cli_print_char
    mov al, 0x08
    call cli_print_char

    pop ax
    ret

; Clear current input line.
; Uses and updates BX as current length and DI as input pointer.
cli_clear_input:
    test bx, bx
    jz .done

.loop:
    call cli_erase_char
    dec di
    dec bx
    mov byte [di], 0
    test bx, bx
    jnz .loop

.done:
    mov di, input_buffer
    ret

; Save input_buffer to command history.
history_save:
    push ax
    push bx
    push cx
    push si
    push di

    mov al, [history_next_slot]
    call history_get_ptr
    mov di, si
    mov si, input_buffer
    mov cx, HISTORY_LINE_LEN

.copy_loop:
    mov al, [si]
    mov [di], al
    inc si
    inc di
    test al, al
    jz .copied
    loop .copy_loop

.copied:
    mov al, [history_next_slot]
    inc al
    cmp al, HISTORY_SLOTS
    jb .store_next
    xor al, al

.store_next:
    mov [history_next_slot], al

    mov al, [history_count]
    cmp al, HISTORY_SLOTS
    jae .done
    inc al
    mov [history_count], al

.done:
    pop di
    pop si
    pop cx
    pop bx
    pop ax
    ret

; Go to older command.
; Uses and updates BX/DI from cli_input.
history_prev:
    push ax
    push si

    cmp byte [history_count], 0
    je .done

    mov al, [history_view]
    cmp al, [history_count]
    jae .show

    inc al
    mov [history_view], al

.show:
    call history_show

.done:
    pop si
    pop ax
    ret

; Go to newer command, or clear line after newest command.
; Uses and updates BX/DI from cli_input.
history_next:
    push ax
    push si

    cmp byte [history_view], 0
    je .done

    cmp byte [history_view], 1
    jne .move_newer

    mov byte [history_view], 0
    call console_to_bottom
    call cli_clear_input
    jmp .done

.move_newer:
    dec byte [history_view]
    call history_show

.done:
    pop si
    pop ax
    ret

; Show command selected by history_view.
; Uses and updates BX/DI from cli_input.
history_show:
    call console_to_bottom
    call cli_clear_input

    mov al, [history_next_slot]
    mov dl, [history_view]

.index_loop:
    cmp dl, 0
    je .index_ready

    cmp al, 0
    jne .dec_index
    mov al, HISTORY_SLOTS

.dec_index:
    dec al
    dec dl
    jmp .index_loop

.index_ready:
    call history_get_ptr
    call cli_load_history_line
    ret

; AL = history slot index.
; Returns SI = pointer to slot.
history_get_ptr:
    push ax
    push bx
    push cx

    xor ah, ah
    mov bx, ax
    mov cl, 6
    shl bx, cl              ; index * 64
    add bx, ax              ; index * 65

    mov si, history_buffer
    add si, bx

    pop cx
    pop bx
    pop ax
    ret

; Copy history line from DS:SI to input_buffer and echo it.
; Updates BX and DI for cli_input.
cli_load_history_line:
    mov di, input_buffer
    xor bx, bx

.loop:
    mov al, [si]
    mov [di], al
    test al, al
    jz .done

    call cli_print_char

    inc si
    inc di
    inc bx
    cmp bx, INPUT_BUFFER_LEN
    jb .loop

    mov byte [di], 0

.done:
    ret

prompt_sign: db 'Realix> ', 0

input_buffer: times HISTORY_LINE_LEN db 0

history_count: db 0
history_next_slot: db 0
history_view:  db 0
history_buffer: times HISTORY_SLOTS * HISTORY_LINE_LEN db 0
