; © Realix > 32-bit RLX Snake Game (Ring 3 User Mode)
; ===================================================

bits 32
org 0x00400000

%include 'shared/rlx.inc'

rlx_header:
    db RLX_MAGIC_0, RLX_MAGIC_1, RLX_MAGIC_2, RLX_MODE_32
    dd app_entry - rlx_header ; Entry offset (0x10)
    dd code_end - app_entry   ; Code size
    dd 0                      ; Reserved

app_entry:
    ; Сброс экрана
    mov eax, SYS_CLEAR
    int 0x80

    mov eax, SYS_PRINT_STRING
    mov esi, msg_welcome
    int 0x80

    mov eax, SYS_READ_KEY
    int 0x80

game_loop:
    ; Отрисовка поля
    call render_game

    ; Чтение клавиши от пользователя (W/A/S/D/Q)
    mov eax, SYS_READ_KEY
    int 0x80

    ; EAX содержит ASCII символ
    cmp al, 'w'
    je .move_up
    cmp al, 'W'
    je .move_up
    cmp al, 's'
    je .move_down
    cmp al, 'S'
    je .move_down
    cmp al, 'a'
    je .move_left
    cmp al, 'A'
    je .move_left
    cmp al, 'd'
    je .move_right
    cmp al, 'D'
    je .move_right
    cmp al, 'q'
    je exit_game
    cmp al, 'Q'
    je exit_game
    jmp game_loop

.move_up:
    mov byte [dir_x], 0
    mov byte [dir_y], -1
    call update_snake
    jmp game_loop

.move_down:
    mov byte [dir_x], 0
    mov byte [dir_y], 1
    call update_snake
    jmp game_loop

.move_left:
    mov byte [dir_x], -1
    mov byte [dir_y], 0
    call update_snake
    jmp game_loop

.move_right:
    mov byte [dir_x], 1
    mov byte [dir_y], 0
    call update_snake
    jmp game_loop

update_snake:
    ; Движение головы змейки
    mov al, [snake_x]
    add al, [dir_x]
    mov [snake_x], al

    mov bl, [snake_y]
    add bl, [dir_y]
    mov [snake_y], bl

    ; Проверка столкновения со стеной (0..19 по X, 0..9 по Y)
    cmp al, 0
    jl game_over
    cmp al, 20
    jge game_over
    cmp bl, 0
    jl game_over
    cmp bl, 10
    jge game_over

    ; Проверка поедания яблока
    mov cl, [food_x]
    cmp al, cl
    jne .check_done
    mov cl, [food_y]
    cmp bl, cl
    jne .check_done

    ; Поели яблоко! Увеличиваем счет и перемещаем яблоко
    inc byte [score]
    add byte [food_x], 3
    add byte [food_y], 2
    
    ; Нормализация координат яблока
    mov al, [food_x]
    xor ah, ah
    mov cl, 18
    div cl
    inc ah
    mov [food_x], ah

    mov al, [food_y]
    xor ah, ah
    mov cl, 8
    div cl
    inc ah
    mov [food_y], ah

.check_done:
    ret

render_game:
    mov eax, SYS_CLEAR
    int 0x80

    mov eax, SYS_PRINT_STRING
    mov esi, msg_header
    int 0x80

    ; Отрисовка верхней рамки
    mov eax, SYS_PRINT_STRING
    mov esi, msg_border_top
    int 0x80

    ; Отрисовка игрового поля 10x20
    mov byte [cur_y], 0
.row_loop:
    cmp byte [cur_y], 10
    jge .row_done

    ; Левая граница #
    mov eax, SYS_PUTCHAR
    mov edx, '#'
    int 0x80

    mov byte [cur_x], 0
.col_loop:
    cmp byte [cur_x], 20
    jge .col_done

    ; Проверка головы змейки
    mov al, [cur_x]
    cmp al, [snake_x]
    jne .check_food
    mov al, [cur_y]
    cmp al, [snake_y]
    jne .check_food

    ; Рисуем голову 'O'
    mov eax, SYS_PUTCHAR
    mov edx, 'O'
    int 0x80
    jmp .next_col

.check_food:
    mov al, [cur_x]
    cmp al, [food_x]
    jne .draw_empty
    mov al, [cur_y]
    cmp al, [food_y]
    jne .draw_empty

    ; Рисуем яблоко '@'
    mov eax, SYS_PUTCHAR
    mov edx, '@'
    int 0x80
    jmp .next_col

.draw_empty:
    mov eax, SYS_PUTCHAR
    mov edx, '.'
    int 0x80

.next_col:
    inc byte [cur_x]
    jmp .col_loop

.col_done:
    ; Правая граница # и перенос строки
    mov eax, SYS_PRINT_STRING
    mov esi, msg_border_right
    int 0x80

    inc byte [cur_y]
    jmp .row_loop

.row_done:
    ; Нижняя граница
    mov eax, SYS_PRINT_STRING
    mov esi, msg_border_top
    int 0x80

    ; Печать управления
    mov eax, SYS_PRINT_STRING
    mov esi, msg_controls
    int 0x80
    ret

game_over:
    mov eax, SYS_CLEAR
    int 0x80
    mov eax, SYS_PRINT_STRING
    mov esi, msg_game_over
    int 0x80

    ; Ожидание нажатия клавиши перед выходом
    mov eax, SYS_READ_KEY
    int 0x80

exit_game:
    mov eax, SYS_CLEAR
    int 0x80
    mov eax, SYS_PRINT_STRING
    mov esi, msg_exit
    int 0x80

    mov eax, SYS_EXIT
    int 0x80

; --- Переменные игры ---
snake_x: db 5
snake_y: db 5
dir_x:   db 1
dir_y:   db 0
food_x:  db 12
food_y:  db 4
score:   db 0
cur_x:   db 0
cur_y:   db 0

; --- Текстовые строки ---
msg_welcome:      db '=== Realix 32-bit Snake Game (Ring 3) ===', 0x0D, 0x0A, 'Press any key to start...', 0x0D, 0x0A, 0
msg_header:       db '--- REALIX SNAKE (Ring 3 User Mode) ---', 0x0D, 0x0A, 0
msg_border_top:   db '######################', 0x0D, 0x0A, 0
msg_border_right: db '#', 0x0D, 0x0A, 0
msg_controls:     db 'Controls: [W] Up | [S] Down | [A] Left | [D] Right | [Q] Quit', 0x0D, 0x0A, 0
msg_game_over:    db 0x0D, 0x0A, '*** GAME OVER! You hit the wall! ***', 0x0D, 0x0A, 'Press any key to exit to Realix Shell...', 0x0D, 0x0A, 0
msg_exit:         db '[Snake] Exiting game cleanly...', 0x0D, 0x0A, 0

code_end:
