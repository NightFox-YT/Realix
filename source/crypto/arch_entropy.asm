; © Realix > CPU Entropy: RDRAND и RDSEED
; (18.06.26) v0.07
; ø Copyright Gleb Obitotsky <https://github.com/oxxx1mif> 2026
; ================

; Экспорт функций наружу
global arch_has_rdrand
global arch_has_rdseed
global arch_get_rdrand32
global arch_get_rdrand64
global arch_get_rdseed32
global arch_get_rdseed64
global arch_get_entropy

; Константы
%define ARCH_MAX_ENTROPY_WORDS 4096


; > Проверяет бит CPUID.01H:ECX[30] на наличие RDRAND
; Вывод:
;  - eax: 1 (поддерживается), 0 (не поддерживается)
arch_has_rdrand:
    push ebx
    push ecx
    push edx

    ; Запрос макс. значения параметра ah функции CPUID на поддержку RDRAND (1h)
    xor eax, eax
    cpuid
    cmp eax, 1
    jb .fail

    ; Запрос информации о процессоре
    mov eax, 1
    cpuid

    bt ecx, 30    ; Тестируем 30-й бит ECX (отвечает за RDRAND) с установкой CF
    xor eax, eax
    setc al       ; Записываем значение CF в al

    pop edx
    pop ecx
    pop ebx
    ret
.fail:
    ; Возвращаем 0, если нет поддержки функции 1h
    xor eax, eax

    pop edx
    pop ecx
    pop ebx
    ret


; > Проверяет бит CPUID.07H:EBX[18] на наличие RDSEED
; Вывод:
;  - eax: 1 (поддерживается), 0 (не поддерживается)
arch_has_rdseed:
    push ebx
    push ecx
    push edx

    ; Запрос макс. значения параметра ah функции CPUID на поддержку RDSEED (7h)
    xor eax, eax
    cpuid
    cmp eax, 7
    jb .fail

    ; Запрос информации о поддержке расширенных функций с нулевым sub-leaf в ECX (доп. лист)
    mov eax, 7
    xor ecx, ecx
    cpuid

    bt ebx, 18    ; Тестируем 18-й бит EBX (отвечает за RDSEED) с установкой CF
    xor eax, eax
    setc al       ; Записываем значение CF в al

    pop edx
    pop ecx
    pop ebx
    ret
.fail:
    xor eax, eax
    pop edx
    pop ecx
    pop ebx
    ret


; > Получение 32-битного случайного числа
; ø int arch_get_rdrand32(uint32_t *out, uint32_t retries)
; Параметры (64 бит, для 32 бит используйте стек):
;  - rdi: указатель куда записать 32-битное случайное число
;  - esi: максимальное количество попыток (рекомендуется 10-32)
; Вывод:
;  - eax: 1 (Успех), 0 (Ошибка)
arch_get_rdrand32:
%if __BITS__ == 64
    ; Проверка на пустые (нулевые) аргументы
    test rdi, rdi
    jz .fail64
    test esi, esi
    jz .fail64

    mov ecx, esi
.retry64:
    ; Генерация случайного 32-битного числа
    rdrand eax
    jc .ok64

    ; Оптимизируем ожидание процессора и пробуем снова
    pause
    loop .retry64
.fail64:
    ; Возврат 0 (Ошибка)
    xor eax, eax
    ret
.ok64:
    ; Запись полученного числа в память (Успех)
    mov [rdi], eax
    mov eax, 1
    ret
%else
    ; Создаём фрейм функции (с сохранением регистров)
    push ebp
    mov ebp, esp
    push ebx
    push ecx

    mov ebx, [ebp+8]   ; Первый параметр: out (указатель)
    mov ecx, [ebp+12]  ; Второй параметр: retries
    test ebx, ebx      ; Проверка указателя на NULL
    jz .fail32

.retry32:
    ; Генерация случайного 32-битного числа
    rdrand eax
    jc .ok32

    ; Оптимизируем ожидание процессора и пробуем снова
    pause
    loop .retry32
.fail32:
    ; Возврат 0 (Ошибка)
    xor eax, eax
    jmp .exit32
.ok32:
    ; Запись полученного числа в память (Успех)
    mov [ebx], eax
    mov eax, 1
.exit32:
    pop ecx
    pop ebx

    mov esp, ebp
    pop ebp
    ret
%endif


; > Получение 64-битного случайного числа (Аналогично arch_get_rdrand32)
; ø int arch_get_rdrand64(uint64_t *out, uint32_t retries)
arch_get_rdrand64:
%if __BITS__ == 64
    ; Проверка на пустые (нулевые) аргументы
    test rdi, rdi
    jz .fail
    test esi, esi
    jz .fail

    mov ecx, esi
.retry:
    ; Генерация случайного 64-битного числа
    rdrand rax
    jc .ok

    ; Оптимизируем ожидание процессора и пробуем снова
    pause
    dec ecx
    jnz .retry
.fail:
    ; Возврат 0 (Ошибка)
    xor eax, eax
    ret
.ok:
    ; Запись полученного сида и возврат 1 (Успех)
    mov [rdi], rax
    mov eax, 1
    ret
%else
    ; В 32-битной архитектуре процессор не имеет поддержки 64-битной генерации, возвращаем 0
    xor eax, eax
    ret
%endif


; > Код аналогичен rdrand32, но собирает чистую энтропию (для сидов)
; ø int arch_get_rdseed32(uint32_t *out, uint32_t retries)
; Параметры (64 бит, для 32 бит используйте стек):
;  - rdi: указатель куда записать 32-битный сид
;  - rsi: максимальное количество попыток (рекомендуется 10-32)
; Вывод:
;  - eax: 1 (Успех), 0 (Ошибка)
arch_get_rdseed32:
%if __BITS__ == 64
    ; Проверка на пустые (нулевые) аргументы
    test rdi, rdi
    jz .fail
    test esi, esi
    jz .fail

    mov ecx, esi
.retry:
    ; Генерация случайного 32-битного сида
    rdseed eax
    jc .ok

    ; Оптимизируем ожидание процессора и пробуем снова
    pause
    dec ecx
    jnz .retry
.fail:
    ; Возврат 0 (Ошибка)
    xor eax, eax
    ret
.ok:
    ; Запись полученного числа (Успех)
    mov [rdi], eax
    mov eax, 1
    ret
%else
    ; Создаём фрейм функции (с сохранением регистров)
    push ebp
    mov ebp, esp
    push ebx
    push ecx

    mov ebx, [ebp+8]   ; Первый параметр: out (указатель)
    mov ecx, [ebp+12]  ; Второй параметр: retries
    test ebx, ebx      ; Проверка указателя на NULL
    jz .fail32

.retry32:
    ; Генерация случайного 32-битного сида
    rdseed eax
    jc .ok32

    ; Оптимизируем ожидание процессора и пробуем снова
    pause
    dec ecx
    jnz .retry32

.fail32:
    ; Возврат 0 (Ошибка)
    xor eax, eax
    jmp .exit32
.ok32:
    ; Запись полученного числа в память (Успех)
    mov [ebx], eax
    mov eax, 1
.exit32:
    pop ecx
    pop ebx

    ; Выход из функции
    mov esp, ebp
    pop ebp
    ret
%endif


; В 32-битном режиме возвращает 64-битное значение через EDX:EAX
; ø int arch_get_rdseed64(uint64_t *out, uint32_t retries)
arch_get_rdseed64:
%if __BITS__ == 64
    ; Проверка на пустые (нулевые) аргументы
    test rdi, rdi
    jz .fail
    test esi, esi
    jz .fail

    mov ecx, esi
.retry:
    ; Генерация случайного 64-битного сида
    rdseed rax
    jc .ok

    ; Оптимизируем ожидание процессора и пробуем снова
    pause
    dec ecx
    jnz .retry
.fail:
    ; Возврат 0 (Ошибка)
    xor eax, eax
    ret
.ok:
    ; Запись полученного сида (Успех)
    mov [rdi], rax
    mov eax, 1
    ret
%else
    ; Создаём фрейм функции (с сохранением регистров)
    push ebp
    mov ebp, esp
    push ebx
    push ecx

    mov ebx, [ebp+8]   ; Первый параметр: out (указатель)
    mov ecx, [ebp+12]  ; Второй параметр: retries

    test ebx, ebx  ; Проверка указателя на NULL
    jz .fail32
    test ecx, ecx  ; Проверка счетчика попыток на 0
    jz .fail32

.retry32:
    ; Генерация первой половины 64-битного сида (младшие 32 бита)
    rdseed eax
    jnc .loop

    ; Сохраняем первую половину в EDX
    mov edx, eax

    ; Пытаемся получить вторую половину (старшие 32 бита)
    rdseed eax
    jc .ok32

.loop:
    pause
    dec ecx
    jnz .retry32

.fail32:
    ; Возврат 0 (Ошибка)
    xor eax, eax
    jmp .exit32

.ok32:
    ; Запись полученного числа в память
    mov [ebx], edx      
    mov [ebx+4], eax

    ; Возврат 1 (Успех)
    mov eax, 1
    jmp .exit32

.exit32:
    pop ecx
    pop ebx
    mov esp, ebp
    pop ebp
    ret
%endif


; ø int arch_get_entropy(void *buffer, uint32_t words, uint32_t retries, uint32_t source)
; Параметры:
;  - buffer:  указатель на память для записи
;  - words:   сколько 32-битных слов нужно сгенерировать
;  - retries: максимум попыток на одно слово
;  - source:  1 = RDRAND, 2 = RDSEED
; Вывод:
;  - eax: 1 (Успешно заполнено words слов), 0 (Ошибка)
; Рекомендации по использованию:
;  - Для начального seeding лучше использовать source=2 (RDSEED)
;  - Для быстрой подкачки энтропии — source=1 (RDRAND)
arch_get_entropy:
%if __BITS__ == 64
    ; SystemV ABI: rdi, rsi, rdx, rcx (Валидация входных данных)
    test rdi, rdi
    jz .fail
    test rsi, rsi
    jz .fail
    test rdx, rdx
    jz .fail

    ; Защита от слишком больших запросов
    cmp rsi, ARCH_MAX_ENTROPY_WORDS
    ja .fail

    ; Сохранение регистров
    push rbx
    push r12
    push r13
    push r14
    push r15

    mov r12, rdi  ; Текущий адрес буфера
    mov r13, rsi  ; Words
    mov r14, rdx  ; Количество попыток (retries)
    mov r15, rcx  ; Источник (1=RDRAND, 2=RDSEED)

.loop:
    ; Проверка счётчика слов
    test r13, r13
    jz .success

    ; Настройка аргументов для вызываемых функций генерации
    mov rdi, r12
    mov esi, r14d

    ; Генерирация выбранным способом
    cmp r15d, 2
    je .do_rdseed
    call arch_get_rdrand32
    jmp .check
.do_rdseed:
    call arch_get_rdseed32

.check:
    ; Проверка кода возврата (1 - Успех, 0 - Ошибка)
    test eax, eax
    jz .fail_restore

    ; Сдвигаем указатель буфера на 4 байта (к след. слову, с уменьшением счётчика)
    add r12, 4
    dec r13
    jmp .loop

.success:
    ; Возврат 1 (Успех)
    mov eax, 1

.fail_restore:
    ; Восстанавливаем сохраненные регистры
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    ret
.fail:
    xor eax, eax
    ret

%else
    ; Принимаем параметры через стек (cdecl)
    push ebp
    mov ebp, esp

    ; Сохранение регистров
    push ebx
    push esi
    push edi

    mov edi, [ebp+8]   ; Текущий адрес буфера
    mov ebx, [ebp+12]  ; Words
    mov esi, [ebp+16]  ; Количество попыток (retries)
    mov edx, [ebp+20]  ; Источник (1=RDRAND, 2=RDSEED)

    ; Проверка валидности
    test edi, edi
    jz .fail32
    test ebx, ebx
    jz .fail32
    test esi, esi
    jz .fail32
    cmp ebx, ARCH_MAX_ENTROPY_WORDS
    ja .fail32

.loop32:
    ; Проверка счётчика слов
    test ebx, ebx
    jz .success32

    ; Настраиваем аргументы для вызываемых функций генерации
    push edx
    push esi  ; Передаем retries
    push edi  ; Передаем текущий указатель на буфер

    ; Каким способом просили генерировать
    cmp edx, 2
    je .rdseed32
    call arch_get_rdrand32
    jmp .cleanup
.rdseed32:
    call arch_get_rdseed32
.cleanup:
    ; Очищаем стек от 3-х аргументов (3 * 4 байта = 12)
    add esp, 12

    ; Проверка кода возврата (1 - Успех, 0 - Ошибка)
    test eax, eax
    jz .fail32

    ; Сдвигаем указатель буфера на 4 байта (к след. слову, с уменьшением счётчика)
    add edi, 4
    dec ebx
    jmp .loop32

.success32:
    ; Возврат 1 (Успех)
    mov eax, 1
    jmp .exit32
.fail32:
    xor eax, eax
.exit32:
    ; Восстанавливаем сохраненные регистры
    pop edi
    pop esi
    pop ebx
    mov esp, ebp
    pop ebp
    ret
%endif