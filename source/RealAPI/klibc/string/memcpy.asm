;; memcpy.asm, the realix libc (klibc and rlibc)
;; WARNING: get from musl libc.
;; Copyright (c) 2015-2026 Musl Developers
;; Copyright (c) 2026-present Alexander Silaev

global MemCopy:function
global __memcpy_fwd:function

section .text

MemCopy:
__memcpy_fwd:
    ; Сохраняем регистры в стек по соглашению cdecl
    push esi
    push edi

    ; Стек сместился на 8 байт, поэтому аргументы теперь тут:
    ; [esp + 12] -> dest
    ; [esp + 16] -> src
    ; [esp + 20] -> n

    mov edi, [esp + 12]     ; edi = dest
    mov esi, [esp + 16]     ; esi = src
    mov ecx, [esp + 20]     ; ecx = n
    mov eax, edi            ; Сохраняем dest в eax, чтобы вернуть его в конце

    cmp ecx, 4              ; Если нужно скопировать меньше 4 байт
    jc .label1              ; ...то выравнивать нет смысла, прыгаем сразу на побайтовое

    test edi, 3             ; Проверяем, выровнен ли адрес dest по границе 4 байт
    jz .label1              ; Если младшие два бита нулевые — адрес выровнен, прыгаем дальше

.label2_align:
    movsb                   ; Копируем 1 байт (edi и esi сдвигаются вперед автоматически)
    dec ecx                 ; Уменьшаем общий счетчик байт n
    test edi, 3             ; Проверяем выравнивание снова
    jnz .label2_align       ; Если всё еще не выровнено, продолжаем цикл

.label1:
    mov edx, ecx            ; Сохраняем текущее значение ecx в edx для обработки остатка
    shr ecx, 2              ; Делим ecx на 4 (получаем количество двойных слов)
    rep movsd               ; Быстрое аппаратное копирование по 4 байта за раз

    and edx, 3              ; Выделяем остаток (0, 1, 2 или 3 байта)
    jz .label1_exit         ; Если остатка нет, выходим

.label2_tail:
    movsb                   ; Докопируем оставшиеся "хвосты" побайтово
    dec edx                 ; Уменьшаем счетчик остатка
    jnz .label2_tail        ; Повторяем, пока edx не станет равен 0

.label1_exit:
    pop edi                 ; Восстанавливаем сохраненные регистры
    pop esi
    ret                     ; Возвращаем eax