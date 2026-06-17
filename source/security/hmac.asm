; ════════════════════════════════════════════════════════════════════════════
;  Realix · Vault — HMAC-SHA256
;  Имитовставка по RFC 2104 поверх потокового SHA-256.
;
;  Эталон    · tools/vault_forge.py (_hmac_sha256)
;  Зависит   · security/sha256.asm
;  Стиль     · аргументы передаём через именованные ячейки, а не через регистры
; ════════════════════════════════════════════════════════════════════════════

; Защита от повторного включения

%ifndef HMAC_ASM
%define HMAC_ASM

%include 'security/sha256.asm'


; > HMAC-SHA256(key, msg) -> 32 байта
; Параметры (ячейки памяти, ds-относительные смещения):
;  - hmac_key_ptr / hmac_key_len: ключ
;  - hmac_msg_ptr / hmac_msg_len: сообщение
;  - hmac_out: буфер под тег (32 байта)
hmac_sha256:
    pushad
    push es
    push ds
    pop es                  ; es = ds

    ; --- Подготовка K0 (64 байта, дополнено нулями) ---
    mov di, hmac_k0
    mov cx, 64
    xor al, al
    rep stosb

    ; Если ключ длиннее блока — K0 = SHA256(key), иначе просто копия
    mov cx, [hmac_key_len]
    cmp cx, 64
    jbe .copy_key
    call sha256_init
    mov si, [hmac_key_ptr]
    mov cx, [hmac_key_len]
    call sha256_update
    mov di, hmac_k0
    call sha256_final
    jmp .have_k0
.copy_key:
    mov si, [hmac_key_ptr]
    mov di, hmac_k0
    rep movsb
.have_k0:

    ; --- Внутренний хеш: SHA(ipad || msg), ipad = K0 ^ 0x36 ---
    mov si, hmac_k0
    mov di, hmac_pad
    mov cx, 64
.ipad:
    mov al, [si]
    xor al, 0x36
    mov [di], al
    inc si
    inc di
    loop .ipad

    call sha256_init
    mov si, hmac_pad
    mov cx, 64
    call sha256_update
    mov si, [hmac_msg_ptr]
    mov cx, [hmac_msg_len]
    call sha256_update
    mov di, hmac_inner
    call sha256_final

    ; --- Внешний хеш: SHA(opad || inner), opad = K0 ^ 0x5c ---
    mov si, hmac_k0
    mov di, hmac_pad
    mov cx, 64
.opad:
    mov al, [si]
    xor al, 0x5c
    mov [di], al
    inc si
    inc di
    loop .opad

    call sha256_init
    mov si, hmac_pad
    mov cx, 64
    call sha256_update
    mov si, hmac_inner
    mov cx, 32
    call sha256_update
    mov di, [hmac_out]
    call sha256_final

    pop es
    popad
    ret


; Параметры вызова
hmac_key_ptr: dw 0
hmac_key_len: dw 0
hmac_msg_ptr: dw 0
hmac_msg_len: dw 0
hmac_out:     dw 0

; Рабочие буферы
hmac_k0:    times 64 db 0   ; Нормализованный ключ
hmac_pad:   times 64 db 0   ; ipad / opad
hmac_inner: times 32 db 0   ; Результат внутреннего хеша

%endif
