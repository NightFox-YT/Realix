; ════════════════════════════════════════════════════════════════════════════
;  Realix · Vault — PBKDF2-HMAC-SHA256
;  Растяжение пароля в ключ по RFC 8018. dkLen = 64 байта (2 блока SHA-256).
;  iters задаёт стоимость одной попытки подбора пароля.
;
;  Эталон    · tools/vault_forge.py (pbkdf2)
;  Зависит   · security/hmac.asm
; ════════════════════════════════════════════════════════════════════════════

; Защита от повторного включения

%ifndef PBKDF2_ASM
%define PBKDF2_ASM

%include 'security/hmac.asm'


; > PBKDF2-HMAC-SHA256 -> 64 байта
; Параметры (ячейки памяти):
;  - pbk_pw_ptr / pbk_pw_len: пароль
;  - pbk_salt_ptr / pbk_salt_len: соль
;  - pbk_iters (dd): число итераций
;  - pbk_out: буфер под 64 байта
pbkdf2_sha256:
    pushad
    push es
    push ds
    pop es                  ; es = ds

    ; Ключ HMAC = пароль (неизменен во всех вызовах)
    mov ax, [pbk_pw_ptr]
    mov [hmac_key_ptr], ax
    mov ax, [pbk_pw_len]
    mov [hmac_key_len], ax

    mov word [pbk_block], 1
.block_loop:
    ; --- U1 = HMAC(pw, salt || BE32(block)) ---
    ; Собираем сообщение в pbk_msg
    mov si, [pbk_salt_ptr]
    mov di, pbk_msg
    mov cx, [pbk_salt_len]
    rep movsb
    ; Номер блока (big-endian 32 бита, старшие байты нулевые)
    mov byte [di + 0], 0
    mov byte [di + 1], 0
    mov ax, [pbk_block]
    mov [di + 2], ah
    mov [di + 3], al

    mov ax, [pbk_salt_len]
    add ax, 4
    mov [hmac_msg_len], ax
    mov word [hmac_msg_ptr], pbk_msg
    mov word [hmac_out], pbk_u
    call hmac_sha256

    ; T = U1
    mov si, pbk_u
    mov di, pbk_t
    mov cx, 32
    rep movsb

    ; --- Остальные (iters - 1) итераций: U = HMAC(pw, U); T ^= U ---
    mov ecx, [pbk_iters]
    dec ecx
.iter:
    test ecx, ecx
    jz .store

    mov word [hmac_msg_ptr], pbk_u
    mov word [hmac_msg_len], 32
    mov word [hmac_out], pbk_u
    call hmac_sha256        ; ecx сохраняется (pushad/popad внутри)

    ; T ^= U (8 двойных слов); bx — счётчик, чтобы не трогать ecx
    mov si, pbk_u
    mov di, pbk_t
    mov bx, 8
.xor_t:
    mov eax, [si]
    xor [di], eax
    add si, 4
    add di, 4
    dec bx
    jnz .xor_t

    dec ecx
    jmp .iter

.store:
    ; out[(block - 1) * 32] = T
    mov ax, [pbk_block]
    dec ax
    shl ax, 5               ; * 32
    mov di, [pbk_out]
    add di, ax
    mov si, pbk_t
    mov cx, 32
    rep movsb

    inc word [pbk_block]
    cmp word [pbk_block], 2
    jbe .block_loop

    pop es
    popad
    ret


; Параметры вызова
pbk_pw_ptr:   dw 0
pbk_pw_len:   dw 0
pbk_salt_ptr: dw 0
pbk_salt_len: dw 0
pbk_iters:    dd 0
pbk_out:      dw 0

; Рабочие буферы
pbk_block: dw 0             ; Номер текущего блока (1..2)
pbk_msg:   times 64 db 0    ; salt || BE32(block)
pbk_u:     times 32 db 0    ; Текущий U
pbk_t:     times 32 db 0    ; Накопленный T (XOR всех U)

%endif
