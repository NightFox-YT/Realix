;; memmove.asm, the realix libc (klibc and rlibc)
;; WARNING: get from musl libc.
;; Copyright (c) 2015-2026 Musl Developers
;; Copyright (c) 2026-present Alexander Silaev
global MemMove:function
extern __memcpy_fwd        ; __memcpy_fwd как зависимость

section .text

MemMove:
    ; Аргументы в стеке (cdecl):
    ; [esp + 4]  -> dest
    ; [esp + 8]  -> src
    ; [esp + 12] -> n

    mov eax, [esp + 4]      ; eax = dest
    sub eax, [esp + 8]      ; eax = dest - src
    cmp [esp + 12], eax     ; Сравниваем n и (dest - src)
    jae __memcpy_fwd        ; Если n <= (dest - src), то перекрытия нет (или dest < src).
                            ; Прыгаем на обычный memcpy, который вернет dest в eax.

    ; Если мы здесь, значит dest > src и области перекрываются. Копируем с конца.
    push esi                ; Сохраняем регистры, так как cdecl требует их возврата
    push edi

    ; Внимание: из-за двух push стек сместился на 8 байт!
    ; Теперь аргументы лежат так:
    ; [esp + 12] -> dest
    ; [esp + 16] -> src
    ; [esp + 20] -> n

    mov edi, [esp + 12]     ; edi = dest
    mov esi, [esp + 16]     ; esi = src
    mov ecx, [esp + 20]     ; ecx = n

    lea edi, [edi + ecx - 1] ; Сдвигаем edi на последний байт (dest + n - 1)
    lea esi, [esi + ecx - 1] ; Сдвигаем esi на последний байт (src + n - 1)

    std                     ; Устанавливаем DF (Direction Flag) = 1. 
                            ; Теперь esi и edi будут уменьшаться при копировании.
    rep movsb               ; Копируем побайтово в обратном направлении
    cld                     ; ОБЯЗАТЕЛЬНО сбрасываем DF = 0, чтобы не сломать остальной C-код

    lea eax, [edi + 1]      ; После цикла edi уменьшился на 1 лишний раз. 
                            ; Восстанавливаем оригинальный dest в eax для возврата.

    pop edi                 ; Восстанавливаем регистры
    pop esi
    ret
